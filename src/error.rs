//! Error types for the ekaos-install application.
//!
//! This module defines all custom error types using thiserror for domain-specific errors.
//! The application uses anyhow::Result for most function returns.

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
    /// Disk was not found
    #[error("Disk '{0}' not found")]
    NotFound(String),

    /// Failed to detect available disks
    #[error("Failed to detect disks: {0}")]
    DetectionFailed(String),

    /// Failed to parse disk information
    #[error("Failed to parse disk information: {0}")]
    ParseError(String),

    /// Disk is currently in use and cannot be modified
    #[error("Disk '{0}' is currently in use")]
    DiskBusy(String),

    /// Insufficient disk space for installation
    #[error("Insufficient disk space: {available} available, {required} required")]
    InsufficientSpace {
        /// Available disk space in bytes
        available: u64,
        /// Required disk space in bytes
        required: u64,
    },
}

/// Errors related to partition operations
#[derive(Debug, Error)]
pub enum PartitionError {
    /// Failed to create partition table
    #[error("Failed to create partition table on '{device}': {reason}")]
    CreateTableFailed {
        /// Device path
        device: String,
        /// Failure reason
        reason: String,
    },

    /// Failed to create a partition
    #[error("Failed to create partition: {0}")]
    CreateFailed(String),

    /// Invalid partition layout configuration
    #[error("Invalid partition layout: {0}")]
    InvalidLayout(String),

    /// Partition was not found
    #[error("Partition '{0}' not found")]
    NotFound(String),
}

/// Errors related to filesystem operations
#[derive(Debug, Error)]
pub enum FilesystemError {
    /// Failed to format a partition
    #[error("Failed to format partition '{partition}' as {fstype}: {reason}")]
    FormatFailed {
        /// Partition path
        partition: String,
        /// Filesystem type
        fstype: String,
        /// Failure reason
        reason: String,
    },

    /// Failed to mount a device
    #[error("Failed to mount '{device}' at '{mountpoint}': {reason}")]
    MountFailed {
        /// Device path
        device: String,
        /// Mount point path
        mountpoint: String,
        /// Failure reason
        reason: String,
    },

    /// Failed to unmount a device
    #[error("Failed to unmount '{0}': {1}")]
    UnmountFailed(String, String),

    /// Unsupported filesystem type
    #[error("Unsupported filesystem type: {0}")]
    UnsupportedFilesystem(String),
}

/// Errors related to NixOS configuration generation
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration generation failed
    #[error("Configuration generation failed: {0}")]
    GenerationFailed(String),

    /// Configuration validation failed
    #[error("Invalid configuration: {0}")]
    ValidationFailed(String),

    /// Failed to write configuration file
    #[error("Failed to write configuration to '{path}': {reason}")]
    WriteFailed {
        /// Configuration file path
        path: String,
        /// Failure reason
        reason: String,
    },

    /// Failed to read configuration file
    #[error("Failed to read configuration from '{path}': {reason}")]
    ReadFailed {
        /// Configuration file path
        path: String,
        /// Failure reason
        reason: String,
    },
}

/// Errors related to command execution
#[derive(Debug, Error)]
pub enum CommandError {
    /// Command execution failed
    #[error("Command '{command}' failed with exit code {code}: {stderr}")]
    ExecutionFailed {
        /// Command name
        command: String,
        /// Exit code
        code: i32,
        /// Standard error output
        stderr: String,
    },

    /// Command was not found
    #[error("Command '{0}' not found")]
    NotFound(String),

    /// Command execution timed out
    #[error("Command '{command}' timed out after {timeout}s")]
    Timeout {
        /// Command name
        command: String,
        /// Timeout in seconds
        timeout: u64,
    },

    /// Invalid command arguments
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
            InstallerError::Disk(DiskError::NotFound(disk)) => Some(format!(
                "Please check that disk '{}' is connected and detected by the system. Run 'lsblk' to see available disks.",
                disk
            )),
            InstallerError::Disk(DiskError::DiskBusy(_)) => Some(
                "The disk may be mounted or in use. Try unmounting all partitions first."
                    .to_string(),
            ),
            InstallerError::Disk(DiskError::InsufficientSpace { .. }) => Some(
                "Please select a disk with more available space or reduce the swap size."
                    .to_string(),
            ),
            InstallerError::Command(CommandError::NotFound(cmd)) => Some(format!(
                "The command '{}' is not available. Make sure you're running from NixOS installation media.",
                cmd
            )),
            InstallerError::Filesystem(FilesystemError::MountFailed { .. }) => Some(
                "Ensure the filesystem was created successfully and the mount point exists."
                    .to_string(),
            ),
            _ => None,
        }
    }
}
