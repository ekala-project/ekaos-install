//! NixOS-specific operations
//!
//! This module contains all NixOS-specific functionality including
//! disk detection, partitioning, filesystem operations, and installation.

pub mod config;
pub mod disk;
pub mod image;
pub mod install;

pub use config::{generate_configuration, write_configuration};
pub use disk::{detect_disks, Disk, DiskType, PartitionInfo};
pub use image::{write_image, ImageFormat};
pub use install::{
    run_fast_installation_async, run_installation_async, InstallExecutor, InstallMessage,
    InstallProgress, InstallStage, MockInstallExecutor, RealInstallExecutor,
};

// Placeholder for future modules
// pub mod partition;
// pub mod filesystem;
