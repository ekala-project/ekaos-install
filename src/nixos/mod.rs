//! NixOS-specific operations
//!
//! This module contains all NixOS-specific functionality including
//! disk detection, partitioning, filesystem operations, and installation.

pub mod config;
pub mod disk;
pub mod install;

pub use config::{generate_configuration, write_configuration};
pub use disk::{Disk, DiskType, PartitionInfo, detect_disks};
pub use install::{
    InstallExecutor, InstallMessage, InstallProgress, InstallStage, MockInstallExecutor,
    RealInstallExecutor, run_installation_async,
};

// Placeholder for future modules
// pub mod partition;
// pub mod filesystem;
