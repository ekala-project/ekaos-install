//! System operations and command execution
//!
//! This module provides safe wrappers around system commands
//! and handles mock vs real execution modes.

pub mod bootmode;
pub mod command;
pub mod detection;
pub mod network;

pub use bootmode::{BootMode, detect_boot_mode};
pub use command::{CommandExecutor, MockExecutor, RealExecutor};
pub use detection::{SystemInfo, detect_system, is_nixos, is_root};
pub use network::{NetworkStatus, check_network, check_network_with_timeout};
