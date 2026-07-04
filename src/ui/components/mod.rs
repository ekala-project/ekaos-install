//! Reusable UI components
//!
//! This module contains all reusable UI components used throughout the application.
//! Components follow a consistent pattern with support for focus, validation, and keyboard input.

use ratatui::Frame;
use ratatui::layout::Rect;

pub mod button;
pub mod checkbox;
pub mod dialog;
pub mod filterable_select;
pub mod help;
pub mod input;
pub mod message;
pub mod progress;
pub mod select;

// Re-export commonly used components
pub use button::Button;
pub use checkbox::CheckboxList;
pub use dialog::ConfirmDialog;
pub use filterable_select::FilterableSelectList;
pub use help::HelpPanel;
pub use input::InputField;
pub use message::{MessageType, StatusMessage};
pub use progress::{ProgressBar, Spinner};
pub use select::SelectList;

/// Trait for components that can receive focus
pub trait Focusable {
    /// Check if the component is currently focused
    fn is_focused(&self) -> bool;

    /// Set the focus state
    fn set_focused(&mut self, focused: bool);

    /// Handle focus being gained
    fn on_focus(&mut self) {}

    /// Handle focus being lost
    fn on_blur(&mut self) {}
}

/// Trait for components that can be validated
pub trait Validatable {
    /// Validate the component's current value
    /// Returns None if valid, or Some(error_message) if invalid
    fn validate(&self) -> Option<String>;

    /// Check if the component is currently valid
    fn is_valid(&self) -> bool {
        self.validate().is_none()
    }
}

/// Trait for components that can be rendered
pub trait Component {
    /// Render the component to the given area
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect);

    /// Get the component's title/label (if any)
    fn title(&self) -> Option<&str> {
        None
    }

    /// Check if the component is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Keyboard input event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    /// Character input
    Char(char),
    /// Enter key
    Enter,
    /// Escape key
    Escape,
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Tab key
    Tab,
    /// Home key
    Home,
    /// End key
    End,
}

/// Trait for components that handle keyboard input
pub trait Interactive: Focusable {
    /// Handle a keyboard input event
    /// Returns true if the event was handled
    fn handle_input(&mut self, event: InputEvent) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        focused: bool,
        value: String,
    }

    impl Focusable for TestComponent {
        fn is_focused(&self) -> bool {
            self.focused
        }

        fn set_focused(&mut self, focused: bool) {
            self.focused = focused;
        }
    }

    impl Validatable for TestComponent {
        fn validate(&self) -> Option<String> {
            if self.value.is_empty() {
                Some("Value cannot be empty".to_string())
            } else {
                None
            }
        }
    }

    #[test]
    fn test_focusable() {
        let mut component = TestComponent {
            focused: false,
            value: String::new(),
        };

        assert!(!component.is_focused());
        component.set_focused(true);
        assert!(component.is_focused());
    }

    #[test]
    fn test_validatable() {
        let component = TestComponent {
            focused: false,
            value: String::new(),
        };

        assert!(!component.is_valid());
        assert_eq!(
            component.validate(),
            Some("Value cannot be empty".to_string())
        );

        let valid_component = TestComponent {
            focused: false,
            value: "test".to_string(),
        };

        assert!(valid_component.is_valid());
        assert_eq!(valid_component.validate(), None);
    }
}
