//! Installation success/complete screen
//!
//! Displays installation summary and next steps after a successful install.
//! Implements `gpui::Render` for the gpui GUI framework.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement, SharedString};

use crate::ui::theme::{icons, spacing, AppTheme};
use crate::ui::utils::NavigationHints;

/// Success screen state
pub struct SuccessScreen {
    /// Selected option (0 = View Summary, 1 = Reboot)
    selected: usize,
}

#[allow(dead_code)]
impl SuccessScreen {
    /// Create a new success screen
    pub fn new() -> Self {
        Self { selected: 1 } // Default to Reboot
    }

    /// Select next option
    pub fn select_next(&mut self) {
        self.selected = (self.selected + 1) % 2;
    }

    /// Select previous option
    pub fn select_previous(&mut self) {
        self.selected = if self.selected == 0 { 1 } else { 0 };
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Get help content lines for the help panel
    pub fn help_content(&self) -> Vec<String> {
        vec![
            "# Installation Complete".to_string(),
            "".to_string(),
            "Congratulations! NixOS has been successfully installed.".to_string(),
            "".to_string(),
            "## What Was Installed".to_string(),
            "".to_string(),
            "- NixOS base system with your chosen configuration".to_string(),
            "- Bootloader (systemd-boot or GRUB)".to_string(),
            "- Your user account with sudo privileges".to_string(),
            "- Network Manager for connectivity".to_string(),
            "- Essential system packages".to_string(),
            "".to_string(),
            "## Configuration Files".to_string(),
            "".to_string(),
            "- /etc/nixos/configuration.nix (your main config)".to_string(),
            "- /etc/nixos/hardware-configuration.nix (hardware-specific)".to_string(),
            "".to_string(),
            "## Next Steps".to_string(),
            "".to_string(),
            "1. Remove the installation media".to_string(),
            "2. Reboot your computer".to_string(),
            "3. Log in with your user account".to_string(),
            "4. Customize via /etc/nixos/configuration.nix".to_string(),
            "5. Run 'sudo nixos-rebuild switch' to apply changes".to_string(),
            "".to_string(),
            "## Learning Resources".to_string(),
            "".to_string(),
            "- NixOS Manual: https://nixos.org/manual/nixos/stable/".to_string(),
            "- Package Search: https://search.nixos.org/packages".to_string(),
            "- Community: https://discourse.nixos.org/".to_string(),
        ]
    }

    /// Build a summary step element with a green checkmark
    fn summary_step(label: &str, theme: &AppTheme) -> impl IntoElement {
        let icon: SharedString = icons::SUCCESS.into();
        let text: SharedString = label.to_string().into();
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing::SMALL))
            .child(
                div()
                    .text_color(theme.success)
                    .font_weight(FontWeight::BOLD)
                    .child(icon),
            )
            .child(div().text_color(theme.foreground).child(text))
    }

    /// Render this screen as a gpui element tree.
    pub fn view(&mut self) -> impl IntoElement {
        let theme = AppTheme::new();

        let mut col = div()
            .flex()
            .flex_col()
            .size_full()
            .gap(px(spacing::MEDIUM))
            .p(px(spacing::LARGE));

        // Success message
        let success_icon: SharedString =
            format!("{} Installation Complete!", icons::SUCCESS).into();
        col = col.child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(spacing::SMALL))
                .pt(px(spacing::LARGE))
                .child(
                    div()
                        .text_color(theme.success)
                        .font_weight(FontWeight::BOLD)
                        .text_xl()
                        .child(success_icon),
                )
                .child(
                    div()
                        .text_color(theme.foreground)
                        .text_sm()
                        .child("NixOS has been successfully installed on your system."),
                ),
        );

        // Installation summary block
        let summary_title: SharedString = " Installation Summary ".into();
        col = col.child(
            div()
                .w_full()
                .border_1()
                .border_color(theme.border)
                .rounded(px(4.0))
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    div()
                        .w_full()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .border_b_1()
                        .border_color(theme.border)
                        .text_color(theme.foreground)
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .child(summary_title),
                )
                .child(
                    div()
                        .p(px(spacing::MEDIUM))
                        .flex()
                        .flex_col()
                        .gap(px(spacing::SMALL))
                        .child(Self::summary_step(
                            "System partitioned and formatted",
                            &theme,
                        ))
                        .child(Self::summary_step("NixOS configuration generated", &theme))
                        .child(Self::summary_step("System packages installed", &theme))
                        .child(Self::summary_step("Bootloader configured", &theme))
                        .child(Self::summary_step("User account created", &theme))
                        .child(div().h(px(spacing::SMALL)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap(px(spacing::SMALL))
                                .child(
                                    div()
                                        .text_color(theme.muted)
                                        .text_sm()
                                        .child("Configuration:"),
                                )
                                .child(
                                    div()
                                        .text_color(theme.foreground)
                                        .text_sm()
                                        .child("/mnt/etc/nixos/configuration.nix"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap(px(spacing::SMALL))
                                .child(
                                    div()
                                        .text_color(theme.muted)
                                        .text_sm()
                                        .child("Installation log:"),
                                )
                                .child(
                                    div()
                                        .text_color(theme.foreground)
                                        .text_sm()
                                        .child("/tmp/ekaos-install.log"),
                                ),
                        ),
                ),
        );

        // Next steps block
        let steps_title: SharedString = " Next Steps ".into();
        col = col.child(
            div()
                .w_full()
                .border_1()
                .border_color(theme.border)
                .rounded(px(4.0))
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    div()
                        .w_full()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .border_b_1()
                        .border_color(theme.border)
                        .text_color(theme.foreground)
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .child(steps_title),
                )
                .child(
                    div()
                        .p(px(spacing::MEDIUM))
                        .flex()
                        .flex_col()
                        .gap(px(spacing::SMALL))
                        .child(
                            div()
                                .text_color(theme.foreground)
                                .child("1. Remove the installation media"),
                        )
                        .child(
                            div()
                                .text_color(theme.foreground)
                                .child("2. Reboot your computer"),
                        )
                        .child(
                            div()
                                .text_color(theme.foreground)
                                .child("3. Log in with your user account"),
                        )
                        .child(
                            div()
                                .text_color(theme.foreground)
                                .child("4. Customize your system:"),
                        )
                        .child(
                            div()
                                .text_color(theme.foreground)
                                .pl(px(spacing::LARGE))
                                .child("Edit /etc/nixos/configuration.nix"),
                        )
                        .child(
                            div()
                                .text_color(theme.foreground)
                                .pl(px(spacing::LARGE))
                                .child("Run 'sudo nixos-rebuild switch' to apply"),
                        )
                        .child(div().h(px(spacing::SMALL)))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap(px(spacing::SMALL))
                                .child(div().text_color(theme.foreground).child("Learn more:"))
                                .child(
                                    div()
                                        .text_color(theme.primary)
                                        .child("https://nixos.org/manual/nixos/stable/"),
                                ),
                        ),
                ),
        );

        // Spacer
        col = col.child(div().flex_1());

        // Navigation hints
        col = col.child(NavigationHints::new(vec![
            ("Enter", "Reboot Now"),
            ("q", "Exit to shell"),
        ]));

        col
    }
}

impl Default for SuccessScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_screen_creation() {
        let screen = SuccessScreen::new();
        assert_eq!(screen.selected, 1);
    }

    #[test]
    fn test_success_screen_default() {
        let screen = SuccessScreen::default();
        assert_eq!(screen.selected, 1);
    }

    #[test]
    fn test_success_screen_navigation() {
        let mut screen = SuccessScreen::new();
        assert_eq!(screen.selected, 1);

        screen.select_next();
        assert_eq!(screen.selected, 0);

        screen.select_next();
        assert_eq!(screen.selected, 1);

        screen.select_previous();
        assert_eq!(screen.selected, 0);

        screen.select_previous();
        assert_eq!(screen.selected, 1);
    }

    #[test]
    fn test_success_screen_selected_index() {
        let screen = SuccessScreen::new();
        assert_eq!(screen.selected_index(), 1);
    }

    #[test]
    fn test_success_screen_wraps_forward() {
        let mut screen = SuccessScreen::new();
        screen.selected = 1;
        screen.select_next();
        assert_eq!(screen.selected, 0);
    }

    #[test]
    fn test_success_screen_wraps_backward() {
        let mut screen = SuccessScreen::new();
        screen.selected = 0;
        screen.select_previous();
        assert_eq!(screen.selected, 1);
    }

    #[test]
    fn test_help_content() {
        let screen = SuccessScreen::new();
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("Installation Complete"));
    }

    #[test]
    fn test_help_content_covers_next_steps() {
        let screen = SuccessScreen::new();
        let help = screen.help_content();
        let joined = help.join("\n");
        assert!(joined.contains("Remove the installation media"));
        assert!(joined.contains("Reboot"));
        assert!(joined.contains("configuration.nix"));
    }

    #[test]
    fn test_help_content_covers_resources() {
        let screen = SuccessScreen::new();
        let help = screen.help_content();
        let joined = help.join("\n");
        assert!(joined.contains("nixos.org"));
        assert!(joined.contains("discourse.nixos.org"));
    }
}
