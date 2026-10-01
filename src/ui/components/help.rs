//! Help panel component
//!
//! Overlay panel with markdown-like content formatting.
//! Pure data struct with visibility state and a `view()` method.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement, SharedString};

use crate::ui::theme::{spacing, AppTheme};

/// Help panel component.
///
/// Pure data struct; the parent renders it as an overlay when visible.
/// Content lines are parsed for simple markdown-like formatting:
/// - Lines starting with `# ` are rendered as headers (cyan, bold)
/// - Lines starting with `## ` are rendered as sub-headers (cyan)
/// - Lines starting with `- ` are rendered as bullet lists
/// - Empty lines are rendered as spacing
pub struct HelpPanel {
    /// Help title
    title: String,
    /// Help content lines
    content: Vec<String>,
    /// Whether the panel is visible
    visible: bool,
}

impl HelpPanel {
    /// Create a new help panel.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: Vec::new(),
            visible: false,
        }
    }

    /// Add a help line.
    pub fn add_line(mut self, line: impl Into<String>) -> Self {
        self.content.push(line.into());
        self
    }

    /// Set content from a vector of lines.
    pub fn with_content(mut self, lines: Vec<String>) -> Self {
        self.content = lines;
        self
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Toggle visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Build a gpui element tree for the help overlay.
    ///
    /// Returns an empty div when not visible. When visible, renders
    /// as an absolute overlay covering the full parent area.
    pub fn view(&self) -> impl IntoElement {
        if !self.visible {
            return div();
        }

        let theme = AppTheme::new();

        let title: SharedString = format!(" {} (Press ? to close) ", self.title).into();

        let mut content_col = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(spacing::SMALL))
            .p(px(spacing::MEDIUM));

        for line in &self.content {
            if line.is_empty() {
                // Empty line = spacing
                content_col = content_col.child(div().h(px(spacing::MEDIUM)));
            } else if let Some(header) = line.strip_prefix("## ") {
                // Sub-header
                let text: SharedString = header.to_string().into();
                content_col = content_col.child(
                    div()
                        .text_color(theme.primary)
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .child(text),
                );
            } else if let Some(header) = line.strip_prefix("# ") {
                // Main header
                let text: SharedString = header.to_string().into();
                content_col = content_col.child(
                    div()
                        .text_color(theme.primary)
                        .font_weight(FontWeight::BOLD)
                        .child(text),
                );
            } else if let Some(item) = line.strip_prefix("- ") {
                // Bullet list item
                let text: SharedString = item.to_string().into();
                content_col = content_col.child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(spacing::SMALL))
                        .child(div().text_color(theme.warning).child("  \u{2022} "))
                        .child(div().text_color(theme.foreground).child(text)),
                );
            } else {
                // Normal text
                let text: SharedString = line.clone().into();
                content_col = content_col.child(div().text_color(theme.foreground).child(text));
            }
        }

        // Overlay
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(gpui::hsla(0.0, 0.0, 0.0, 0.7))
            .flex()
            .items_center()
            .justify_center()
            .child(
                // Panel
                div()
                    .w(px(600.0))
                    .max_h(px(500.0))
                    .bg(theme.background)
                    .border_2()
                    .border_color(theme.warning)
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
                            .border_b_1()
                            .border_color(theme.warning)
                            .text_color(theme.warning)
                            .font_weight(FontWeight::BOLD)
                            .child(title),
                    )
                    // Content
                    .child(content_col),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_panel_creation() {
        let help = HelpPanel::new("Help");
        assert_eq!(help.title, "Help");
        assert!(!help.is_visible());
        assert!(help.content.is_empty());
    }

    #[test]
    fn test_help_panel_toggle() {
        let mut help = HelpPanel::new("Help").add_line("Line 1").add_line("Line 2");

        assert!(!help.is_visible());

        help.toggle();
        assert!(help.is_visible());

        help.toggle();
        assert!(!help.is_visible());
    }

    #[test]
    fn test_help_panel_set_visible() {
        let mut help = HelpPanel::new("Help");

        help.set_visible(true);
        assert!(help.is_visible());

        help.set_visible(false);
        assert!(!help.is_visible());
    }

    #[test]
    fn test_help_content() {
        let help = HelpPanel::new("Test")
            .add_line("# Header")
            .add_line("## Sub-header")
            .add_line("- List item")
            .add_line("Normal text")
            .add_line("");

        assert_eq!(help.content.len(), 5);
    }

    #[test]
    fn test_help_with_content() {
        let lines = vec!["# Title".to_string(), "Some text".to_string()];
        let help = HelpPanel::new("Test").with_content(lines);
        assert_eq!(help.content.len(), 2);
        assert_eq!(help.content[0], "# Title");
    }

    #[test]
    fn test_help_add_line_chaining() {
        let help = HelpPanel::new("Test")
            .add_line("A")
            .add_line("B")
            .add_line("C");
        assert_eq!(help.content.len(), 3);
    }
}
