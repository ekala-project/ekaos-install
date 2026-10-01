//! Reusable UI components
//!
//! This module contains all reusable UI components used throughout the application.
//! Components use gpui's `Render` (stateful) or `RenderOnce` (stateless) traits.

pub mod button;
pub mod checkbox;
pub mod dialog;
pub mod filterable_select;
pub mod help;
pub mod input;
pub mod message;
pub mod progress;
pub mod select;

// Re-exports
pub use button::{Button, ButtonStyle};
pub use checkbox::CheckboxList;
pub use dialog::ConfirmDialog;
pub use filterable_select::FilterableSelectList;
pub use help::HelpPanel;
pub use input::InputField;
pub use message::{MessageType, StatusMessage};
pub use progress::{ProgressBar, Spinner};
pub use select::SelectList;

/// Trait for components that can be validated
pub trait Validatable {
    /// Validate the component's current value.
    /// Returns `None` if valid, or `Some(error_message)` if invalid.
    fn validate(&self) -> Option<String>;

    /// Check if the component is currently valid.
    fn is_valid(&self) -> bool {
        self.validate().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        value: String,
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
    fn test_validatable_invalid() {
        let component = TestComponent {
            value: String::new(),
        };

        assert!(!component.is_valid());
        assert_eq!(
            component.validate(),
            Some("Value cannot be empty".to_string())
        );
    }

    #[test]
    fn test_validatable_valid() {
        let component = TestComponent {
            value: "test".to_string(),
        };

        assert!(component.is_valid());
        assert_eq!(component.validate(), None);
    }
}
