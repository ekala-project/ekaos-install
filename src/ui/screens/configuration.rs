//! System configuration screen
//!
//! Collects basic system configuration: hostname, username, password, timezone.
//! Implements `gpui::Render` for the gpui GUI framework.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, Hsla, IntoElement, SharedString};

use crate::config::{BootLoader, InstallConfig};
use crate::data::keymaps::get_keymap_list;
use crate::data::locales::get_locale_list;
use crate::data::timezones::{detect_timezone, get_timezone_list};
use crate::system::BootMode;
use crate::ui::components::{Button, ButtonStyle, FilterableSelectList};
use crate::ui::theme::{spacing, AppTheme};
use crate::ui::utils::NavigationHints;

/// Evaluate password strength and return a label and color (Hsla)
fn password_strength(password: &str) -> (&'static str, Hsla) {
    let len = password.len();
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    let variety = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    let theme = AppTheme::new();
    if len >= 12 && variety >= 3 {
        ("Strong", theme.success)
    } else if len >= 8 && variety >= 2 {
        ("Medium", theme.warning)
    } else {
        ("Weak", theme.error)
    }
}

/// Which field is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedField {
    Hostname,
    Username,
    Password,
    PasswordConfirm,
    Timezone,
    Keymap,
    Locale,
    ContinueButton,
}

/// Configuration screen state
pub struct ConfigurationScreen {
    /// Configuration being built
    config: InstallConfig,
    /// Hostname input value
    hostname_value: String,
    /// Username input value
    username_value: String,
    /// Password input value
    password_value: String,
    /// Password confirmation input value
    password_confirm_value: String,
    /// Timezone selector
    timezone_selector: FilterableSelectList<String>,
    /// Keymap selector
    keymap_selector: FilterableSelectList<String>,
    /// Locale selector
    locale_selector: FilterableSelectList<String>,
    /// Currently focused field
    focused_field: FocusedField,
    /// Validation error message
    error_message: Option<String>,
}

#[allow(dead_code)]
impl ConfigurationScreen {
    /// Create a new configuration screen
    pub fn new() -> Self {
        // Create timezone selector with all available timezones
        let timezones = get_timezone_list();
        let timezone_items: Vec<(String, String)> =
            timezones.into_iter().map(|tz| (tz.clone(), tz)).collect();
        let mut timezone_selector =
            FilterableSelectList::new("Timezone").with_items(timezone_items);
        // Try to auto-detect timezone, fall back to America/New_York
        let default_tz = detect_timezone().unwrap_or_else(|| "America/New_York".to_string());
        timezone_selector.set_selected_by_label(&default_tz);

        // Create keymap selector
        let keymap_items = get_keymap_list();
        let mut keymap_selector =
            FilterableSelectList::new("Keyboard Layout").with_items(keymap_items);
        keymap_selector.set_selected_by_label("us");

        // Create locale selector
        let locale_items = get_locale_list();
        let mut locale_selector = FilterableSelectList::new("Locale").with_items(locale_items);
        locale_selector.set_selected_by_label("en_US.UTF-8");

        Self {
            config: InstallConfig::default(),
            hostname_value: "nixos".to_string(),
            username_value: String::new(),
            password_value: String::new(),
            password_confirm_value: String::new(),
            timezone_selector,
            keymap_selector,
            locale_selector,
            focused_field: FocusedField::Hostname,
            error_message: None,
        }
    }

    /// Set boot mode to determine bootloader
    pub fn set_boot_mode(&mut self, boot_mode: BootMode) {
        self.config.bootloader = match boot_mode {
            BootMode::Uefi => BootLoader::SystemdBoot,
            BootMode::Bios => BootLoader::Grub,
            BootMode::Unknown => BootLoader::SystemdBoot,
        };
    }

    /// Set disk and partition configuration from partition planning screen
    pub fn set_disk_config(
        &mut self,
        disk_path: String,
        disk_size: u64,
        swap_size_gb: u64,
        root_filesystem: String,
        luks_encryption: bool,
        luks_passphrase: String,
    ) {
        self.config.disk_path = disk_path;
        self.config.disk_size = disk_size;
        self.config.swap_size_gb = swap_size_gb;
        self.config.root_filesystem = root_filesystem;
        self.config.luks_encryption = luks_encryption;
        self.config.luks_passphrase = luks_passphrase;
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &InstallConfig {
        &self.config
    }

    /// Focus next field
    fn focus_next(&mut self) {
        self.focused_field = match self.focused_field {
            FocusedField::Hostname => FocusedField::Username,
            FocusedField::Username => FocusedField::Password,
            FocusedField::Password => FocusedField::PasswordConfirm,
            FocusedField::PasswordConfirm => FocusedField::Timezone,
            FocusedField::Timezone => FocusedField::Keymap,
            FocusedField::Keymap => FocusedField::Locale,
            FocusedField::Locale => FocusedField::ContinueButton,
            FocusedField::ContinueButton => FocusedField::Hostname,
        };
    }

    /// Focus previous field
    fn focus_previous(&mut self) {
        self.focused_field = match self.focused_field {
            FocusedField::Hostname => FocusedField::ContinueButton,
            FocusedField::Username => FocusedField::Hostname,
            FocusedField::Password => FocusedField::Username,
            FocusedField::PasswordConfirm => FocusedField::Password,
            FocusedField::Timezone => FocusedField::PasswordConfirm,
            FocusedField::Keymap => FocusedField::Timezone,
            FocusedField::Locale => FocusedField::Keymap,
            FocusedField::ContinueButton => FocusedField::Locale,
        };
    }

    /// Update configuration from input fields
    fn update_config(&mut self) {
        self.config.hostname = self.hostname_value.clone();
        self.config.user.username = self.username_value.clone();
        self.config.user.password = self.password_value.clone();
        if let Some(timezone) = self.timezone_selector.selected_value() {
            self.config.timezone = timezone.clone();
        }
        if let Some(keymap) = self.keymap_selector.selected_value() {
            self.config.keymap = keymap.clone();
        }
        if let Some(locale) = self.locale_selector.selected_value() {
            self.config.locale = locale.clone();
        }
    }

    /// Validate all inputs
    fn validate(&mut self) -> bool {
        self.error_message = None;
        self.update_config();

        // Check password match
        if self.password_value != self.password_confirm_value {
            self.error_message = Some("Passwords do not match".to_string());
            return false;
        }

        // Validate config
        if let Err(e) = self.config.validate() {
            self.error_message = Some(e);
            return false;
        }

        true
    }

    /// Whether this screen can proceed (basic requirements met)
    pub fn can_proceed(&self) -> bool {
        !self.hostname_value.is_empty()
            && !self.username_value.is_empty()
            && !self.password_value.is_empty()
            && !self.password_confirm_value.is_empty()
    }

    /// Whether this screen can go back
    pub fn can_go_back(&self) -> bool {
        true
    }

    /// Title of this screen
    pub fn title(&self) -> &str {
        "System Configuration"
    }

    /// Help content for this screen
    pub fn help_content(&self) -> Vec<String> {
        vec![
            "# System Configuration Screen".to_string(),
            "".to_string(),
            "Configure basic system settings for your NixOS installation.".to_string(),
            "".to_string(),
            "## Required Fields".to_string(),
            "".to_string(),
            "- Hostname: Name for your computer on the network".to_string(),
            "- Username: Your user account name".to_string(),
            "- Password: Your account password (min 6 characters)".to_string(),
            "".to_string(),
            "## Additional Settings".to_string(),
            "".to_string(),
            "- Timezone: Select your timezone".to_string(),
            "- Keyboard Layout: Select your console keymap".to_string(),
            "- Locale: Select your system locale".to_string(),
        ]
    }

    /// Build a labeled input field element
    fn input_field_element(
        label: &str,
        value: &str,
        focused: bool,
        is_password: bool,
        theme: &AppTheme,
    ) -> impl IntoElement {
        let border_color = if focused { theme.primary } else { theme.border };
        let label_text: SharedString = label.to_string().into();
        let display_text: SharedString = if is_password && !value.is_empty() {
            if focused {
                format!("{}{}", "\u{2022}".repeat(value.len()), "\u{2588}").into()
            } else {
                "\u{2022}".repeat(value.len()).into()
            }
        } else if value.is_empty() {
            if focused {
                "\u{2588}".to_string().into()
            } else {
                SharedString::default()
            }
        } else if focused {
            format!("{}\u{2588}", value).into()
        } else {
            value.to_string().into()
        };

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(spacing::SMALL))
            .child(
                div()
                    .text_color(border_color)
                    .text_sm()
                    .font_weight(if focused {
                        FontWeight::BOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .child(label_text),
            )
            .child(
                div()
                    .w_full()
                    .border_1()
                    .border_color(border_color)
                    .rounded(px(4.0))
                    .px(px(spacing::MEDIUM))
                    .py(px(spacing::SMALL))
                    .text_color(if value.is_empty() {
                        theme.muted
                    } else {
                        theme.foreground
                    })
                    .child(display_text),
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

        // Instructions
        col = col.child(div().w_full().flex().justify_center().child(
            div().text_color(theme.foreground).text_sm().child(
                "Configure your NixOS system. Use Tab to move \
                         between fields.",
            ),
        ));

        // Hostname
        col = col.child(Self::input_field_element(
            "Hostname",
            &self.hostname_value.clone(),
            self.focused_field == FocusedField::Hostname,
            false,
            &theme,
        ));

        // Username
        col = col.child(Self::input_field_element(
            "Username",
            &self.username_value.clone(),
            self.focused_field == FocusedField::Username,
            false,
            &theme,
        ));

        // Password
        col = col.child(Self::input_field_element(
            "Password",
            &self.password_value.clone(),
            self.focused_field == FocusedField::Password,
            true,
            &theme,
        ));

        // Password strength indicator
        if !self.password_value.is_empty() {
            let (label, color) = password_strength(&self.password_value);
            let strength_text: SharedString = format!("  Strength: {}", label).into();
            col = col.child(div().text_color(color).text_sm().child(strength_text));
        }

        // Password confirm
        col = col.child(Self::input_field_element(
            "Confirm Password",
            &self.password_confirm_value.clone(),
            self.focused_field == FocusedField::PasswordConfirm,
            true,
            &theme,
        ));

        // Timezone selector
        col = col.child(
            self.timezone_selector
                .view(self.focused_field == FocusedField::Timezone),
        );

        // Keymap selector
        col = col.child(
            self.keymap_selector
                .view(self.focused_field == FocusedField::Keymap),
        );

        // Locale selector
        col = col.child(
            self.locale_selector
                .view(self.focused_field == FocusedField::Locale),
        );

        // Continue button
        col = col.child(
            div().w_full().flex().justify_center().child(
                Button::new("Continue")
                    .with_style(ButtonStyle::Primary)
                    .focused(self.focused_field == FocusedField::ContinueButton)
                    .enabled(self.can_proceed()),
            ),
        );

        // Error message
        if let Some(ref error) = self.error_message {
            let error_text: SharedString = error.clone().into();
            col = col.child(
                div()
                    .w_full()
                    .border_1()
                    .border_color(theme.error)
                    .rounded(px(4.0))
                    .p(px(spacing::MEDIUM))
                    .flex()
                    .justify_center()
                    .child(div().text_color(theme.error).text_sm().child(error_text)),
            );
        }

        // Spacer
        col = col.child(div().flex_1());

        // Navigation hints
        col = col.child(NavigationHints::new(vec![
            ("Up/Down/Left/Right", "Navigate fields"),
            ("Tab", "Next field"),
            ("Enter", "Next / Submit"),
            ("Esc", "Back / Quit"),
        ]));

        col
    }
}

impl Default for ConfigurationScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_screen_creation() {
        let screen = ConfigurationScreen::new();
        assert_eq!(screen.title(), "System Configuration");
        assert!(screen.can_go_back());
    }

    #[test]
    fn test_focus_navigation() {
        let mut screen = ConfigurationScreen::new();
        assert_eq!(screen.focused_field, FocusedField::Hostname);

        screen.focus_next();
        assert_eq!(screen.focused_field, FocusedField::Username);

        screen.focus_next();
        assert_eq!(screen.focused_field, FocusedField::Password);

        screen.focus_previous();
        assert_eq!(screen.focused_field, FocusedField::Username);
    }

    #[test]
    fn test_bootloader_selection() {
        let mut screen = ConfigurationScreen::new();

        screen.set_boot_mode(BootMode::Uefi);
        assert_eq!(screen.config.bootloader, BootLoader::SystemdBoot);

        screen.set_boot_mode(BootMode::Bios);
        assert_eq!(screen.config.bootloader, BootLoader::Grub);
    }

    #[test]
    fn test_help_content() {
        let screen = ConfigurationScreen::new();
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("System Configuration"));
    }

    #[test]
    fn test_arrow_key_navigation() {
        let mut screen = ConfigurationScreen::new();
        assert_eq!(screen.focused_field, FocusedField::Hostname);

        // Down moves to next field
        screen.focus_next();
        assert_eq!(screen.focused_field, FocusedField::Username);

        screen.focus_next();
        assert_eq!(screen.focused_field, FocusedField::Password);

        screen.focus_next();
        assert_eq!(screen.focused_field, FocusedField::PasswordConfirm);

        // Up moves to previous field
        screen.focus_previous();
        assert_eq!(screen.focused_field, FocusedField::Password);

        screen.focus_previous();
        assert_eq!(screen.focused_field, FocusedField::Username);

        screen.focus_previous();
        assert_eq!(screen.focused_field, FocusedField::Hostname);
    }

    #[test]
    fn test_set_disk_config() {
        let mut screen = ConfigurationScreen::new();

        // Initially, disk config should be empty/default
        assert_eq!(screen.config.disk_path, "");
        assert_eq!(screen.config.disk_size, 0);
        assert_eq!(screen.config.swap_size_gb, 8);
        assert_eq!(screen.config.root_filesystem, "ext4");

        // Set disk config
        screen.set_disk_config(
            "/dev/nvme0n1".to_string(),
            1_000_000_000_000,
            16,
            "btrfs".to_string(),
            true,
            "mypassphrase".to_string(),
        );

        // Verify config was updated
        assert_eq!(screen.config.disk_path, "/dev/nvme0n1");
        assert_eq!(screen.config.disk_size, 1_000_000_000_000);
        assert_eq!(screen.config.swap_size_gb, 16);
        assert_eq!(screen.config.root_filesystem, "btrfs");
        assert!(screen.config.luks_encryption);
        assert_eq!(screen.config.luks_passphrase, "mypassphrase");
    }

    #[test]
    fn test_can_proceed_empty() {
        let screen = ConfigurationScreen::new();
        // Username, password, password_confirm are empty
        assert!(!screen.can_proceed());
    }

    #[test]
    fn test_can_proceed_filled() {
        let mut screen = ConfigurationScreen::new();
        screen.hostname_value = "myhost".to_string();
        screen.username_value = "user".to_string();
        screen.password_value = "password".to_string();
        screen.password_confirm_value = "password".to_string();
        assert!(screen.can_proceed());
    }

    #[test]
    fn test_update_config() {
        let mut screen = ConfigurationScreen::new();
        screen.hostname_value = "myhost".to_string();
        screen.username_value = "testuser".to_string();
        screen.password_value = "mypassword".to_string();
        screen.update_config();

        assert_eq!(screen.config.hostname, "myhost");
        assert_eq!(screen.config.user.username, "testuser");
        assert_eq!(screen.config.user.password, "mypassword");
    }

    #[test]
    fn test_validate_password_mismatch() {
        let mut screen = ConfigurationScreen::new();
        screen.hostname_value = "myhost".to_string();
        screen.username_value = "testuser".to_string();
        screen.password_value = "password123".to_string();
        screen.password_confirm_value = "different".to_string();

        assert!(!screen.validate());
        assert_eq!(
            screen.error_message,
            Some("Passwords do not match".to_string())
        );
    }

    #[test]
    fn test_validate_success() {
        let mut screen = ConfigurationScreen::new();
        screen.hostname_value = "myhost".to_string();
        screen.username_value = "testuser".to_string();
        screen.password_value = "password123".to_string();
        screen.password_confirm_value = "password123".to_string();

        assert!(screen.validate());
        assert!(screen.error_message.is_none());
    }

    #[test]
    fn test_password_strength_weak() {
        let (label, color) = password_strength("abc");
        assert_eq!(label, "Weak");
        let theme = AppTheme::new();
        assert!((color.h - theme.error.h).abs() < f32::EPSILON);
    }

    #[test]
    fn test_password_strength_medium() {
        let (label, color) = password_strength("Abcdef12");
        assert_eq!(label, "Medium");
        let theme = AppTheme::new();
        assert!((color.h - theme.warning.h).abs() < f32::EPSILON);
    }

    #[test]
    fn test_password_strength_strong() {
        let (label, color) = password_strength("Abcdef12!@#x");
        assert_eq!(label, "Strong");
        let theme = AppTheme::new();
        assert!((color.h - theme.success.h).abs() < f32::EPSILON);
    }

    #[test]
    fn test_focus_wraps_forward() {
        let mut screen = ConfigurationScreen::new();
        screen.focused_field = FocusedField::ContinueButton;
        screen.focus_next();
        assert_eq!(screen.focused_field, FocusedField::Hostname);
    }

    #[test]
    fn test_focus_wraps_backward() {
        let mut screen = ConfigurationScreen::new();
        screen.focused_field = FocusedField::Hostname;
        screen.focus_previous();
        assert_eq!(screen.focused_field, FocusedField::ContinueButton);
    }

    #[test]
    fn test_default_hostname() {
        let screen = ConfigurationScreen::new();
        assert_eq!(screen.hostname_value, "nixos");
    }

    #[test]
    fn test_boot_mode_unknown() {
        let mut screen = ConfigurationScreen::new();
        screen.set_boot_mode(BootMode::Unknown);
        assert_eq!(screen.config.bootloader, BootLoader::SystemdBoot);
    }

    #[test]
    fn test_get_config() {
        let screen = ConfigurationScreen::new();
        let config = screen.get_config();
        assert_eq!(config.hostname, "nixos");
    }

    #[test]
    fn test_default_impl() {
        let screen = ConfigurationScreen::default();
        assert_eq!(screen.focused_field, FocusedField::Hostname);
        assert!(screen.error_message.is_none());
    }
}
