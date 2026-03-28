//! Error types for the ekaos-install application.
//!
//! This module defines all custom error types using thiserror for domain-specific errors.
//! The application uses anyhow::Result for most function returns.

use std::fmt;
use thiserror::Error;

/// Main error type for the installer application
#[derive(Debug, Error)]
pub enum InstallerError {
    /// Errors related to disk operations
    #[error("Disk error: {0}")]
    Disk(#[from] DiskError),

    /// Errors related to partition operations
    #[error("Partition error: {0}")]
    Partition(#[from] PartitionError),

    /// Errors related to filesystem operations
    #[error("Filesystem error: {0}")]
    Filesystem(#[from] FilesystemError),

    /// Errors related to NixOS configuration
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Errors related to system command execution
    #[error("Command execution error: {0}")]
    Command(#[from] CommandError),

    /// Errors related to the TUI
    #[error("UI error: {0}")]
    Ui(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other errors
    #[error("{0}")]
    Other(String),
}

/// Errors related to disk detection and management
#[derive(Debug, Error)]
pub enum DiskError {
    #[error("Disk '{0}' not found")]
    NotFound(String),

    #[error("Failed to detect disks: {0}")]
    DetectionFailed(String),

    #[error("Failed to parse disk information: {0}")]
    ParseError(String),

    #[error("Disk '{0}' is currently in use")]
    DiskBusy(String),

    #[error("Insufficient disk space: {available} available, {required} required")]
    InsufficientSpace { available: u64, required: u64 },
}

/// Errors related to partition operations
#[derive(Debug, Error)]
pub enum PartitionError {
    #[error("Failed to create partition table on '{device}': {reason}")]
    CreateTableFailed { device: String, reason: String },

    #[error("Failed to create partition: {0}")]
    CreateFailed(String),

    #[error("Invalid partition layout: {0}")]
    InvalidLayout(String),

    #[error("Partition '{0}' not found")]
    NotFound(String),
}

/// Errors related to filesystem operations
#[derive(Debug, Error)]
pub enum FilesystemError {
    #[error("Failed to format partition '{partition}' as {fstype}: {reason}")]
    FormatFailed {
        partition: String,
        fstype: String,
        reason: String,
    },

    #[error("Failed to mount '{device}' at '{mountpoint}': {reason}")]
    MountFailed {
        device: String,
        mountpoint: String,
        reason: String,
    },

    #[error("Failed to unmount '{0}': {1}")]
    UnmountFailed(String, String),

    #[error("Unsupported filesystem type: {0}")]
    UnsupportedFilesystem(String),
}

/// Errors related to NixOS configuration generation
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration generation failed: {0}")]
    GenerationFailed(String),

    #[error("Invalid configuration: {0}")]
    ValidationFailed(String),

    #[error("Failed to write configuration to '{path}': {reason}")]
    WriteFailed { path: String, reason: String },

    #[error("Failed to read configuration from '{path}': {reason}")]
    ReadFailed { path: String, reason: String },
}

/// Errors related to command execution
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Command '{command}' failed with exit code {code}: {stderr}")]
    ExecutionFailed {
        command: String,
        code: i32,
        stderr: String,
    },

    #[error("Command '{0}' not found")]
    NotFound(String),

    #[error("Command '{command}' timed out after {timeout}s")]
    Timeout { command: String, timeout: u64 },

    #[error("Invalid command arguments: {0}")]
    InvalidArguments(String),
}

/// Result type alias using anyhow::Error
pub type Result<T> = anyhow::Result<T>;

impl InstallerError {
    /// Create a UI error
    pub fn ui(msg: impl Into<String>) -> Self {
        Self::Ui(msg.into())
    }

    /// Create an other error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            InstallerError::Command(_) | InstallerError::Io(_) | InstallerError::Other(_)
        )
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            InstallerError::Disk(e) => format!("Disk operation failed: {}", e),
            InstallerError::Partition(e) => format!("Partitioning failed: {}", e),
            InstallerError::Filesystem(e) => format!("Filesystem operation failed: {}", e),
            InstallerError::Config(e) => format!("Configuration error: {}", e),
            InstallerError::Command(e) => format!("System command failed: {}", e),
            InstallerError::Ui(msg) => msg.clone(),
            InstallerError::Io(e) => format!("I/O error: {}", e),
            InstallerError::Other(msg) => msg.clone(),
        }
    }

    /// Get suggested action for the user
    pub fn suggestion(&self) -> Option<String> {
        match self {
            InstallerError::Disk(DiskError::NotFound(disk)) => {
                Some(format!("Please check that disk '{}' is connected and detected by the system. Run 'lsblk' to see available disks.", disk))
            }
            InstallerError::Disk(DiskError::DiskBusy(_)) => {
                Some("The disk may be mounted or in use. Try unmounting all partitions first.".to_string())
            }
            InstallerError::Disk(DiskError::InsufficientSpace { .. }) => {
                Some("Please select a disk with more available space or reduce the swap size.".to_string())
            }
            InstallerError::Command(CommandError::NotFound(cmd)) => {
                Some(format!("The command '{}' is not available. Make sure you're running from NixOS installation media.", cmd))
            }
            InstallerError::Filesystem(FilesystemError::MountFailed { .. }) => {
                Some("Ensure the filesystem was created successfully and the mount point exists.".to_string())
            }
            _ => None,
        }
    }
}

impl fmt::Display for InstallerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}
