//! Theme system for consistent styling across the application
//!
//! This module provides a centralized theme with colors and spacing constants
//! for the gpui-based GUI.

use gpui::Hsla;

/// Application theme with color palette for the installer GUI
#[derive(Debug, Clone)]
pub struct AppTheme {
    /// Primary accent color (cyan)
    pub primary: Hsla,
    /// Success color (green)
    pub success: Hsla,
    /// Warning color (yellow)
    pub warning: Hsla,
    /// Error color (red)
    pub error: Hsla,
    /// Info color (blue)
    pub info: Hsla,
    /// Background color (dark)
    pub background: Hsla,
    /// Foreground (text) color (white)
    pub foreground: Hsla,
    /// Muted/disabled color (dark gray)
    pub muted: Hsla,
    /// Border color (normal state)
    pub border: Hsla,
    /// Border color (focused state, same as primary)
    pub border_focused: Hsla,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            primary: gpui::hsla(0.5, 0.8, 0.6, 1.0),
            success: gpui::hsla(0.33, 0.7, 0.45, 1.0),
            warning: gpui::hsla(0.13, 0.9, 0.55, 1.0),
            error: gpui::hsla(0.0, 0.8, 0.5, 1.0),
            info: gpui::hsla(0.6, 0.7, 0.5, 1.0),
            background: gpui::hsla(0.0, 0.0, 0.1, 1.0),
            foreground: gpui::hsla(0.0, 0.0, 0.95, 1.0),
            muted: gpui::hsla(0.0, 0.0, 0.4, 1.0),
            border: gpui::hsla(0.0, 0.0, 0.7, 1.0),
            border_focused: gpui::hsla(0.5, 0.8, 0.6, 1.0),
        }
    }
}

impl AppTheme {
    /// Create the default theme
    pub fn new() -> Self {
        Self::default()
    }
}

/// Spacing constants for consistent layout (in pixels)
pub mod spacing {
    /// No spacing
    pub const NONE: f32 = 0.0;
    /// Small spacing (4px)
    pub const SMALL: f32 = 4.0;
    /// Medium spacing (8px)
    pub const MEDIUM: f32 = 8.0;
    /// Large spacing (16px)
    pub const LARGE: f32 = 16.0;
    /// Extra large spacing (24px)
    pub const XLARGE: f32 = 24.0;
    /// Extra extra large spacing (32px)
    pub const XXLARGE: f32 = 32.0;
}

/// Icons used throughout the application
pub mod icons {
    /// Success checkmark
    pub const SUCCESS: &str = "\u{2713}";
    /// Error cross
    pub const ERROR: &str = "\u{2717}";
    /// Warning triangle
    pub const WARNING: &str = "\u{26a0}";
    /// Info circle
    pub const INFO: &str = "\u{2139}";
    /// Selection arrow
    pub const SELECTION: &str = "\u{25b6}";
    /// Checked checkbox
    pub const CHECKED: &str = "[\u{2713}]";
    /// Unchecked checkbox
    pub const UNCHECKED: &str = "[ ]";
    /// Bullet point
    pub const BULLET: &str = "\u{2022}";
    /// Loading spinner frames
    pub const SPINNER: &[&str] = &[
        "\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}", "\u{2826}",
        "\u{2827}", "\u{2807}", "\u{280f}",
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = AppTheme::new();
        // Primary is cyan: hue ~0.5
        assert!((theme.primary.h - 0.5).abs() < f32::EPSILON);
        assert!((theme.primary.s - 0.8).abs() < f32::EPSILON);
        assert!((theme.primary.l - 0.6).abs() < f32::EPSILON);
        assert!((theme.primary.a - 1.0).abs() < f32::EPSILON);

        // Success is green: hue ~0.33
        assert!((theme.success.h - 0.33).abs() < f32::EPSILON);

        // Error is red: hue ~0.0
        assert!((theme.error.h - 0.0).abs() < f32::EPSILON);
        assert!((theme.error.s - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_theme_default_matches_new() {
        let from_new = AppTheme::new();
        let from_default = AppTheme::default();
        assert!((from_new.primary.h - from_default.primary.h).abs() < f32::EPSILON);
        assert!((from_new.background.l - from_default.background.l).abs() < f32::EPSILON);
        assert!((from_new.foreground.l - from_default.foreground.l).abs() < f32::EPSILON);
    }

    #[test]
    fn test_border_focused_matches_primary() {
        let theme = AppTheme::new();
        assert!((theme.border_focused.h - theme.primary.h).abs() < f32::EPSILON);
        assert!((theme.border_focused.s - theme.primary.s).abs() < f32::EPSILON);
        assert!((theme.border_focused.l - theme.primary.l).abs() < f32::EPSILON);
    }

    #[test]
    fn test_all_colors_fully_opaque() {
        let theme = AppTheme::new();
        assert!((theme.primary.a - 1.0).abs() < f32::EPSILON);
        assert!((theme.success.a - 1.0).abs() < f32::EPSILON);
        assert!((theme.warning.a - 1.0).abs() < f32::EPSILON);
        assert!((theme.error.a - 1.0).abs() < f32::EPSILON);
        assert!((theme.info.a - 1.0).abs() < f32::EPSILON);
        assert!((theme.background.a - 1.0).abs() < f32::EPSILON);
        assert!((theme.foreground.a - 1.0).abs() < f32::EPSILON);
        assert!((theme.muted.a - 1.0).abs() < f32::EPSILON);
        assert!((theme.border.a - 1.0).abs() < f32::EPSILON);
        assert!((theme.border_focused.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_spacing_constants() {
        assert!((spacing::NONE - 0.0).abs() < f32::EPSILON);
        assert!((spacing::SMALL - 4.0).abs() < f32::EPSILON);
        assert!((spacing::MEDIUM - 8.0).abs() < f32::EPSILON);
        assert!((spacing::LARGE - 16.0).abs() < f32::EPSILON);
        assert!((spacing::XLARGE - 24.0).abs() < f32::EPSILON);
        assert!((spacing::XXLARGE - 32.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_icons() {
        assert_eq!(icons::SUCCESS, "\u{2713}");
        assert_eq!(icons::ERROR, "\u{2717}");
        assert_eq!(icons::WARNING, "\u{26a0}");
        assert_eq!(icons::INFO, "\u{2139}");
        assert_eq!(icons::SELECTION, "\u{25b6}");
        assert_eq!(icons::CHECKED, "[\u{2713}]");
        assert_eq!(icons::UNCHECKED, "[ ]");
        assert_eq!(icons::BULLET, "\u{2022}");
    }

    #[test]
    fn test_spinner_frames() {
        assert_eq!(icons::SPINNER.len(), 10);
        for frame in icons::SPINNER {
            assert!(!frame.is_empty());
        }
    }
}
