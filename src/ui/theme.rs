//! Theme system for consistent styling across the application
//!
//! This module provides a centralized theme with colors, styles, and spacing constants.

use ratatui::style::{Color, Modifier, Style};

/// Application theme with color palette and style definitions
#[derive(Debug, Clone)]
pub struct AppTheme {
    /// Primary accent color
    pub primary: Color,
    /// Success color
    pub success: Color,
    /// Warning color
    pub warning: Color,
    /// Error color
    pub error: Color,
    /// Info color
    pub info: Color,
    /// Background color
    pub background: Color,
    /// Foreground (text) color
    pub foreground: Color,
    /// Muted/disabled color
    pub muted: Color,
    /// Border color (normal state)
    pub border: Color,
    /// Border color (focused state)
    pub border_focused: Color,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::new()
    }
}

impl AppTheme {
    /// Create the default theme
    pub fn new() -> Self {
        Self {
            primary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            background: Color::Black,
            foreground: Color::White,
            muted: Color::DarkGray,
            border: Color::White,
            border_focused: Color::Cyan,
        }
    }

    // Style builders for common UI patterns

    /// Normal text style
    pub fn text(&self) -> Style {
        Style::default().fg(self.foreground)
    }

    /// Muted/disabled text style
    pub fn text_muted(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Title text style (bold, primary color)
    pub fn title(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Subtitle style
    pub fn subtitle(&self) -> Style {
        Style::default().fg(self.primary)
    }

    /// Success message style
    pub fn success(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Success message style (bold)
    pub fn success_bold(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    /// Warning message style
    pub fn warning(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Warning message style (bold)
    pub fn warning_bold(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    /// Error message style
    pub fn error(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Error message style (bold)
    pub fn error_bold(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    /// Info message style
    pub fn info(&self) -> Style {
        Style::default().fg(self.info)
    }

    /// Info message style (bold)
    pub fn info_bold(&self) -> Style {
        Style::default().fg(self.info).add_modifier(Modifier::BOLD)
    }

    /// Normal border style
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Focused border style
    pub fn border_focused_style(&self) -> Style {
        Style::default()
            .fg(self.border_focused)
            .add_modifier(Modifier::BOLD)
    }

    /// Disabled border style
    pub fn border_disabled_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Primary button style (focused)
    pub fn button_primary_focused(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Primary button style (normal)
    pub fn button_primary(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Secondary button style (focused)
    pub fn button_secondary_focused(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(self.foreground)
            .add_modifier(Modifier::BOLD)
    }

    /// Secondary button style (normal)
    pub fn button_secondary(&self) -> Style {
        Style::default()
            .fg(self.foreground)
            .add_modifier(Modifier::BOLD)
    }

    /// Danger button style (focused)
    pub fn button_danger_focused(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(self.error)
            .add_modifier(Modifier::BOLD)
    }

    /// Danger button style (normal)
    pub fn button_danger(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    /// Selection indicator style
    pub fn selection(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Focused item style
    pub fn focused_item(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Help text style
    pub fn help_text(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Keyboard shortcut style
    pub fn shortcut(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }
}

/// Spacing constants for consistent layout
pub mod spacing {
    /// No margin
    pub const NONE: u16 = 0;
    /// Small margin (1 unit)
    pub const SMALL: u16 = 1;
    /// Medium margin (2 units)
    pub const MEDIUM: u16 = 2;
    /// Large margin (3 units)
    pub const LARGE: u16 = 3;
    /// Extra large margin (4 units)
    pub const XLARGE: u16 = 4;
}

/// Icons used throughout the application
pub mod icons {
    /// Success checkmark
    pub const SUCCESS: &str = "✓";
    /// Error cross
    pub const ERROR: &str = "✗";
    /// Warning triangle
    pub const WARNING: &str = "⚠";
    /// Info circle
    pub const INFO: &str = "ℹ";
    /// Selection arrow
    pub const SELECTION: &str = "▶";
    /// Checked checkbox
    pub const CHECKED: &str = "[✓]";
    /// Unchecked checkbox
    pub const UNCHECKED: &str = "[ ]";
    /// Bullet point
    pub const BULLET: &str = "•";
    /// Loading spinner frames
    pub const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = AppTheme::new();
        assert_eq!(theme.primary, Color::Cyan);
        assert_eq!(theme.success, Color::Green);
        assert_eq!(theme.error, Color::Red);
    }

    #[test]
    fn test_style_builders() {
        let theme = AppTheme::new();

        let title = theme.title();
        assert_eq!(title.fg, Some(Color::Cyan));

        let error = theme.error_bold();
        assert_eq!(error.fg, Some(Color::Red));
    }

    #[test]
    fn test_spacing_constants() {
        assert_eq!(spacing::SMALL, 1);
        assert_eq!(spacing::MEDIUM, 2);
        assert_eq!(spacing::LARGE, 3);
    }

    #[test]
    fn test_icons() {
        assert_eq!(icons::SUCCESS, "✓");
        assert_eq!(icons::ERROR, "✗");
        assert_eq!(icons::WARNING, "⚠");
        assert!(icons::SPINNER.len() > 0);
    }
}
