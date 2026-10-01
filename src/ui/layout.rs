//! Layout components for the installer GUI
//!
//! Provides the Header and Footer components that frame every screen.
//! Header displays step progress and screen title.
//! Footer displays navigation hints and mode indicators.

use gpui::prelude::*;
use gpui::{div, px, relative, FontWeight, IntoElement, SharedString};

use super::theme::{spacing, AppTheme};
use crate::app::Screen;

/// Header component showing step progress bar and screen title
#[derive(IntoElement)]
pub struct Header {
    /// Current screen (used to derive step number)
    pub screen: Screen,
    /// Title text displayed in the header
    pub title: &'static str,
}

impl Header {
    /// Create a new header for the given screen
    pub fn new(screen: Screen, title: &'static str) -> Self {
        Self { screen, title }
    }
}

impl RenderOnce for Header {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let theme = AppTheme::new();
        let (current, total) = self.screen.step_number();
        let progress_fraction = current as f32 / total as f32;
        let step_label: SharedString =
            format!("Step {} of {} \u{2014} {}", current, total, self.title).into();

        div()
            .w_full()
            .p(px(spacing::MEDIUM))
            .bg(theme.background)
            .flex()
            .flex_col()
            .gap(px(spacing::SMALL))
            .child(
                // Step label
                div()
                    .text_color(theme.foreground)
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .child(step_label),
            )
            .child(
                // Progress bar track
                div()
                    .w_full()
                    .h(px(6.0))
                    .bg(theme.muted)
                    .rounded(px(3.0))
                    .child(
                        // Progress bar fill
                        div()
                            .h_full()
                            .rounded(px(3.0))
                            .bg(theme.success)
                            .w(relative(progress_fraction)),
                    ),
            )
    }
}

/// Footer component showing navigation hints and optional mode indicator
#[derive(IntoElement)]
pub struct Footer {
    /// Current screen (used to determine available navigation)
    pub screen: Screen,
    /// Whether the app is running in mock mode
    pub is_mock: bool,
}

impl Footer {
    /// Create a new footer for the given screen
    pub fn new(screen: Screen, is_mock: bool) -> Self {
        Self { screen, is_mock }
    }
}

impl RenderOnce for Footer {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let theme = AppTheme::new();

        let mut row = div()
            .w_full()
            .p(px(spacing::MEDIUM))
            .bg(theme.background)
            .border_t_1()
            .border_color(theme.border)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing::MEDIUM));

        // Back hint (if navigable)
        if self.screen.can_go_back() {
            row = row.child(hint_pair("\u{2190} Back", theme.primary));
            row = row.child(separator(theme.muted));
        }

        // Next / Finish hint
        if self.screen.next().is_some() {
            row = row.child(hint_pair("\u{21b5} Next", theme.success));
            row = row.child(separator(theme.muted));
        }

        // Help hint
        row = row.child(hint_pair("? Help", theme.warning));
        row = row.child(separator(theme.muted));

        // Quit hint
        row = row.child(hint_pair("q Quit", theme.error));

        // Mock mode indicator
        if self.is_mock {
            row = row.child(separator(theme.muted));
            row = row.child(
                div()
                    .text_color(theme.warning)
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .child("[MOCK MODE]"),
            );
        }

        row
    }
}

/// Create a styled hint label (single combined key+description string)
fn hint_pair(label: &'static str, color: gpui::Hsla) -> impl IntoElement {
    div().flex().flex_row().items_center().child(
        div()
            .text_color(color)
            .text_sm()
            .font_weight(FontWeight::BOLD)
            .child(label),
    )
}

/// Create a vertical separator pipe character
fn separator(color: gpui::Hsla) -> impl IntoElement {
    div().text_color(color).text_sm().child("|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_creation() {
        let header = Header::new(Screen::Welcome, "Welcome to NixOS Installation");
        assert_eq!(header.title, "Welcome to NixOS Installation");
    }

    #[test]
    fn test_footer_creation() {
        let footer = Footer::new(Screen::Welcome, false);
        assert!(!footer.is_mock);
    }

    #[test]
    fn test_footer_mock_mode() {
        let footer = Footer::new(Screen::DiskSelection, true);
        assert!(footer.is_mock);
    }

    #[test]
    fn test_header_all_screens() {
        let screens = [
            Screen::Welcome,
            Screen::DiskSelection,
            Screen::PartitionPlanning,
            Screen::Configuration,
            Screen::Confirmation,
            Screen::Installation,
            Screen::Complete,
        ];
        for screen in &screens {
            let header = Header::new(*screen, screen.title());
            let (step, total) = header.screen.step_number();
            assert!((1..=7).contains(&step));
            assert_eq!(total, 7);
        }
    }

    #[test]
    fn test_footer_back_navigation() {
        // Welcome has no back
        assert!(!Screen::Welcome.can_go_back());
        // DiskSelection can go back
        assert!(Screen::DiskSelection.can_go_back());
        // Installation cannot go back
        assert!(!Screen::Installation.can_go_back());
    }
}
