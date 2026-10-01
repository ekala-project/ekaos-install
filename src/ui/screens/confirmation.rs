//! Confirmation screen for reviewing configuration before installation
//!
//! This screen displays all collected configuration and requires explicit
//! user confirmation before proceeding to the installation phase.
//! Implements `gpui::Render` for the gpui GUI framework.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement, SharedString};

use crate::config::InstallConfig;
use crate::ui::theme::{icons, spacing, AppTheme};
use crate::ui::utils::NavigationHints;

/// Confirmation screen state
pub struct ConfirmationScreen {
    /// Configuration to display and confirm
    config: Option<InstallConfig>,
    /// Confirmation input value - must type "DELETE" to proceed
    confirm_value: String,
    /// Whether the confirmation gate is satisfied
    confirmed: bool,
}

#[allow(dead_code)]
impl ConfirmationScreen {
    /// Create a new confirmation screen
    pub fn new() -> Self {
        Self {
            config: None,
            confirm_value: String::new(),
            confirmed: false,
        }
    }

    /// Set the configuration to display
    pub fn set_config(&mut self, config: InstallConfig) {
        self.config = Some(config);
        // Reset confirmation state when config changes
        self.confirm_value.clear();
        self.confirmed = false;
    }

    /// Get the confirmed configuration
    pub fn get_config(&self) -> Option<&InstallConfig> {
        self.config.as_ref()
    }

    /// Check if user has typed "DELETE"
    fn check_confirmation(&mut self) {
        self.confirmed = self.confirm_value == "DELETE";
    }

    /// Add a character to the confirmation input
    pub fn add_char(&mut self, c: char) {
        self.confirm_value.push(c);
        self.check_confirmation();
    }

    /// Remove the last character from the confirmation input
    pub fn remove_char(&mut self) {
        self.confirm_value.pop();
        self.check_confirmation();
    }

    /// Whether this screen can proceed (config present and confirmed)
    pub fn can_proceed(&self) -> bool {
        self.config.is_some() && self.confirmed
    }

    /// Whether this screen can go back
    pub fn can_go_back(&self) -> bool {
        true
    }

    /// Title of this screen
    pub fn title(&self) -> &str {
        "Review Configuration"
    }

    /// Help content for this screen
    pub fn help_content(&self) -> Vec<String> {
        vec![
            "# Review Configuration Screen".to_string(),
            "".to_string(),
            "This screen displays all the configuration you have entered.".to_string(),
            "Please review carefully before proceeding.".to_string(),
            "".to_string(),
            "# Confirmation Required".to_string(),
            "".to_string(),
            "You must type DELETE (all caps) to confirm that you understand".to_string(),
            "all data on the selected disk will be permanently erased.".to_string(),
            "".to_string(),
            "# What Happens Next".to_string(),
            "".to_string(),
            "If you confirm, the installer will:".to_string(),
            "".to_string(),
            "1. Format the selected disk".to_string(),
            "2. Create partitions (EFI/Boot, Swap, Root)".to_string(),
            "3. Install NixOS base system".to_string(),
            "4. Configure the bootloader".to_string(),
            "5. Set up your user account".to_string(),
            "6. Apply system settings".to_string(),
            "".to_string(),
            "# Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- Type DELETE then Enter: Confirm and begin installation".to_string(),
            "- Left / Backspace: Go back to configuration".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q / Esc: Quit the installer".to_string(),
        ]
    }

    /// Build a labeled info row (label: value)
    fn info_row(label: &str, value: &str, theme: &AppTheme) -> impl IntoElement {
        let label_text: SharedString = format!("  {}  ", label).into();
        let value_text: SharedString = value.to_string().into();

        div()
            .flex()
            .flex_row()
            .items_center()
            .child(div().text_color(theme.muted).text_sm().child(label_text))
            .child(
                div()
                    .text_color(theme.foreground)
                    .text_sm()
                    .child(value_text),
            )
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

        // Title
        col = col.child(
            div()
                .w_full()
                .flex()
                .justify_center()
                .pt(px(spacing::MEDIUM))
                .child(
                    div()
                        .text_color(theme.primary)
                        .font_weight(FontWeight::BOLD)
                        .text_xl()
                        .child("Review Configuration"),
                ),
        );

        // If no config, show error
        let Some(config) = &self.config else {
            col = col.child(
                div().w_full().flex().justify_center().child(
                    div()
                        .text_color(theme.error)
                        .child("No configuration available"),
                ),
            );
            return col;
        };

        // System Settings section
        let system_title: SharedString = " System Settings ".into();
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
                        .child(system_title),
                )
                .child(
                    div()
                        .py(px(spacing::SMALL))
                        .flex()
                        .flex_col()
                        .child(Self::info_row("Hostname: ", &config.hostname, &theme))
                        .child(Self::info_row("Timezone: ", &config.timezone, &theme))
                        .child(Self::info_row("Locale:   ", &config.locale, &theme))
                        .child(Self::info_row("Keymap:   ", &config.keymap, &theme)),
                ),
        );

        // User Account section
        let user_title: SharedString = " User Account ".into();
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
                        .child(user_title),
                )
                .child(div().py(px(spacing::SMALL)).child(Self::info_row(
                    "Username: ",
                    &config.user.username,
                    &theme,
                ))),
        );

        // Disk & Partitioning section
        let disk_title: SharedString = " Disk & Partitioning ".into();
        let disk_size_gb = config.disk_size as f64 / 1_000_000_000.0;
        let disk_display = format!("{} ({:.1} GB)", config.disk_path, disk_size_gb);
        let swap_display = format!("{} GB", config.swap_size_gb);
        let encryption_status = if config.luks_encryption {
            "Yes (LUKS)"
        } else {
            "No"
        };

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
                        .child(disk_title),
                )
                .child(
                    div()
                        .py(px(spacing::SMALL))
                        .flex()
                        .flex_col()
                        .child(Self::info_row("Disk:      ", &disk_display, &theme))
                        .child(Self::info_row(
                            "Filesystem:",
                            &config.root_filesystem,
                            &theme,
                        ))
                        .child(Self::info_row("Swap:      ", &swap_display, &theme))
                        .child(Self::info_row("Encryption:", encryption_status, &theme)),
                ),
        );

        // Bootloader section
        let boot_title: SharedString = " Bootloader ".into();
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
                        .child(boot_title),
                )
                .child(div().py(px(spacing::SMALL)).child(Self::info_row(
                    "Bootloader:",
                    config.bootloader.as_str(),
                    &theme,
                ))),
        );

        // Warning message
        let warning_icon: SharedString = icons::WARNING.into();
        col = col.child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(spacing::SMALL))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(spacing::SMALL))
                        .child(
                            div()
                                .text_color(theme.error)
                                .font_weight(FontWeight::BOLD)
                                .child(warning_icon),
                        )
                        .child(
                            div()
                                .text_color(theme.error)
                                .font_weight(FontWeight::BOLD)
                                .child(
                                    "ALL DATA ON THIS DISK WILL BE PERMANENTLY \
                                 ERASED",
                                ),
                        ),
                )
                .child(
                    div()
                        .text_color(theme.warning)
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .child(
                            "Type DELETE below to confirm and begin \
                             installation",
                        ),
                ),
        );

        // Confirmation input
        let confirm_display: SharedString = if self.confirm_value.is_empty() {
            "\u{2588}".to_string().into()
        } else {
            format!("{}\u{2588}", self.confirm_value).into()
        };

        let confirm_border = if self.confirmed {
            theme.success
        } else {
            theme.primary
        };

        col = col.child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(spacing::SMALL))
                .child(
                    div()
                        .text_color(confirm_border)
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .child("Type DELETE to confirm"),
                )
                .child(
                    div()
                        .w_full()
                        .border_1()
                        .border_color(confirm_border)
                        .rounded(px(4.0))
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .text_color(theme.foreground)
                        .child(confirm_display),
                ),
        );

        // Spacer
        col = col.child(div().flex_1());

        // Navigation hints
        let hints = if self.confirmed {
            vec![
                ("Enter", "BEGIN INSTALL"),
                ("Left", "Go Back"),
                ("?", "Help"),
            ]
        } else {
            vec![
                ("Type DELETE", "to confirm"),
                ("Left", "Go Back"),
                ("?", "Help"),
            ]
        };

        col = col.child(NavigationHints::new(hints));

        col
    }
}

impl Default for ConfirmationScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BootLoader, UserConfig};

    fn create_test_config() -> InstallConfig {
        InstallConfig {
            hostname: "test-nixos".to_string(),
            user: UserConfig {
                username: "testuser".to_string(),
                full_name: Some("Test User".to_string()),
                is_admin: true,
                password: "testpass123".to_string(),
            },
            timezone: "America/New_York".to_string(),
            locale: "en_US.UTF-8".to_string(),
            keymap: "us".to_string(),
            bootloader: BootLoader::SystemdBoot,
            network_manager: true,
            disk_path: "/dev/sda".to_string(),
            disk_size: 500_000_000_000,
            swap_size_gb: 8,
            root_filesystem: "ext4".to_string(),
            luks_encryption: false,
            luks_passphrase: String::new(),
        }
    }

    #[test]
    fn test_confirmation_screen_creation() {
        let screen = ConfirmationScreen::new();
        assert_eq!(screen.title(), "Review Configuration");
        assert!(screen.can_go_back());
        assert!(!screen.can_proceed());
        assert!(screen.config.is_none());
        assert!(screen.confirm_value.is_empty());
        assert!(!screen.confirmed);
    }

    #[test]
    fn test_set_config() {
        let mut screen = ConfirmationScreen::new();
        let config = create_test_config();
        screen.set_config(config);

        assert!(screen.config.is_some());
        assert_eq!(screen.get_config().unwrap().hostname, "test-nixos");
    }

    #[test]
    fn test_check_confirmation() {
        let mut screen = ConfirmationScreen::new();
        screen.set_config(create_test_config());

        // Initially not confirmed
        assert!(!screen.confirmed);
        assert!(!screen.can_proceed());

        // Type "DELETE"
        for c in "DELETE".chars() {
            screen.add_char(c);
        }

        assert!(screen.confirmed);
        assert!(screen.can_proceed());
    }

    #[test]
    fn test_partial_confirmation() {
        let mut screen = ConfirmationScreen::new();
        screen.set_config(create_test_config());

        // Type partial
        for c in "DEL".chars() {
            screen.add_char(c);
        }

        assert!(!screen.confirmed);
        assert!(!screen.can_proceed());
    }

    #[test]
    fn test_wrong_confirmation() {
        let mut screen = ConfirmationScreen::new();
        screen.set_config(create_test_config());

        for c in "delete".chars() {
            screen.add_char(c);
        }

        assert!(!screen.confirmed);
    }

    #[test]
    fn test_add_and_remove_char() {
        let mut screen = ConfirmationScreen::new();
        screen.set_config(create_test_config());

        screen.add_char('D');
        screen.add_char('E');
        assert_eq!(screen.confirm_value, "DE");

        screen.remove_char();
        assert_eq!(screen.confirm_value, "D");

        screen.remove_char();
        assert_eq!(screen.confirm_value, "");

        // Remove on empty is safe
        screen.remove_char();
        assert_eq!(screen.confirm_value, "");
    }

    #[test]
    fn test_set_config_resets_confirmation() {
        let mut screen = ConfirmationScreen::new();
        screen.set_config(create_test_config());

        for c in "DELETE".chars() {
            screen.add_char(c);
        }
        assert!(screen.confirmed);

        // Setting new config resets confirmation
        screen.set_config(create_test_config());
        assert!(!screen.confirmed);
        assert!(screen.confirm_value.is_empty());
    }

    #[test]
    fn test_can_proceed_no_config() {
        let mut screen = ConfirmationScreen::new();
        // Even if somehow confirmed without config
        screen.confirmed = true;
        assert!(!screen.can_proceed());
    }

    #[test]
    fn test_get_config_none() {
        let screen = ConfirmationScreen::new();
        assert!(screen.get_config().is_none());
    }

    #[test]
    fn test_help_content() {
        let screen = ConfirmationScreen::new();
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("Review Configuration"));
    }

    #[test]
    fn test_help_content_mentions_delete() {
        let screen = ConfirmationScreen::new();
        let help = screen.help_content();
        let joined = help.join("\n");
        assert!(joined.contains("DELETE"));
    }

    #[test]
    fn test_default_impl() {
        let screen = ConfirmationScreen::default();
        assert!(screen.config.is_none());
        assert!(!screen.confirmed);
    }

    #[test]
    fn test_confirmation_with_luks_config() {
        let mut screen = ConfirmationScreen::new();
        let mut config = create_test_config();
        config.luks_encryption = true;
        config.luks_passphrase = "secretpass".to_string();
        screen.set_config(config);

        assert!(screen.get_config().unwrap().luks_encryption);
    }
}
