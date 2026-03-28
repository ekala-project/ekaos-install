//! NixOS-specific operations
//!
//! This module contains all NixOS-specific functionality including
//! disk detection, partitioning, filesystem operations, and installation.

pub mod disk;

pub use disk::{detect_disks, Disk, DiskType};

// Placeholder for future modules
// pub mod partition;
// pub mod filesystem;
// pub mod config;
// pub mod install;
