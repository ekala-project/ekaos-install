//! Safe command execution with mock support
//!
//! This module provides a safe abstraction for executing system commands
//! with support for both mock (testing) and real execution modes.

use crate::error::{CommandError, Result};
use std::collections::HashMap;
use std::process::{Command, Output, Stdio};
use tracing::{debug, info, warn};

/// Trait for command execution (allows mocking)
pub trait CommandExecutor: Send + Sync {
    /// Execute a command and return its output
    fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput>;

    /// Check if a command exists in the system
    fn command_exists(&self, command: &str) -> bool;
}

/// Output from a command execution
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
    /// Whether the command succeeded
    pub success: bool,
}

impl CommandOutput {
    /// Create a new command output
    pub fn new(stdout: String, stderr: String, exit_code: i32) -> Self {
        Self {
            stdout,
            stderr,
            exit_code,
            success: exit_code == 0,
        }
    }

    /// Create a successful output
    pub fn success(stdout: impl Into<String>) -> Self {
        Self::new(stdout.into(), String::new(), 0)
    }

    /// Create a failed output
    pub fn failure(exit_code: i32, stderr: impl Into<String>) -> Self {
        Self::new(String::new(), stderr.into(), exit_code)
    }
}

impl From<Output> for CommandOutput {
    fn from(output: Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            success: output.status.success(),
        }
    }
}

/// Real command executor that actually runs system commands
#[derive(Debug, Default)]
pub struct RealExecutor;

impl CommandExecutor for RealExecutor {
    fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput> {
        info!("Executing command: {} {}", command, args.join(" "));

        let output = Command::new(command)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CommandError::NotFound(command.to_string())
                } else {
                    CommandError::InvalidArguments(e.to_string())
                }
            })?;

        let cmd_output = CommandOutput::from(output);

        if !cmd_output.success {
            warn!(
                "Command '{}' failed with exit code {}: {}",
                command, cmd_output.exit_code, cmd_output.stderr
            );
            return Err(CommandError::ExecutionFailed {
                command: command.to_string(),
                code: cmd_output.exit_code,
                stderr: cmd_output.stderr.clone(),
            }
            .into());
        }

        debug!("Command succeeded with output: {}", cmd_output.stdout);
        Ok(cmd_output)
    }

    fn command_exists(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}

/// Mock command executor for testing
#[derive(Debug)]
pub struct MockExecutor {
    /// Predefined responses for commands
    responses: HashMap<String, CommandOutput>,
}

impl Default for MockExecutor {
    fn default() -> Self {
        let mut executor = Self {
            responses: HashMap::new(),
        };

        // Set up default mock responses
        executor.add_response(
            "lsblk",
            CommandOutput::success(
                r#"{
   "blockdevices": [
      {"name": "sda", "size": "256G", "type": "disk", "mountpoint": null,
       "children": [
          {"name": "sda1", "size": "512M", "type": "part", "mountpoint": null},
          {"name": "sda2", "size": "247.5G", "type": "part", "mountpoint": null},
          {"name": "sda3", "size": "8G", "type": "part", "mountpoint": null}
       ]
      }
   ]
}"#,
            ),
        );

        executor.add_response(
            "test -d /sys/firmware/efi",
            CommandOutput::success(""), // UEFI system
        );

        executor.add_response(
            "ip link show",
            CommandOutput::success(
                "1: lo: <LOOPBACK,UP,LOWER_UP>\n2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP>",
            ),
        );

        executor.add_response(
            "curl -I https://cache.nixos.org",
            CommandOutput::success("HTTP/2 200"),
        );

        executor.add_response("uname -m", CommandOutput::success("x86_64\n"));

        executor.add_response(
            "grep -c processor /proc/cpuinfo",
            CommandOutput::success("8\n"),
        );

        executor.add_response(
            "free -h",
            CommandOutput::success("              total        used        free      shared  buff/cache   available\nMem:           15Gi       2.0Gi       10Gi       100Mi       3.0Gi       13Gi\nSwap:            0B          0B          0B"),
        );

        executor
    }
}

impl MockExecutor {
    /// Create a new mock executor
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mock response for a command
    pub fn add_response(&mut self, command: &str, output: CommandOutput) {
        self.responses.insert(command.to_string(), output);
    }

    /// Generate a command key from command and args
    fn command_key(command: &str, args: &[&str]) -> String {
        if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        }
    }
}

impl CommandExecutor for MockExecutor {
    fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput> {
        let key = Self::command_key(command, args);
        debug!("Mock executing: {}", key);

        // Try exact match first
        if let Some(output) = self.responses.get(&key) {
            return Ok(output.clone());
        }

        // Try command without args
        if let Some(output) = self.responses.get(command) {
            return Ok(output.clone());
        }

        // Default successful response for unknown commands in mock mode
        warn!("No mock response for command: {}", key);
        Ok(CommandOutput::success(format!(
            "[MOCK] Command executed: {}",
            key
        )))
    }

    fn command_exists(&self, command: &str) -> bool {
        // In mock mode, all standard commands exist
        matches!(
            command,
            "lsblk"
                | "parted"
                | "mkfs.ext4"
                | "mkfs.fat"
                | "mkswap"
                | "mount"
                | "umount"
                | "nixos-generate-config"
                | "nixos-install"
        )
    }
}

/// Builder for safely constructing and executing system commands
pub struct SafeCommand<'a> {
    command: String,
    args: Vec<String>,
    executor: &'a dyn CommandExecutor,
    description: String,
}

impl<'a> SafeCommand<'a> {
    /// Create a new safe command
    pub fn new(
        command: impl Into<String>,
        executor: &'a dyn CommandExecutor,
        description: impl Into<String>,
    ) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            executor,
            description: description.into(),
        }
    }

    /// Add an argument to the command
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        let arg = arg.into();
        // Basic validation to prevent command injection
        if arg.contains(';') || arg.contains('&') || arg.contains('|') {
            warn!("Potentially dangerous characters in argument: {}", arg);
        }
        self.args.push(arg);
        self
    }

    /// Add multiple arguments
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for arg in args {
            self = self.arg(arg);
        }
        self
    }

    /// Execute the command
    pub fn execute(&self) -> Result<CommandOutput> {
        let args: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        self.executor.execute(&self.command, &args)
    }

    /// Get the command description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the full command as a string (for display purposes)
    pub fn to_string(&self) -> String {
        format!("{} {}", self.command, self.args.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_output_creation() {
        let output = CommandOutput::success("test output");
        assert!(output.success);
        assert_eq!(output.stdout, "test output");
        assert_eq!(output.exit_code, 0);

        let output = CommandOutput::failure(1, "error message");
        assert!(!output.success);
        assert_eq!(output.stderr, "error message");
        assert_eq!(output.exit_code, 1);
    }

    #[test]
    fn test_mock_executor() {
        let executor = MockExecutor::new();
        let result = executor.execute("lsblk", &["-J"]).unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("blockdevices"));
    }

    #[test]
    fn test_safe_command_builder() {
        let executor = MockExecutor::new();
        let cmd = SafeCommand::new("test", &executor, "Test command")
            .arg("arg1")
            .arg("arg2");

        assert_eq!(cmd.to_string(), "test arg1 arg2");
        assert_eq!(cmd.description(), "Test command");
    }

    #[test]
    fn test_command_exists() {
        let executor = MockExecutor::new();
        assert!(executor.command_exists("lsblk"));
        assert!(!executor.command_exists("nonexistent"));
    }
}
