//! Text input field component with validation
//!
//! Stateful `Render` component that implements a full text input from scratch,
//! since gpui has no built-in text input widget.
//!
//! Supports:
//! - Text entry and editing with cursor movement
//! - Input validation with error display
//! - Password mode (bullet masking)
//! - Focus management via `FocusHandle`

use gpui::prelude::*;
use gpui::{div, px, FocusHandle, FontWeight, IntoElement, SharedString};

use crate::ui::theme::{spacing, AppTheme};

use super::Validatable;

/// Validation function type: takes a string value, returns `None` if valid
/// or `Some(error_message)` if invalid.
type ValidatorFn = Box<dyn Fn(&str) -> Option<String>>;

/// Text input field with validation support.
///
/// This is a **stateful** component that implements `gpui::Render`.
/// It owns a `FocusHandle` and manages cursor position, value, and error state.
pub struct InputField {
    /// Label for the input field
    label: String,
    /// Current value
    value: String,
    /// Cursor position (character index)
    cursor: usize,
    /// gpui focus handle
    focus_handle: FocusHandle,
    /// Whether the field is enabled
    enabled: bool,
    /// Validation function
    validator: Option<ValidatorFn>,
    /// Cached validation error
    error: Option<String>,
    /// Placeholder text
    placeholder: Option<String>,
    /// Maximum length (None = unlimited)
    max_length: Option<usize>,
    /// Whether to hide input (password field)
    password: bool,
}

impl InputField {
    /// Create a new input field.
    pub fn new(label: impl Into<String>, cx: &mut gpui::App) -> Self {
        Self {
            label: label.into(),
            value: String::new(),
            cursor: 0,
            focus_handle: cx.focus_handle(),
            enabled: true,
            validator: None,
            error: None,
            placeholder: None,
            max_length: None,
            password: false,
        }
    }

    /// Set the initial value.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor = self.value.len();
        self
    }

    /// Set placeholder text.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set maximum length.
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Enable password mode (bullet masking).
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Set a validation function.
    pub fn with_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Option<String> + 'static,
    {
        self.validator = Some(Box::new(validator));
        self
    }

    /// Set enabled state.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get the current value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set the value programmatically.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.cursor.min(self.value.len());
        self.revalidate();
    }

    /// Clear the input.
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
        self.error = None;
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        if let Some(max) = self.max_length {
            if self.value.len() >= max {
                return;
            }
        }
        self.value.insert(self.cursor, c);
        self.cursor += 1;
        self.revalidate();
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.value.remove(self.cursor - 1);
            self.cursor -= 1;
            self.revalidate();
        }
    }

    /// Delete the character at the cursor (forward delete).
    pub fn delete_char_forward(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
            self.revalidate();
        }
    }

    /// Move cursor left.
    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right.
    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.value.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to the start.
    pub fn move_cursor_home(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to the end.
    pub fn move_cursor_end(&mut self) {
        self.cursor = self.value.len();
    }

    /// Revalidate the current value and cache the error.
    pub fn revalidate(&mut self) {
        self.error = self.validate();
    }

    /// Get the display text (masked for password fields).
    pub fn display_text(&self) -> String {
        if self.password && !self.value.is_empty() {
            "\u{2022}".repeat(self.value.len())
        } else if self.value.is_empty() {
            self.placeholder.as_deref().unwrap_or("").to_string()
        } else {
            self.value.clone()
        }
    }

    /// Handle a gpui key down event.
    fn handle_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, Self>,
    ) {
        if !self.enabled {
            return;
        }

        let keystroke = &event.keystroke;

        match keystroke.key.as_str() {
            "backspace" => {
                self.delete_char();
                cx.notify();
            }
            "delete" => {
                self.delete_char_forward();
                cx.notify();
            }
            "left" => {
                self.move_cursor_left();
                cx.notify();
            }
            "right" => {
                self.move_cursor_right();
                cx.notify();
            }
            "home" => {
                self.move_cursor_home();
                cx.notify();
            }
            "end" => {
                self.move_cursor_end();
                cx.notify();
            }
            _ => {
                // Handle character input via ime_key
                if let Some(text) = &keystroke.key_char {
                    for c in text.chars() {
                        self.insert_char(c);
                    }
                    cx.notify();
                }
            }
        }
    }
}

impl Validatable for InputField {
    fn validate(&self) -> Option<String> {
        if let Some(validator) = &self.validator {
            validator(&self.value)
        } else {
            None
        }
    }
}

impl Render for InputField {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, Self>,
    ) -> impl IntoElement {
        let theme = AppTheme::new();
        let is_focused = self.focus_handle.is_focused(window);

        // Determine border color based on state
        let border_color = if !self.enabled {
            theme.muted
        } else if self.error.is_some() {
            theme.error
        } else if is_focused {
            theme.primary
        } else {
            theme.border
        };

        let display = self.display_text();
        let show_placeholder = self.value.is_empty() && self.placeholder.is_some();

        // Cursor position in display characters
        let cursor_pos = if self.password {
            self.cursor.min(self.value.len())
        } else {
            self.cursor
        };

        // Build the text display with cursor
        let text_element = if is_focused && self.enabled {
            let chars: Vec<char> = display.chars().collect();
            let before: String = chars[..cursor_pos.min(chars.len())].iter().collect();
            let cursor_char = chars.get(cursor_pos).copied().unwrap_or(' ');
            let after: String = if cursor_pos < chars.len() {
                chars[cursor_pos + 1..].iter().collect()
            } else {
                String::new()
            };

            div()
                .flex()
                .flex_row()
                .child(
                    div()
                        .text_color(if show_placeholder {
                            theme.muted
                        } else {
                            theme.foreground
                        })
                        .child(SharedString::from(before)),
                )
                .child(
                    // Cursor block: inverted colors
                    div()
                        .bg(theme.primary)
                        .text_color(gpui::hsla(0.0, 0.0, 0.0, 1.0))
                        .child(SharedString::from(cursor_char.to_string())),
                )
                .child(
                    div()
                        .text_color(if show_placeholder {
                            theme.muted
                        } else {
                            theme.foreground
                        })
                        .child(SharedString::from(after)),
                )
        } else {
            // Not focused: just show the text
            let text_color = if !self.enabled || show_placeholder {
                theme.muted
            } else {
                theme.foreground
            };

            div().child(
                div()
                    .text_color(text_color)
                    .child(SharedString::from(display)),
            )
        };

        // Build the error element (if any)
        let error_label: SharedString = self
            .error
            .as_ref()
            .map(|e| format!("{} {}", "\u{2717}", e))
            .unwrap_or_default()
            .into();

        let label_text: SharedString = self.label.clone().into();
        let label_weight = if is_focused {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        };

        let focus_handle = self.focus_handle.clone();

        div()
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .gap(px(spacing::SMALL))
            // Label
            .child(
                div()
                    .text_color(border_color)
                    .text_sm()
                    .font_weight(label_weight)
                    .child(label_text),
            )
            // Input box
            .child(
                div()
                    .w_full()
                    .border_1()
                    .border_color(border_color)
                    .rounded(px(4.0))
                    .px(px(spacing::MEDIUM))
                    .py(px(spacing::SMALL))
                    .child(text_element),
            )
            // Error message
            .when(self.error.is_some(), |el| {
                el.child(div().text_color(theme.error).text_sm().child(error_label))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::MaybeUninit;

    /// Test wrapper for InputField that avoids needing a real gpui App.
    ///
    /// We only test pure-state methods (insert_char, cursor movement, etc.)
    /// that never touch the `focus_handle`. The wrapper uses `ManuallyDrop`
    /// so the invalid handle is never dropped.
    struct TestField {
        inner: std::mem::ManuallyDrop<InputField>,
    }

    impl TestField {
        #[allow(invalid_value)]
        fn new(label: &str) -> Self {
            let field = InputField {
                label: label.to_string(),
                value: String::new(),
                cursor: 0,
                focus_handle: unsafe { MaybeUninit::zeroed().assume_init() },
                enabled: true,
                validator: None,
                error: None,
                placeholder: None,
                max_length: None,
                password: false,
            };
            Self {
                inner: std::mem::ManuallyDrop::new(field),
            }
        }

        fn with_value(mut self, value: &str) -> Self {
            self.inner.value = value.to_string();
            self.inner.cursor = value.len();
            self
        }
    }

    impl std::ops::Deref for TestField {
        type Target = InputField;
        fn deref(&self) -> &InputField {
            &self.inner
        }
    }

    impl std::ops::DerefMut for TestField {
        fn deref_mut(&mut self) -> &mut InputField {
            &mut self.inner
        }
    }

    #[test]
    fn test_input_field_creation() {
        let input = TestField::new("Username");
        assert_eq!(input.value(), "");
        assert_eq!(input.label, "Username");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn test_input_field_with_value() {
        let input = TestField::new("Username").with_value("test");
        assert_eq!(input.value(), "test");
        assert_eq!(input.cursor, 4);
    }

    #[test]
    fn test_insert_char() {
        let mut input = TestField::new("Test");
        input.insert_char('a');
        assert_eq!(input.value(), "a");
        input.insert_char('b');
        assert_eq!(input.value(), "ab");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn test_backspace() {
        let mut input = TestField::new("Test").with_value("abc");
        input.delete_char();
        assert_eq!(input.value(), "ab");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn test_delete_forward() {
        let mut input = TestField::new("Test").with_value("abc");
        input.move_cursor_home();
        input.delete_char_forward();
        assert_eq!(input.value(), "bc");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn test_max_length() {
        let mut input = TestField::new("Test");
        input.max_length = Some(3);
        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');
        input.insert_char('d'); // Should be rejected
        assert_eq!(input.value(), "abc");
    }

    #[test]
    fn test_validation() {
        let mut input = TestField::new("Email");
        input.validator = Some(Box::new(|value| {
            if value.contains('@') {
                None
            } else {
                Some("Must contain @".to_string())
            }
        }));

        assert!(!input.is_valid());

        input.value = "test@example.com".to_string();
        input.cursor = input.value.len();
        input.revalidate();
        assert!(input.is_valid());
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = TestField::new("Test").with_value("hello");

        input.move_cursor_home();
        assert_eq!(input.cursor, 0);

        input.move_cursor_end();
        assert_eq!(input.cursor, 5);

        input.move_cursor_left();
        assert_eq!(input.cursor, 4);

        input.move_cursor_right();
        assert_eq!(input.cursor, 5);

        // Should not go past end
        input.move_cursor_right();
        assert_eq!(input.cursor, 5);

        // Should not go past start
        input.move_cursor_home();
        input.move_cursor_left();
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn test_display_text_normal() {
        let input = TestField::new("Test").with_value("hello");
        assert_eq!(input.display_text(), "hello");
    }

    #[test]
    fn test_display_text_password() {
        let mut input = TestField::new("Password").with_value("abc");
        input.password = true;
        assert_eq!(input.display_text(), "\u{2022}\u{2022}\u{2022}");
    }

    #[test]
    fn test_display_text_placeholder() {
        let mut input = TestField::new("Test");
        input.placeholder = Some("Type here...".to_string());
        assert_eq!(input.display_text(), "Type here...");
    }

    #[test]
    fn test_display_text_empty_no_placeholder() {
        let input = TestField::new("Test");
        assert_eq!(input.display_text(), "");
    }

    #[test]
    fn test_set_value() {
        let mut input = TestField::new("Test").with_value("hello");
        input.set_value("world");
        assert_eq!(input.value(), "world");
        assert_eq!(input.cursor, 5);
    }

    #[test]
    fn test_set_value_shorter() {
        let mut input = TestField::new("Test").with_value("hello");
        input.set_value("hi");
        assert_eq!(input.value(), "hi");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn test_clear() {
        let mut input = TestField::new("Test").with_value("hello");
        input.clear();
        assert_eq!(input.value(), "");
        assert_eq!(input.cursor, 0);
        assert!(input.error.is_none());
    }

    #[test]
    fn test_insert_at_middle() {
        let mut input = TestField::new("Test").with_value("ac");
        input.cursor = 1;
        input.insert_char('b');
        assert_eq!(input.value(), "abc");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn test_backspace_at_start() {
        let mut input = TestField::new("Test").with_value("abc");
        input.cursor = 0;
        input.delete_char();
        assert_eq!(input.value(), "abc");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn test_delete_forward_at_end() {
        let mut input = TestField::new("Test").with_value("abc");
        input.delete_char_forward();
        assert_eq!(input.value(), "abc");
    }
}
