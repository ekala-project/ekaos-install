//! Text input field component with validation
//!
//! Provides a text input field that supports:
//! - Text entry and editing
//! - Cursor movement
//! - Input validation
//! - Error display
//! - Focus management

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{Component, Focusable, InputEvent, Interactive, Validatable};

/// Text input field with validation support
pub struct InputField {
    /// Label for the input field
    label: String,
    /// Current value
    value: String,
    /// Cursor position
    cursor: usize,
    /// Whether the field is focused
    focused: bool,
    /// Whether the field is enabled
    enabled: bool,
    /// Validation function
    validator: Option<Box<dyn Fn(&str) -> Option<String>>>,
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
    /// Create a new input field
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: String::new(),
            cursor: 0,
            focused: false,
            enabled: true,
            validator: None,
            error: None,
            placeholder: None,
            max_length: None,
            password: false,
        }
    }

    /// Set the initial value
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor = self.value.len();
        self
    }

    /// Set a placeholder text
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set maximum length
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set as password field (hide input)
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Set a validation function
    pub fn with_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Option<String> + 'static,
    {
        self.validator = Some(Box::new(validator));
        self
    }

    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get the current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set the value programmatically
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.len().min(self.cursor);
        self.revalidate();
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
        self.error = None;
    }

    /// Insert a character at the cursor position
    fn insert_char(&mut self, c: char) {
        if let Some(max) = self.max_length {
            if self.value.len() >= max {
                return;
            }
        }

        self.value.insert(self.cursor, c);
        self.cursor += 1;
        self.revalidate();
    }

    /// Delete the character before the cursor
    fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.value.remove(self.cursor - 1);
            self.cursor -= 1;
            self.revalidate();
        }
    }

    /// Delete the character at the cursor
    fn delete_char_forward(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
            self.revalidate();
        }
    }

    /// Move cursor left
    fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    fn move_cursor_right(&mut self) {
        if self.cursor < self.value.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start
    fn move_cursor_home(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end
    fn move_cursor_end(&mut self) {
        self.cursor = self.value.len();
    }

    /// Revalidate the current value
    fn revalidate(&mut self) {
        self.error = self.validate();
    }

    /// Get display text (masked for password fields)
    fn display_text(&self) -> String {
        if self.password && !self.value.is_empty() {
            "•".repeat(self.value.len())
        } else if self.value.is_empty() {
            self.placeholder.as_deref().unwrap_or("").to_string()
        } else {
            self.value.clone()
        }
    }
}

impl Focusable for InputField {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if focused {
            self.on_focus();
        } else {
            self.on_blur();
        }
    }

    fn on_blur(&mut self) {
        self.revalidate();
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

impl Interactive for InputField {
    fn handle_input(&mut self, event: InputEvent) -> bool {
        if !self.enabled {
            return false;
        }

        match event {
            InputEvent::Char(c) => {
                self.insert_char(c);
                true
            }
            InputEvent::Backspace => {
                self.delete_char();
                true
            }
            InputEvent::Delete => {
                self.delete_char_forward();
                true
            }
            InputEvent::Left => {
                self.move_cursor_left();
                true
            }
            InputEvent::Right => {
                self.move_cursor_right();
                true
            }
            InputEvent::Home => {
                self.move_cursor_home();
                true
            }
            InputEvent::End => {
                self.move_cursor_end();
                true
            }
            _ => false,
        }
    }
}

impl Component for InputField {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let display_text = self.display_text();

        // Determine border color based on state
        let border_color = if !self.enabled {
            Color::DarkGray
        } else if self.error.is_some() {
            Color::Red
        } else if self.focused {
            Color::Cyan
        } else {
            Color::White
        };

        // Create border style
        let border_style = if self.focused {
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(border_color)
        };

        // Build the block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.label.as_str())
            .border_style(border_style);

        // Create the text display
        let text_style = if !self.enabled {
            Style::default().fg(Color::DarkGray)
        } else if self.value.is_empty() && self.placeholder.is_some() {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(Color::White)
        };

        // Show cursor if focused
        // For password fields, we need to use character count, not byte position
        let cursor_char_pos = if self.password {
            // Cursor tracks actual value position, but display uses bullet characters
            self.cursor.min(self.value.len())
        } else {
            self.cursor
        };

        let display_char_count = display_text.chars().count();

        let text_line = if self.focused && cursor_char_pos == display_char_count {
            Line::from(vec![
                Span::styled(display_text, text_style),
                Span::styled("█", Style::default().fg(Color::Cyan)),
            ])
        } else if self.focused && cursor_char_pos < display_char_count {
            // Split at character boundary, not byte boundary
            let before: String = display_text.chars().take(cursor_char_pos).collect();
            let cursor_char = display_text.chars().nth(cursor_char_pos).unwrap_or(' ');
            let rest: String = display_text.chars().skip(cursor_char_pos + 1).collect();

            Line::from(vec![
                Span::styled(before, text_style),
                Span::styled(
                    cursor_char.to_string(),
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ),
                Span::styled(rest, text_style),
            ])
        } else {
            Line::from(Span::styled(display_text, text_style))
        };

        let paragraph = Paragraph::new(text_line).block(block);

        frame.render_widget(paragraph, area);

        // Render error message if present
        if let Some(error) = &self.error {
            if area.height > 3 {
                let error_area = Rect {
                    x: area.x + 2,
                    y: area.y + area.height - 1,
                    width: area.width.saturating_sub(4),
                    height: 1,
                };

                let error_line = Line::from(Span::styled(
                    format!("✗ {}", error),
                    Style::default().fg(Color::Red),
                ));

                frame.render_widget(Paragraph::new(error_line), error_area);
            }
        }
    }

    fn title(&self) -> Option<&str> {
        Some(&self.label)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_field_creation() {
        let input = InputField::new("Username");
        assert_eq!(input.value(), "");
        assert_eq!(input.label, "Username");
        assert!(!input.is_focused());
    }

    #[test]
    fn test_input_field_with_value() {
        let input = InputField::new("Username").with_value("test");
        assert_eq!(input.value(), "test");
        assert_eq!(input.cursor, 4);
    }

    #[test]
    fn test_input_char() {
        let mut input = InputField::new("Test");
        input.set_focused(true);

        input.handle_input(InputEvent::Char('a'));
        assert_eq!(input.value(), "a");

        input.handle_input(InputEvent::Char('b'));
        assert_eq!(input.value(), "ab");
    }

    #[test]
    fn test_input_backspace() {
        let mut input = InputField::new("Test").with_value("abc");
        input.handle_input(InputEvent::Backspace);
        assert_eq!(input.value(), "ab");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn test_input_max_length() {
        let mut input = InputField::new("Test").with_max_length(3);
        input.handle_input(InputEvent::Char('a'));
        input.handle_input(InputEvent::Char('b'));
        input.handle_input(InputEvent::Char('c'));
        input.handle_input(InputEvent::Char('d'));

        assert_eq!(input.value(), "abc");
    }

    #[test]
    fn test_input_validation() {
        let input = InputField::new("Email").with_validator(|value| {
            if value.contains('@') {
                None
            } else {
                Some("Must contain @".to_string())
            }
        });

        assert!(!input.is_valid());

        let mut valid_input = InputField::new("Email")
            .with_value("test@example.com")
            .with_validator(|value| {
                if value.contains('@') {
                    None
                } else {
                    Some("Must contain @".to_string())
                }
            });

        valid_input.revalidate();
        assert!(valid_input.is_valid());
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = InputField::new("Test").with_value("hello");

        input.move_cursor_home();
        assert_eq!(input.cursor, 0);

        input.move_cursor_end();
        assert_eq!(input.cursor, 5);

        input.move_cursor_left();
        assert_eq!(input.cursor, 4);

        input.move_cursor_right();
        assert_eq!(input.cursor, 5);
    }
}
