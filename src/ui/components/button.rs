//! Button component
//!
//! Interactive button widget with focus and click support

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use super::{Component, Focusable, InputEvent, Interactive};

/// Button widget
pub struct Button {
    /// Button label
    label: String,
    /// Whether the button is focused
    focused: bool,
    /// Whether the button is enabled
    enabled: bool,
    /// Button style (Primary, Secondary, Danger)
    style: ButtonStyle,
}

/// Button visual style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Primary action button (cyan)
    Primary,
    /// Secondary action button (white)
    Secondary,
    /// Dangerous action button (red)
    Danger,
    /// Success action button (green)
    Success,
}

impl Button {
    /// Create a new button
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            focused: false,
            enabled: true,
            style: ButtonStyle::Primary,
        }
    }

    /// Set the button style
    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get the base color for this button style
    fn base_color(&self) -> Color {
        match self.style {
            ButtonStyle::Primary => Color::Cyan,
            ButtonStyle::Secondary => Color::White,
            ButtonStyle::Danger => Color::Red,
            ButtonStyle::Success => Color::Green,
        }
    }
}

impl Focusable for Button {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Interactive for Button {
    fn handle_input(&mut self, event: InputEvent) -> bool {
        matches!(event, InputEvent::Enter)
    }
}

impl Component for Button {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let color = if !self.enabled {
            Color::DarkGray
        } else {
            self.base_color()
        };

        let style = if self.focused {
            Style::default()
                .fg(Color::Black)
                .bg(color)
                .add_modifier(Modifier::BOLD)
        } else if self.enabled {
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = if self.focused {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        } else {
            Block::default().borders(Borders::ALL).border_style(
                Style::default().fg(if self.enabled { color } else { Color::DarkGray }),
            )
        };

        let label = format!(" {} ", self.label);
        let paragraph = Paragraph::new(Line::from(label))
            .block(block)
            .style(style)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new("Click Me");
        assert_eq!(button.label, "Click Me");
        assert!(!button.is_focused());
        assert!(button.is_enabled());
    }

    #[test]
    fn test_button_focus() {
        let mut button = Button::new("Test");
        assert!(!button.is_focused());

        button.set_focused(true);
        assert!(button.is_focused());
    }

    #[test]
    fn test_button_input() {
        let mut button = Button::new("Test");
        button.set_focused(true);

        // Enter should be handled
        assert!(button.handle_input(InputEvent::Enter));

        // Other keys should not
        assert!(!button.handle_input(InputEvent::Char('a')));
    }

    #[test]
    fn test_button_styles() {
        let primary = Button::new("Primary");
        assert_eq!(primary.base_color(), Color::Cyan);

        let danger = Button::new("Delete").with_style(ButtonStyle::Danger);
        assert_eq!(danger.base_color(), Color::Red);
    }
}
