//! Confirmation dialog component
//!
//! Centered popup overlay with Yes/No buttons.
//! Pure data struct with navigation methods and a `view()` method.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement, SharedString};

use crate::ui::theme::{spacing, AppTheme};

/// Confirmation dialog with Yes/No buttons.
///
/// Pure data struct; the parent renders it as an overlay and routes
/// keyboard events to `navigate_left()`, `navigate_right()`, etc.
pub struct ConfirmDialog {
    /// Dialog title
    title: String,
    /// Dialog message
    message: String,
    /// Currently focused button (0=Yes, 1=No)
    focused_button: usize,
    /// Dialog type (affects color)
    dialog_type: DialogType,
}

/// Dialog type variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    /// Normal confirmation (blue)
    Confirm,
    /// Warning confirmation (yellow)
    Warning,
    /// Danger confirmation (red)
    Danger,
}

impl ConfirmDialog {
    /// Create a new confirmation dialog. Defaults to "No" focused.
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            focused_button: 1, // Default to "No"
            dialog_type: DialogType::Confirm,
        }
    }

    /// Set dialog type.
    pub fn with_type(mut self, dialog_type: DialogType) -> Self {
        self.dialog_type = dialog_type;
        self
    }

    /// Create a warning dialog.
    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_type(DialogType::Warning)
    }

    /// Create a danger dialog.
    pub fn danger(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_type(DialogType::Danger)
    }

    /// Get whether "Yes" is currently focused.
    pub fn is_yes_focused(&self) -> bool {
        self.focused_button == 0
    }

    /// Toggle between Yes and No.
    pub fn toggle_button(&mut self) {
        self.focused_button = 1 - self.focused_button;
    }

    /// Select "No" (e.g. on Escape).
    pub fn select_no(&mut self) {
        self.focused_button = 1;
    }

    /// Get the dialog type.
    pub fn dialog_type(&self) -> DialogType {
        self.dialog_type
    }

    /// Get the dialog type color from the theme.
    fn type_color(&self) -> gpui::Hsla {
        let theme = AppTheme::new();
        match self.dialog_type {
            DialogType::Confirm => theme.info,
            DialogType::Warning => theme.warning,
            DialogType::Danger => theme.error,
        }
    }

    /// Build a gpui element tree for this dialog.
    ///
    /// Renders as a centered overlay. The parent should place this
    /// on top of other content using absolute positioning.
    pub fn view(&self) -> impl IntoElement {
        let theme = AppTheme::new();
        let type_color = self.type_color();

        let title: SharedString = self.title.clone().into();
        let message: SharedString = self.message.clone().into();

        // Build Yes button
        let yes_button = if self.focused_button == 0 {
            div()
                .px(px(spacing::LARGE))
                .py(px(spacing::SMALL))
                .bg(theme.success)
                .text_color(gpui::hsla(0.0, 0.0, 0.0, 1.0))
                .font_weight(FontWeight::BOLD)
                .rounded(px(4.0))
                .flex()
                .justify_center()
                .child("[ Yes ]")
        } else {
            div()
                .px(px(spacing::LARGE))
                .py(px(spacing::SMALL))
                .text_color(theme.success)
                .font_weight(FontWeight::BOLD)
                .rounded(px(4.0))
                .border_1()
                .border_color(theme.success)
                .flex()
                .justify_center()
                .child("[ Yes ]")
        };

        // Build No button
        let no_button = if self.focused_button == 1 {
            div()
                .px(px(spacing::LARGE))
                .py(px(spacing::SMALL))
                .bg(theme.error)
                .text_color(gpui::hsla(0.0, 0.0, 0.0, 1.0))
                .font_weight(FontWeight::BOLD)
                .rounded(px(4.0))
                .flex()
                .justify_center()
                .child("[ No ]")
        } else {
            div()
                .px(px(spacing::LARGE))
                .py(px(spacing::SMALL))
                .text_color(theme.error)
                .font_weight(FontWeight::BOLD)
                .rounded(px(4.0))
                .border_1()
                .border_color(theme.error)
                .flex()
                .justify_center()
                .child("[ No ]")
        };

        // Outer overlay: semi-transparent backdrop
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(gpui::hsla(0.0, 0.0, 0.0, 0.6))
            .flex()
            .items_center()
            .justify_center()
            .child(
                // Dialog box
                div()
                    .w(px(400.0))
                    .bg(theme.background)
                    .border_2()
                    .border_color(type_color)
                    .rounded(px(8.0))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    // Title bar
                    .child(
                        div()
                            .w_full()
                            .px(px(spacing::LARGE))
                            .py(px(spacing::MEDIUM))
                            .bg(type_color)
                            .text_color(gpui::hsla(0.0, 0.0, 0.0, 1.0))
                            .font_weight(FontWeight::BOLD)
                            .child(title),
                    )
                    // Message
                    .child(
                        div()
                            .w_full()
                            .px(px(spacing::LARGE))
                            .py(px(spacing::LARGE))
                            .text_color(theme.foreground)
                            .child(message),
                    )
                    // Button row
                    .child(
                        div()
                            .w_full()
                            .px(px(spacing::LARGE))
                            .py(px(spacing::MEDIUM))
                            .flex()
                            .flex_row()
                            .justify_center()
                            .gap(px(spacing::LARGE))
                            .child(yes_button)
                            .child(no_button),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_dialog_creation() {
        let dialog = ConfirmDialog::new("Confirm", "Are you sure?");
        assert_eq!(dialog.focused_button, 1); // Defaults to No
        assert_eq!(dialog.dialog_type, DialogType::Confirm);
        assert!(!dialog.is_yes_focused());
    }

    #[test]
    fn test_dialog_navigation() {
        let mut dialog = ConfirmDialog::new("Test", "Message");

        assert_eq!(dialog.focused_button, 1);

        dialog.toggle_button();
        assert_eq!(dialog.focused_button, 0);
        assert!(dialog.is_yes_focused());

        dialog.toggle_button();
        assert_eq!(dialog.focused_button, 1);
        assert!(!dialog.is_yes_focused());
    }

    #[test]
    fn test_dialog_types() {
        let warning = ConfirmDialog::warning("Warning", "Be careful");
        assert_eq!(warning.dialog_type, DialogType::Warning);

        let danger = ConfirmDialog::danger("Danger", "This is dangerous");
        assert_eq!(danger.dialog_type, DialogType::Danger);
    }

    #[test]
    fn test_dialog_select_no() {
        let mut dialog = ConfirmDialog::new("Test", "Message");
        dialog.toggle_button(); // Now on Yes
        assert!(dialog.is_yes_focused());

        dialog.select_no();
        assert!(!dialog.is_yes_focused());
        assert_eq!(dialog.focused_button, 1);
    }

    #[test]
    fn test_dialog_with_type() {
        let dialog = ConfirmDialog::new("Test", "Message").with_type(DialogType::Danger);
        assert_eq!(dialog.dialog_type, DialogType::Danger);
    }

    #[test]
    fn test_dialog_type_accessor() {
        let dialog = ConfirmDialog::warning("Warn", "msg");
        assert_eq!(dialog.dialog_type(), DialogType::Warning);
    }
}
