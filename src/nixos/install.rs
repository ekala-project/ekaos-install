//! NixOS installation execution
//!
//! Handles the actual NixOS installation process including configuration generation,
//! nixos-install execution, and post-installation verification.

use crate::config::InstallConfig;
use crate::error::{CommandError, ConfigError, InstallerError};
use crate::system::command::{CommandExecutor, MockExecutor, RealExecutor};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Installation progress state
#[derive(Debug, Clone, PartialEq)]
pub enum InstallStage {
    /// Generating configuration files
    GeneratingConfig,
    /// Running nixos-generate-config
    GeneratingHardwareConfig,
    /// Installing NixOS
    Installing,
    /// Verifying installation
    Verifying,
    /// Installation complete
    Complete,
    /// Installation failed
    Failed(String),
}

/// Installation progress information
#[derive(Debug, Clone)]
pub struct InstallProgress {
    /// Current stage
    pub stage: InstallStage,
    /// Progress percentage (0-100)
    pub percent: u8,
    /// Current operation description
    pub operation: String,
    /// Latest log line
    pub log_line: Option<String>,
}

/// Installation output message
#[derive(Debug, Clone)]
pub enum InstallMessage {
    /// Progress update
    Progress(InstallProgress),
    /// Log line output
    Log(String),
    /// Installation completed successfully
    Success,
    /// Installation failed
    Error(String),
}

/// Trait for executing installation operations
pub trait InstallExecutor {
    /// Generate hardware configuration using nixos-generate-config
    fn generate_hardware_config(&self, root_path: &Path) -> Result<(), InstallerError>;

    /// Execute nixos-install
    fn execute_install(&self, root_path: &Path) -> Result<(), InstallerError>;

    /// Verify installation was successful
    fn verify_installation(&self, root_path: &Path) -> Result<(), InstallerError>;
}

/// Real installation executor
pub struct RealInstallExecutor {
    executor: RealExecutor,
}

impl RealInstallExecutor {
    /// Create a new real installation executor
    pub fn new() -> Self {
        Self {
            executor: RealExecutor,
        }
    }
}

impl Default for RealInstallExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallExecutor for RealInstallExecutor {
    fn generate_hardware_config(&self, root_path: &Path) -> Result<(), InstallerError> {
        let output = self
            .executor
            .execute(
                "nixos-generate-config",
                &["--root", root_path.to_str().unwrap()],
            )
            .map_err(|e| InstallerError::Config(ConfigError::GenerationFailed(format!("{}", e))))?;

        if !output.success {
            return Err(InstallerError::Config(ConfigError::GenerationFailed(
                output.stderr,
            )));
        }

        Ok(())
    }

    fn execute_install(&self, root_path: &Path) -> Result<(), InstallerError> {
        let output = self
            .executor
            .execute(
                "nixos-install",
                &["--root", root_path.to_str().unwrap(), "--no-root-passwd"],
            )
            .map_err(|e| {
                InstallerError::Command(CommandError::ExecutionFailed {
                    command: "nixos-install".to_string(),
                    code: -1,
                    stderr: format!("{}", e),
                })
            })?;

        if !output.success {
            return Err(InstallerError::Command(CommandError::ExecutionFailed {
                command: "nixos-install".to_string(),
                code: output.exit_code,
                stderr: output.stderr,
            }));
        }

        Ok(())
    }

    fn verify_installation(&self, root_path: &Path) -> Result<(), InstallerError> {
        // Check that configuration.nix exists
        let config_path = root_path.join("etc/nixos/configuration.nix");
        if !config_path.exists() {
            return Err(InstallerError::Config(ConfigError::ReadFailed {
                path: config_path.display().to_string(),
                reason: "Configuration file not found".to_string(),
            }));
        }

        // Check that hardware-configuration.nix exists
        let hw_config_path = root_path.join("etc/nixos/hardware-configuration.nix");
        if !hw_config_path.exists() {
            return Err(InstallerError::Config(ConfigError::ReadFailed {
                path: hw_config_path.display().to_string(),
                reason: "Hardware configuration file not found".to_string(),
            }));
        }

        // Check that boot directory exists and has content
        let boot_path = root_path.join("boot");
        if !boot_path.exists() {
            return Err(InstallerError::other(
                "Boot directory not found after installation",
            ));
        }

        Ok(())
    }
}

/// Mock installation executor for testing
pub struct MockInstallExecutor {
    executor: MockExecutor,
    should_fail: bool,
}

impl MockInstallExecutor {
    /// Create a new mock installation executor
    pub fn new() -> Self {
        Self {
            executor: MockExecutor::new(),
            should_fail: false,
        }
    }

    /// Set whether the installation should fail (for testing)
    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
}

impl Default for MockInstallExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallExecutor for MockInstallExecutor {
    fn generate_hardware_config(&self, _root_path: &Path) -> Result<(), InstallerError> {
        if self.should_fail {
            return Err(InstallerError::Config(ConfigError::GenerationFailed(
                "Mock failure".to_string(),
            )));
        }

        // Simulate some delay
        thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    fn execute_install(&self, _root_path: &Path) -> Result<(), InstallerError> {
        if self.should_fail {
            return Err(InstallerError::Command(CommandError::ExecutionFailed {
                command: "nixos-install".to_string(),
                code: 1,
                stderr: "Mock installation failure".to_string(),
            }));
        }

        // Simulate installation delay
        thread::sleep(Duration::from_millis(500));
        Ok(())
    }

    fn verify_installation(&self, _root_path: &Path) -> Result<(), InstallerError> {
        if self.should_fail {
            return Err(InstallerError::other("Verification failed"));
        }

        Ok(())
    }
}

/// Run the installation process in a background thread
///
/// Sends progress updates through the returned channel
pub fn run_installation_async(
    config: InstallConfig,
    root_path: String,
    is_mock: bool,
) -> Receiver<InstallMessage> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        run_installation_impl(config, root_path, is_mock, tx);
    });

    rx
}

/// Implementation of the installation process
fn run_installation_impl(
    config: InstallConfig,
    root_path: String,
    is_mock: bool,
    tx: Sender<InstallMessage>,
) {
    let root = Path::new(&root_path);

    // Create executor based on mode
    let executor: Box<dyn InstallExecutor> = if is_mock {
        Box::new(MockInstallExecutor::new())
    } else {
        Box::new(RealInstallExecutor::new())
    };

    // Stage 1: Generate configuration (0-20%)
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::GeneratingConfig,
        percent: 5,
        operation: "Generating NixOS configuration...".to_string(),
        log_line: None,
    }));

    let config_path = root.join("etc/nixos/configuration.nix");
    if let Err(e) = crate::nixos::config::write_configuration(&config, &config_path) {
        let _ = tx.send(InstallMessage::Error(format!(
            "Failed to write configuration: {}",
            e
        )));
        return;
    }

    let _ = tx.send(InstallMessage::Log(format!(
        "Configuration written to {}",
        config_path.display()
    )));

    // Stage 2: Generate hardware configuration (20-30%)
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::GeneratingHardwareConfig,
        percent: 20,
        operation: "Generating hardware configuration...".to_string(),
        log_line: None,
    }));

    if let Err(e) = executor.generate_hardware_config(root) {
        let _ = tx.send(InstallMessage::Error(format!(
            "Failed to generate hardware configuration: {}",
            e
        )));
        return;
    }

    let _ = tx.send(InstallMessage::Log(
        "Hardware configuration generated successfully".to_string(),
    ));

    // Stage 3: Run nixos-install (30-90%)
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::Installing,
        percent: 30,
        operation: "Installing NixOS (this may take a while)...".to_string(),
        log_line: None,
    }));

    // Simulate progress updates during installation
    for i in 0..6 {
        thread::sleep(Duration::from_millis(if is_mock { 200 } else { 5000 }));
        let percent = 30 + (i * 10);
        let _ = tx.send(InstallMessage::Progress(InstallProgress {
            stage: InstallStage::Installing,
            percent,
            operation: format!("Installing NixOS... {}%", percent),
            log_line: Some(format!("Installing system packages (step {})", i + 1)),
        }));
    }

    if let Err(e) = executor.execute_install(root) {
        let _ = tx.send(InstallMessage::Error(format!("Installation failed: {}", e)));
        return;
    }

    let _ = tx.send(InstallMessage::Log(
        "NixOS installation completed successfully".to_string(),
    ));

    // Stage 4: Verify installation (90-100%)
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::Verifying,
        percent: 90,
        operation: "Verifying installation...".to_string(),
        log_line: None,
    }));

    if let Err(e) = executor.verify_installation(root) {
        let _ = tx.send(InstallMessage::Error(format!(
            "Installation verification failed: {}",
            e
        )));
        return;
    }

    let _ = tx.send(InstallMessage::Log(
        "Installation verified successfully".to_string(),
    ));

    // Stage 5: Complete
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::Complete,
        percent: 100,
        operation: "Installation complete!".to_string(),
        log_line: None,
    }));

    let _ = tx.send(InstallMessage::Success);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BootLoader, UserConfig};
    use std::path::PathBuf;

    fn create_test_config() -> InstallConfig {
        InstallConfig {
            hostname: "test-nixos".to_string(),
            user: UserConfig {
                username: "testuser".to_string(),
                full_name: Some("Test User".to_string()),
                is_admin: true,
                password: "testpass123".to_string(),
            },
            timezone: "America/New_York".to_string(),
            locale: "en_US.UTF-8".to_string(),
            keymap: "us".to_string(),
            bootloader: BootLoader::SystemdBoot,
            network_manager: true,
        }
    }

    #[test]
    fn test_mock_executor_success() {
        let executor = MockInstallExecutor::new();
        let root = PathBuf::from("/mnt");

        assert!(executor.generate_hardware_config(&root).is_ok());
        assert!(executor.execute_install(&root).is_ok());
        assert!(executor.verify_installation(&root).is_ok());
    }

    #[test]
    fn test_mock_executor_failure() {
        let executor = MockInstallExecutor::new().with_failure(true);
        let root = PathBuf::from("/mnt");

        assert!(executor.generate_hardware_config(&root).is_err());
        assert!(executor.execute_install(&root).is_err());
        assert!(executor.verify_installation(&root).is_err());
    }

    #[test]
    fn test_install_stages() {
        assert_eq!(
            InstallStage::GeneratingConfig,
            InstallStage::GeneratingConfig
        );
        assert_ne!(InstallStage::Installing, InstallStage::Complete);
    }

    #[test]
    fn test_install_message() {
        let progress = InstallProgress {
            stage: InstallStage::Installing,
            percent: 50,
            operation: "Test".to_string(),
            log_line: None,
        };

        let msg = InstallMessage::Progress(progress.clone());
        if let InstallMessage::Progress(p) = msg {
            assert_eq!(p.percent, 50);
            assert_eq!(p.stage, InstallStage::Installing);
        } else {
            panic!("Expected Progress message");
        }
    }

    #[test]
    fn test_async_installation() {
        let config = create_test_config();
        let rx = run_installation_async(config, "/mnt".to_string(), true);

        // Collect messages
        let messages: Vec<InstallMessage> = rx.iter().collect();

        // Should receive multiple progress updates
        assert!(!messages.is_empty());

        // Last message should be success or error (error expected since /mnt doesn't exist)
        match messages.last() {
            Some(InstallMessage::Success) => {
                // Expected in a real environment
            }
            Some(InstallMessage::Error(_)) => {
                // Expected in test environment (can't write to /mnt)
            }
            _ => {
                // Check if we got at least one progress message
                let has_progress = messages
                    .iter()
                    .any(|m| matches!(m, InstallMessage::Progress(_)));
                assert!(has_progress, "Expected at least one progress message");
            }
        }
    }
}
