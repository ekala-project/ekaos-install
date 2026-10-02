//! Progress indicator components
//!
//! `ProgressBar` is a stateless `RenderOnce` div-based progress bar.
//! `Spinner` is a pure-state struct for indeterminate progress (no `Render` impl).

use gpui::prelude::*;
use gpui::{div, px, relative, Hsla, IntoElement, SharedString};

use crate::ui::theme::{icons, spacing, AppTheme};

/// Progress bar component (stateless, rendered by parent).
#[derive(IntoElement)]
pub struct ProgressBar {
    /// Progress percentage (0-100)
    percent: u16,
    /// Optional label text displayed over the bar
    label: Option<SharedString>,
    /// Fill color
    color: Hsla,
}

impl ProgressBar {
    /// Create a new progress bar at the given percentage.
    pub fn new(percent: u16) -> Self {
        let theme = AppTheme::new();
        Self {
            percent: percent.min(100),
            label: None,
            color: theme.success,
        }
    }

    /// Set a custom label.
    pub fn with_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the fill color.
    pub fn with_color(mut self, color: Hsla) -> Self {
        self.color = color;
        self
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let theme = AppTheme::new();
        let fraction = self.percent as f32 / 100.0;
        let label: SharedString = self
            .label
            .unwrap_or_else(|| format!("{}%", self.percent).into());

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(spacing::SMALL))
            .child(
                // Label
                div().text_color(theme.foreground).text_sm().child(label),
            )
            .child(
                // Track
                div()
                    .w_full()
                    .h(px(12.0))
                    .bg(theme.muted)
                    .rounded(px(4.0))
                    .overflow_hidden()
                    .child(
                        // Fill
                        div()
                            .h_full()
                            .rounded(px(4.0))
                            .bg(self.color)
                            .w(relative(fraction)),
                    ),
            )
    }
}

/// Spinner component for indeterminate progress.
///
/// Pure state struct with `tick()` and `current()` methods.
/// The parent is responsible for rendering the returned string.
pub struct Spinner {
    /// Current frame index
    frame: usize,
    /// Spinner characters
    frames: &'static [&'static str],
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    /// Create a new spinner.
    pub fn new() -> Self {
        Self {
            frame: 0,
            frames: icons::SPINNER,
        }
    }

    /// Advance to the next frame.
    pub fn tick(&mut self) {
        self.frame = (self.frame + 1) % self.frames.len();
    }

    /// Get the current frame character.
    pub fn current(&self) -> &str {
        self.frames[self.frame]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_creation() {
        let bar = ProgressBar::new(50);
        assert_eq!(bar.percent, 50);
    }

    #[test]
    fn test_progress_bar_clamp() {
        let bar = ProgressBar::new(150);
        assert_eq!(bar.percent, 100);
    }

    #[test]
    fn test_progress_bar_with_label() {
        let bar = ProgressBar::new(75).with_label("Installing...");
        assert!(bar.label.is_some());
        assert_eq!(bar.label.unwrap().as_ref(), "Installing...");
    }

    #[test]
    fn test_progress_bar_with_color() {
        let theme = AppTheme::new();
        let bar = ProgressBar::new(50).with_color(theme.primary);
        assert!((bar.color.h - theme.primary.h).abs() < f32::EPSILON);
    }

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new();
        assert_eq!(spinner.frame, 0);
        assert!(!spinner.current().is_empty());
    }

    #[test]
    fn test_spinner_tick() {
        let mut spinner = Spinner::new();
        let first = spinner.current().to_string();
        spinner.tick();
        let second = spinner.current().to_string();
        assert_ne!(first, second);
    }

    #[test]
    fn test_spinner_wraps() {
        let mut spinner = Spinner::new();
        let len = icons::SPINNER.len();
        for _ in 0..len {
            spinner.tick();
        }
        // Should be back to frame 0 after a full cycle
        assert_eq!(spinner.frame, 0);
    }

    #[test]
    fn test_spinner_default() {
        let spinner = Spinner::default();
        assert_eq!(spinner.frame, 0);
        assert_eq!(spinner.frames.len(), icons::SPINNER.len());
    }
}
