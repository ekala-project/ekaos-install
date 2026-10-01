//! Input handling utilities
//!
//! Input handling is now done via gpui actions defined in `crate::ui`.
//! The old KeyCode bridge is no longer needed because gpui dispatches
//! typed action structs directly from its own key-binding system.
