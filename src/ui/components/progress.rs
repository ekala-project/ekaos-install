//! Progress indicator components

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Gauge},
};

use super::Component;

/// Progress bar component
pub struct ProgressBar {
    /// Progress percentage (0-100)
    percent: u16,
    /// Label text
    label: Option<String>,
    /// Color
    color: Color,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(percent: u16) -> Self {
        Self {
            percent: percent.min(100),
            label: None,
            color: Color::Green,
        }
    }

    /// Set label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set percentage
    pub fn set_percent(&mut self, percent: u16) {
        self.percent = percent.min(100);
    }
}

impl Component for ProgressBar {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let label = if let Some(text) = &self.label {
            text.clone()
        } else {
            format!("{}%", self.percent)
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(
                Style::default()
                    .fg(self.color)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(self.percent)
            .label(Span::styled(label, Style::default().fg(Color::White)));

        frame.render_widget(gauge, area);
    }
}

/// Spinner component for indeterminate progress
pub struct Spinner {
    /// Current frame
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
    /// Create a new spinner
    pub fn new() -> Self {
        Self {
            frame: 0,
            frames: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
        }
    }

    /// Advance to next frame
    pub fn tick(&mut self) {
        self.frame = (self.frame + 1) % self.frames.len();
    }

    /// Get current frame character
    pub fn current(&self) -> &str {
        self.frames[self.frame]
    }
}

impl Component for Spinner {
    fn render(&mut self, frame: &mut Frame<'_>, _area: Rect) {
        // Spinner is typically rendered inline, not in its own area
        // This is a placeholder implementation
        let _ = frame;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        let mut bar = ProgressBar::new(50);
        assert_eq!(bar.percent, 50);

        bar.set_percent(75);
        assert_eq!(bar.percent, 75);

        bar.set_percent(150); // Should clamp to 100
        assert_eq!(bar.percent, 100);
    }

    #[test]
    fn test_spinner() {
        let mut spinner = Spinner::new();
        let first = spinner.current().to_string();
        spinner.tick();
        let second = spinner.current().to_string();

        assert_ne!(first, second);
    }
}
