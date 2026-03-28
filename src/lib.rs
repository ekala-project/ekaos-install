//! Ekaos Install - NixOS Installation TUI
//!
//! A terminal user interface application that guides users through installing NixOS.
//!
//! # Modules
//!
//! - `app`: Application state and wizard flow control
//! - `ui`: TUI components and screens
//! - `system`: System command execution and validation
//! - `nixos`: NixOS-specific operations (disk, partition, install)
//! - `error`: Error types and handling

#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

pub mod app;
pub mod error;
pub mod nixos;
pub mod system;
pub mod ui;

pub use error::{InstallerError, Result};

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
