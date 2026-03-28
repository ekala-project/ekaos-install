//! System operations and command execution
//!
//! This module provides safe wrappers around system commands
//! and handles mock vs real execution modes.

pub mod command;

pub use command::{CommandExecutor, MockExecutor, RealExecutor};
