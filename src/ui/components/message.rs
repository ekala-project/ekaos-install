//! Status message component
//!
//! Stateless `RenderOnce` widget for displaying success, warning, error, or info messages.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, Hsla, IntoElement, SharedString};

use crate::ui::theme::{icons, spacing, AppTheme};

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
    /// Get the color for this message type from the theme.
    pub fn color(&self) -> Hsla {
        let theme = AppTheme::new();
        match self {
            MessageType::Success => theme.success,
            MessageType::Warning => theme.warning,
            MessageType::Error => theme.error,
            MessageType::Info => theme.info,
        }
    }

    /// Get the icon for this message type.
    pub fn icon(&self) -> &'static str {
        match self {
            MessageType::Success => icons::SUCCESS,
            MessageType::Warning => icons::WARNING,
            MessageType::Error => icons::ERROR,
            MessageType::Info => icons::INFO,
        }
    }
}

/// Status message component
#[derive(IntoElement)]
pub struct StatusMessage {
    /// Message text
    text: SharedString,
    /// Message type
    message_type: MessageType,
    /// Whether to show a border
    show_border: bool,
}

impl StatusMessage {
    /// Create a new status message.
    pub fn new(message_type: MessageType, text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            message_type,
            show_border: false,
        }
    }

    /// Show a border around the message.
    pub fn with_border(mut self) -> Self {
        self.show_border = true;
        self
    }

    /// Create a success message.
    pub fn success(text: impl Into<SharedString>) -> Self {
        Self::new(MessageType::Success, text)
    }

    /// Create a warning message.
    pub fn warning(text: impl Into<SharedString>) -> Self {
        Self::new(MessageType::Warning, text)
    }

    /// Create an error message.
    pub fn error(text: impl Into<SharedString>) -> Self {
        Self::new(MessageType::Error, text)
    }

    /// Create an info message.
    pub fn info(text: impl Into<SharedString>) -> Self {
        Self::new(MessageType::Info, text)
    }
}

impl RenderOnce for StatusMessage {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let color = self.message_type.color();
        let icon: SharedString = format!("{} ", self.message_type.icon()).into();

        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing::SMALL))
            .child(
                div()
                    .text_color(color)
                    .font_weight(FontWeight::BOLD)
                    .child(icon),
            )
            .child(div().text_color(color).child(self.text));

        if self.show_border {
            row = row
                .border_1()
                .border_color(color)
                .rounded(px(4.0))
                .p(px(spacing::MEDIUM));
        }

        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_colors() {
        let theme = AppTheme::new();

        let success_color = MessageType::Success.color();
        assert!((success_color.h - theme.success.h).abs() < f32::EPSILON);

        let error_color = MessageType::Error.color();
        assert!((error_color.h - theme.error.h).abs() < f32::EPSILON);

        let warning_color = MessageType::Warning.color();
        assert!((warning_color.h - theme.warning.h).abs() < f32::EPSILON);

        let info_color = MessageType::Info.color();
        assert!((info_color.h - theme.info.h).abs() < f32::EPSILON);
    }

    #[test]
    fn test_message_type_icons() {
        assert_eq!(MessageType::Success.icon(), icons::SUCCESS);
        assert_eq!(MessageType::Error.icon(), icons::ERROR);
        assert_eq!(MessageType::Warning.icon(), icons::WARNING);
        assert_eq!(MessageType::Info.icon(), icons::INFO);
    }

    #[test]
    fn test_status_message_creation() {
        let msg = StatusMessage::new(MessageType::Success, "All good");
        assert_eq!(msg.text.as_ref(), "All good");
        assert_eq!(msg.message_type, MessageType::Success);
        assert!(!msg.show_border);
    }

    #[test]
    fn test_status_message_convenience() {
        let s = StatusMessage::success("ok");
        assert_eq!(s.message_type, MessageType::Success);

        let w = StatusMessage::warning("watch out");
        assert_eq!(w.message_type, MessageType::Warning);

        let e = StatusMessage::error("bad");
        assert_eq!(e.message_type, MessageType::Error);

        let i = StatusMessage::info("fyi");
        assert_eq!(i.message_type, MessageType::Info);
    }

    #[test]
    fn test_status_message_with_border() {
        let msg = StatusMessage::info("bordered").with_border();
        assert!(msg.show_border);
    }
}
