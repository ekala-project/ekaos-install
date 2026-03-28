//! Application state and flow control
//!
//! This module manages the wizard's state machine, screen transitions,
//! and overall application flow.

pub mod state;

pub use state::{App, AppMode, Screen};
