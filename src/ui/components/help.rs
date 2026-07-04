//! Help panel component

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::Component;

/// Help panel component
pub struct HelpPanel {
    /// Help title
    title: String,
    /// Help content lines
    content: Vec<String>,
    /// Whether the panel is visible
    visible: bool,
}

impl HelpPanel {
    /// Create a new help panel
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: Vec::new(),
            visible: false,
        }
    }

    /// Add a help line
    pub fn add_line(mut self, line: impl Into<String>) -> Self {
        self.content.push(line.into());
        self
    }

    /// Add multiple help lines
    pub fn with_content(mut self, lines: Vec<String>) -> Self {
        self.content = lines;
        self
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Component for HelpPanel {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        if !self.visible {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} (Press ? to close) ", self.title))
            .border_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        let lines: Vec<Line> = self
            .content
            .iter()
            .map(|line| {
                if line.is_empty() {
                    Line::from("")
                } else if line.starts_with("# ") {
                    // Header
                    Line::from(Span::styled(
                        line.trim_start_matches("# "),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with("- ") {
                    // List item
                    Line::from(vec![
                        Span::styled("  • ", Style::default().fg(Color::Yellow)),
                        Span::styled(
                            line.trim_start_matches("- "),
                            Style::default().fg(Color::White),
                        ),
                    ])
                } else {
                    Line::from(Span::styled(
                        line.as_str(),
                        Style::default().fg(Color::White),
                    ))
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_panel() {
        let mut help = HelpPanel::new("Help").add_line("Line 1").add_line("Line 2");

        assert!(!help.is_visible());

        help.toggle();
        assert!(help.is_visible());

        help.toggle();
        assert!(!help.is_visible());
    }

    #[test]
    fn test_help_content() {
        let help = HelpPanel::new("Test")
            .add_line("# Header")
            .add_line("- List item")
            .add_line("Normal text");

        assert_eq!(help.content.len(), 3);
    }
}
