//! Layout definitions and rendering helpers
//!
//! This module defines the three-panel layout structure used throughout the application:
//! - Header: Progress indicator and screen title
//! - Content: Main interactive area
//! - Footer: Navigation hints and help information

use ratatui::{
    layout::{Constraint, Direction, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::Screen;

/// Layout manager for the application
#[derive(Debug)]
pub struct Layout {
    /// Header height in lines
    pub header_height: u16,
    /// Footer height in lines
    pub footer_height: u16,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            header_height: 3,
            footer_height: 3,
        }
    }
}

impl Layout {
    /// Create a new layout manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Split the terminal area into header, content, and footer
    pub fn split(&self, area: Rect) -> (Rect, Rect, Rect) {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(self.header_height),
                Constraint::Min(0),
                Constraint::Length(self.footer_height),
            ])
            .split(area);

        (chunks[0], chunks[1], chunks[2])
    }
}

/// Render the header with progress indicator and title
pub fn render_header(frame: &mut Frame<'_>, area: Rect, screen: Screen, title: &str) {
    let (current, total) = screen.step_number();
    let progress = (current as f64 / total as f64) * 100.0;

    // Create progress bar
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Step {} of {} ", current, total)),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Green)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .percent(progress as u16)
        .label(Span::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

    frame.render_widget(gauge, area);
}

/// Render the footer with navigation hints
pub fn render_footer(frame: &mut Frame<'_>, area: Rect, screen: Screen, is_mock: bool) {
    let mut hints = Vec::new();

    // Add navigation hints based on current screen
    if screen.can_go_back() {
        hints.push(Span::styled(
            "← Back",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
        hints.push(Span::raw(" | "));
    }

    if screen.next().is_some() {
        hints.push(Span::styled(
            "↵ Next",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));
        hints.push(Span::raw(" | "));
    }

    hints.push(Span::styled(
        "? Help",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
    hints.push(Span::raw(" | "));
    hints.push(Span::styled(
        "q Quit",
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    ));

    // Add mode indicator
    if is_mock {
        hints.push(Span::raw(" | "));
        hints.push(Span::styled(
            "[MOCK MODE]",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let footer = Paragraph::new(Line::from(hints))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(footer, area);
}

/// Render a centered message (useful for welcome/completion screens)
pub fn render_centered_message(frame: &mut Frame<'_>, area: Rect, lines: Vec<Line<'_>>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Theme colors for the application
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub background: Color,
    pub foreground: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            background: Color::Black,
            foreground: Color::White,
        }
    }
}

impl Theme {
    /// Get the default theme
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a style for the given color type
    pub fn style(&self, color_type: ThemeColor) -> Style {
        let color = match color_type {
            ThemeColor::Primary => self.primary,
            ThemeColor::Success => self.success,
            ThemeColor::Warning => self.warning,
            ThemeColor::Error => self.error,
            ThemeColor::Info => self.info,
        };
        Style::default().fg(color)
    }

    /// Get a bold style for the given color type
    pub fn bold_style(&self, color_type: ThemeColor) -> Style {
        self.style(color_type).add_modifier(Modifier::BOLD)
    }
}

/// Theme color types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeColor {
    Primary,
    Success,
    Warning,
    Error,
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_split() {
        let layout = Layout::new();
        let area = Rect::new(0, 0, 100, 30);
        let (header, content, footer) = layout.split(area);

        assert_eq!(header.height, 3);
        assert_eq!(footer.height, 3);
        assert!(content.height > 0);
        assert_eq!(header.height + content.height + footer.height, area.height);
    }

    #[test]
    fn test_theme_colors() {
        let theme = Theme::new();
        assert_eq!(theme.primary, Color::Cyan);
        assert_eq!(theme.success, Color::Green);
        assert_eq!(theme.error, Color::Red);
    }
}
