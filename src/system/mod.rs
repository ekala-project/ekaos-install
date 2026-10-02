//! System operations and command execution
//!
//! This module provides safe wrappers around system commands
//! and handles mock vs real execution modes.

pub mod bootmode;
pub mod command;
pub mod detection;
pub mod network;

pub use bootmode::{detect_boot_mode, BootMode};
pub use command::{CommandExecutor, MockExecutor, RealExecutor};
pub use detection::{detect_system, is_nixos, is_root, SystemInfo};
pub use network::{check_network, check_network_with_timeout, NetworkStatus};
