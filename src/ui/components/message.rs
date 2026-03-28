//! Status message component
//!
//! Displays success, warning, error, or info messages

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::Component;

/// Message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Success message (green)
    Success,
    /// Warning message (yellow)
    Warning,
    /// Error message (red)
    Error,
    /// Info message (blue)
    Info,
}

impl MessageType {
    /// Get the color for this message type
    pub fn color(&self) -> Color {
        match self {
            MessageType::Success => Color::Green,
            MessageType::Warning => Color::Yellow,
            MessageType::Error => Color::Red,
            MessageType::Info => Color::Blue,
        }
    }

    /// Get the icon for this message type
    pub fn icon(&self) -> &'static str {
        match self {
            MessageType::Success => "✓",
            MessageType::Warning => "⚠",
            MessageType::Error => "✗",
            MessageType::Info => "ℹ",
        }
    }
}

/// Status message component
pub struct StatusMessage {
    /// Message text
    text: String,
    /// Message type
    message_type: MessageType,
    /// Whether to show border
    show_border: bool,
}

impl StatusMessage {
    /// Create a new status message
    pub fn new(message_type: MessageType, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            message_type,
            show_border: false,
        }
    }

    /// Show border around message
    pub fn with_border(mut self) -> Self {
        self.show_border = true;
        self
    }

    /// Create a success message
    pub fn success(text: impl Into<String>) -> Self {
        Self::new(MessageType::Success, text)
    }

    /// Create a warning message
    pub fn warning(text: impl Into<String>) -> Self {
        Self::new(MessageType::Warning, text)
    }

    /// Create an error message
    pub fn error(text: impl Into<String>) -> Self {
        Self::new(MessageType::Error, text)
    }

    /// Create an info message
    pub fn info(text: impl Into<String>) -> Self {
        Self::new(MessageType::Info, text)
    }
}

impl Component for StatusMessage {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let color = self.message_type.color();
        let icon = self.message_type.icon();

        let line = Line::from(vec![
            Span::styled(
                format!("{} ", icon),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(self.text.clone(), Style::default().fg(color)),
        ]);

        let paragraph = if self.show_border {
            Paragraph::new(line).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color)),
            )
        } else {
            Paragraph::new(line)
        };

        frame.render_widget(paragraph, area);
    }
}
