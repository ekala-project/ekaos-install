//! Utility functions for UI components
//!
//! This module provides common utilities used across screens and components.

pub mod input;
pub mod navigation;

pub use input::keycode_to_input_event;
pub use navigation::render_navigation_hints;
