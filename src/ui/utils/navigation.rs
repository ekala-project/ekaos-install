//! Navigation hint rendering utilities

use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::ui::theme::AppTheme;

/// Render navigation hints at the bottom of the screen
///
/// This function creates a centered navigation bar showing keyboard shortcuts
/// and their descriptions. The shortcuts are styled with the theme's shortcut
/// style for visual emphasis.
///
/// # Arguments
///
/// * `frame` - The ratatui frame to render into
/// * `hints` - Slice of (key, description) tuples to display
/// * `theme` - The application theme for styling
/// * `area` - The rectangular area to render into
///
/// # Examples
///
/// ```ignore
/// render_navigation_hints(
///     frame,
///     &[
///         ("Enter", "Continue"),
///         ("←", "Back"),
///         ("?", "Help"),
///         ("q", "Quit"),
///     ],
///     &self.theme,
///     chunks[last],
/// );
/// ```
pub fn render_navigation_hints(
    frame: &mut Frame<'_>,
    hints: &[(&str, &str)],
    theme: &AppTheme,
    area: Rect,
) {
    if hints.is_empty() {
        return;
    }

    let mut spans = Vec::new();

    for (i, (key, description)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" | "));
        }
        spans.push(Span::styled(*key, theme.shortcut()));
        spans.push(Span::raw(" "));
        spans.push(Span::raw(*description));
    }

    let nav_text = vec![Line::from(""), Line::from(spans)];
    let nav_para = Paragraph::new(nav_text)
        .alignment(Alignment::Center)
        .style(theme.text());

    frame.render_widget(nav_para, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_navigation_hints() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = AppTheme::new();

        terminal
            .draw(|frame| {
                let area = frame.size();
                render_navigation_hints(
                    frame,
                    &[("Enter", "Continue"), ("q", "Quit")],
                    &theme,
                    area,
                );
            })
            .unwrap();

        // Test that it doesn't panic
    }

    #[test]
    fn test_render_empty_hints() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = AppTheme::new();

        terminal
            .draw(|frame| {
                let area = frame.size();
                render_navigation_hints(frame, &[], &theme, area);
            })
            .unwrap();

        // Test that it doesn't panic with empty hints
    }

    #[test]
    fn test_render_single_hint() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = AppTheme::new();

        terminal
            .draw(|frame| {
                let area = frame.size();
                render_navigation_hints(frame, &[("q", "Quit")], &theme, area);
            })
            .unwrap();

        // Test that it doesn't panic with single hint
    }
}
