//! Navigation hint rendering component

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement};

use crate::ui::theme::{spacing, AppTheme};

/// A horizontal row of key-description hint pairs for navigation guidance
#[derive(IntoElement)]
pub struct NavigationHints {
    /// Pairs of (key label, description) to display
    pub hints: Vec<(&'static str, &'static str)>,
}

impl NavigationHints {
    /// Create a new navigation hints component
    pub fn new(hints: Vec<(&'static str, &'static str)>) -> Self {
        Self { hints }
    }
}

impl RenderOnce for NavigationHints {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let theme = AppTheme::new();

        if self.hints.is_empty() {
            return div();
        }

        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing::MEDIUM));

        for (i, (key, description)) in self.hints.iter().enumerate() {
            if i > 0 {
                row = row.child(div().text_color(theme.muted).text_sm().child("|"));
            }
            row = row.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing::SMALL))
                    .child(
                        div()
                            .text_color(theme.primary)
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .child(*key),
                    )
                    .child(
                        div()
                            .text_color(theme.foreground)
                            .text_sm()
                            .child(*description),
                    ),
            );
        }

        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_hints_creation() {
        let hints = NavigationHints::new(vec![("Enter", "Continue"), ("q", "Quit")]);
        assert_eq!(hints.hints.len(), 2);
        assert_eq!(hints.hints[0], ("Enter", "Continue"));
        assert_eq!(hints.hints[1], ("q", "Quit"));
    }

    #[test]
    fn test_empty_hints() {
        let hints = NavigationHints::new(vec![]);
        assert!(hints.hints.is_empty());
    }

    #[test]
    fn test_single_hint() {
        let hints = NavigationHints::new(vec![("q", "Quit")]);
        assert_eq!(hints.hints.len(), 1);
    }
}
