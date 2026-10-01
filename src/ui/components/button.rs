//! Button component
//!
//! Stateless `RenderOnce` button widget. The parent owns focused/enabled state.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, Hsla, IntoElement, SharedString};

use crate::ui::theme::{spacing, AppTheme};

/// Stateless button widget rendered by the parent.
#[derive(IntoElement)]
pub struct Button {
    /// Button label
    label: SharedString,
    /// Whether the button appears focused
    focused: bool,
    /// Whether the button is enabled
    enabled: bool,
    /// Visual style variant
    style: ButtonStyle,
}

/// Button visual style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Primary action button (cyan)
    Primary,
    /// Secondary action button (white)
    Secondary,
    /// Dangerous action button (red)
    Danger,
    /// Success action button (green)
    Success,
}

impl Button {
    /// Create a new button with the given label.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            focused: false,
            enabled: true,
            style: ButtonStyle::Primary,
        }
    }

    /// Set the button style variant.
    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Set whether this button appears focused.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set whether this button is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Get the base color for this button style from the theme.
    fn base_color(&self) -> Hsla {
        let theme = AppTheme::new();
        match self.style {
            ButtonStyle::Primary => theme.primary,
            ButtonStyle::Secondary => theme.foreground,
            ButtonStyle::Danger => theme.error,
            ButtonStyle::Success => theme.success,
        }
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let theme = AppTheme::new();
        let color = if !self.enabled {
            theme.muted
        } else {
            self.base_color()
        };

        let mut el = div()
            .px(px(spacing::LARGE))
            .py(px(spacing::SMALL))
            .rounded(px(4.0))
            .border_1()
            .flex()
            .items_center()
            .justify_center()
            .text_sm()
            .font_weight(FontWeight::BOLD);

        if self.focused && self.enabled {
            // Focused: colored background, black text
            el = el
                .bg(color)
                .text_color(gpui::hsla(0.0, 0.0, 0.0, 1.0))
                .border_color(color);
        } else if self.enabled {
            // Normal: colored text and border, no background
            el = el.text_color(color).border_color(color);
        } else {
            // Disabled: muted everything
            el = el.text_color(theme.muted).border_color(theme.muted);
        }

        el.child(self.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new("Click Me");
        assert_eq!(button.label.as_ref(), "Click Me");
        assert!(!button.focused);
        assert!(button.enabled);
        assert_eq!(button.style, ButtonStyle::Primary);
    }

    #[test]
    fn test_button_styles() {
        let theme = AppTheme::new();

        let primary = Button::new("Primary");
        let primary_color = primary.base_color();
        assert!((primary_color.h - theme.primary.h).abs() < f32::EPSILON);

        let danger = Button::new("Delete").with_style(ButtonStyle::Danger);
        let danger_color = danger.base_color();
        assert!((danger_color.h - theme.error.h).abs() < f32::EPSILON);

        let success = Button::new("OK").with_style(ButtonStyle::Success);
        let success_color = success.base_color();
        assert!((success_color.h - theme.success.h).abs() < f32::EPSILON);

        let secondary = Button::new("Cancel").with_style(ButtonStyle::Secondary);
        let secondary_color = secondary.base_color();
        assert!((secondary_color.l - theme.foreground.l).abs() < f32::EPSILON);
    }

    #[test]
    fn test_button_builder() {
        let button = Button::new("Test")
            .with_style(ButtonStyle::Danger)
            .focused(true)
            .enabled(false);

        assert_eq!(button.style, ButtonStyle::Danger);
        assert!(button.focused);
        assert!(!button.enabled);
    }

    #[test]
    fn test_button_base_color_disabled() {
        let theme = AppTheme::new();
        let button = Button::new("Disabled").enabled(false);
        // base_color returns the style color regardless; the render
        // method picks muted when disabled.
        let color = button.base_color();
        assert!((color.h - theme.primary.h).abs() < f32::EPSILON);
    }
}
