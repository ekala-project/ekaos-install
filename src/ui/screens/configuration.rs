//! System configuration screen
//!
//! Collects basic system configuration: hostname, username, password, timezone.

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tracing::debug;

use crate::config::{BootLoader, InstallConfig};
use crate::data::keymaps::get_keymap_list;
use crate::data::locales::get_locale_list;
use crate::data::timezones::{detect_timezone, get_timezone_list};
use crate::system::BootMode;
use crate::ui::{
    components::{Button, Component, Focusable, FilterableSelectList, InputField, Interactive},
    theme::AppTheme,
    utils::{keycode_to_input_event, render_navigation_hints},
};

use super::{Screen, ScreenAction};

/// Evaluate password strength and return a label and color
fn password_strength(password: &str) -> (&'static str, Color) {
    let len = password.len();
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    let variety = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    if len >= 12 && variety >= 3 {
        ("Strong ███████████", Color::Green)
    } else if len >= 8 && variety >= 2 {
        ("Medium ███████░░░░", Color::Yellow)
    } else {
        ("Weak   ███░░░░░░░░", Color::Red)
    }
}

/// Configuration screen state
pub struct ConfigurationScreen {
    /// Application theme
    theme: AppTheme,
    /// Configuration being built
    config: InstallConfig,
    /// Input fields
    hostname_input: InputField,
    username_input: InputField,
    password_input: InputField,
    password_confirm_input: InputField,
    timezone_selector: FilterableSelectList<String>,
    keymap_selector: FilterableSelectList<String>,
    locale_selector: FilterableSelectList<String>,
    continue_button: Button,
    /// Currently focused field
    focused_field: FocusedField,
    /// Validation error message
    error_message: Option<String>,
}

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

impl ConfigurationScreen {
    /// Create a new configuration screen
    pub fn new() -> Self {
        let mut hostname_input = InputField::new("Hostname");
        hostname_input.set_value("nixos".to_string());
        hostname_input.set_focused(true);

        let username_input = InputField::new("Username");
        let password_input = InputField::new("Password").password();
        let password_confirm_input = InputField::new("Confirm Password").password();

        // Create timezone selector with all available timezones
        let timezones = get_timezone_list();
        let timezone_items: Vec<(String, String)> = timezones
            .into_iter()
            .map(|tz| (tz.clone(), tz))
            .collect();
        let mut timezone_selector = FilterableSelectList::new("Timezone")
            .with_items(timezone_items);
        // Try to auto-detect timezone, fall back to America/New_York
        let default_tz = detect_timezone().unwrap_or_else(|| "America/New_York".to_string());
        timezone_selector.set_selected_by_label(&default_tz);

        // Create keymap selector
        let keymap_items = get_keymap_list();
        let mut keymap_selector = FilterableSelectList::new("Keyboard Layout")
            .with_items(keymap_items);
        keymap_selector.set_selected_by_label("us");

        // Create locale selector
        let locale_items = get_locale_list();
        let mut locale_selector = FilterableSelectList::new("Locale")
            .with_items(locale_items);
        locale_selector.set_selected_by_label("en_US.UTF-8");

        let continue_button = Button::new("Continue");

        Self {
            theme: AppTheme::new(),
            config: InstallConfig::default(),
            hostname_input,
            username_input,
            password_input,
            password_confirm_input,
            timezone_selector,
            keymap_selector,
            locale_selector,
            continue_button,
            focused_field: FocusedField::Hostname,
            error_message: None,
        }
    }

    /// Set boot mode to determine bootloader
    pub fn set_boot_mode(&mut self, boot_mode: BootMode) {
        self.config.bootloader = match boot_mode {
            BootMode::Uefi => BootLoader::SystemdBoot,
            BootMode::Bios => BootLoader::Grub,
            BootMode::Unknown => BootLoader::SystemdBoot, // Default to systemd-boot
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
        self.clear_focus();
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
        self.set_focus();
    }

    /// Focus previous field
    fn focus_previous(&mut self) {
        self.clear_focus();
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
        self.set_focus();
    }

    /// Clear focus from all fields
    fn clear_focus(&mut self) {
        self.hostname_input.set_focused(false);
        self.username_input.set_focused(false);
        self.password_input.set_focused(false);
        self.password_confirm_input.set_focused(false);
        self.timezone_selector.set_focused(false);
        self.keymap_selector.set_focused(false);
        self.locale_selector.set_focused(false);
        self.continue_button.set_focused(false);
    }

    /// Set focus on current field
    fn set_focus(&mut self) {
        match self.focused_field {
            FocusedField::Hostname => self.hostname_input.set_focused(true),
            FocusedField::Username => self.username_input.set_focused(true),
            FocusedField::Password => self.password_input.set_focused(true),
            FocusedField::PasswordConfirm => self.password_confirm_input.set_focused(true),
            FocusedField::Timezone => self.timezone_selector.set_focused(true),
            FocusedField::Keymap => self.keymap_selector.set_focused(true),
            FocusedField::Locale => self.locale_selector.set_focused(true),
            FocusedField::ContinueButton => self.continue_button.set_focused(true),
        }
    }

    /// Update configuration from input fields
    fn update_config(&mut self) {
        self.config.hostname = self.hostname_input.value().to_string();
        self.config.user.username = self.username_input.value().to_string();
        self.config.user.password = self.password_input.value().to_string();
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
        if self.password_input.value() != self.password_confirm_input.value() {
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
}

impl Default for ConfigurationScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for ConfigurationScreen {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Instructions
                Constraint::Length(3),  // Hostname
                Constraint::Length(3),  // Username
                Constraint::Length(3),  // Password
                Constraint::Length(1),  // Password strength
                Constraint::Length(3),  // Confirm password
                Constraint::Length(8),  // Timezone selector
                Constraint::Length(8),  // Keymap selector
                Constraint::Length(8),  // Locale selector
                Constraint::Length(3),  // Continue button
                Constraint::Length(3),  // Error/status
                Constraint::Min(0),     // Spacer
                Constraint::Length(3),  // Navigation
            ])
            .split(area);

        // Instructions
        let instructions = Paragraph::new(vec![
            Line::from(""),
            Line::from("Configure your NixOS system. Use Tab to move between fields."),
        ])
        .style(self.theme.text())
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(instructions, chunks[0]);

        // Input fields
        self.hostname_input.render(frame, chunks[1]);
        self.username_input.render(frame, chunks[2]);
        self.password_input.render(frame, chunks[3]);

        // Password strength indicator
        let password = self.password_input.value();
        let strength_line = if password.is_empty() {
            Line::from("")
        } else {
            let (label, color) = password_strength(password);
            Line::from(vec![
                Span::raw("  Strength: "),
                Span::styled(label, Style::default().fg(color)),
            ])
        };
        frame.render_widget(Paragraph::new(strength_line), chunks[4]);

        self.password_confirm_input.render(frame, chunks[5]);
        self.timezone_selector.render(frame, chunks[6]);
        self.keymap_selector.render(frame, chunks[7]);
        self.locale_selector.render(frame, chunks[8]);

        // Continue button
        self.continue_button.render(frame, chunks[9]);

        // Error message or status
        if let Some(ref error) = self.error_message {
            let error_block = Block::default()
                .borders(Borders::ALL)
                .border_style(self.theme.error());

            let error_para = Paragraph::new(error.as_str())
                .block(error_block)
                .style(self.theme.error())
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(error_para, chunks[10]);
        }

        // Navigation hints
        render_navigation_hints(
            frame,
            &[
                ("↑↓←→", "Navigate fields"),
                ("Tab", "Next field"),
                ("Enter", "Next / Submit"),
                ("Esc", "Back / Quit"),
            ],
            &self.theme,
            chunks[12],
        );
    }

    fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
        // Try standard handlers first (quit, help), but only when not in an input field
        if !self.hostname_input.is_focused()
            && !self.username_input.is_focused()
            && !self.password_input.is_focused()
            && !self.password_confirm_input.is_focused()
            && !self.timezone_selector.is_focused()
            && !self.keymap_selector.is_focused()
            && !self.locale_selector.is_focused()
        {
            if let Some(action) = self.handle_standard_input(key) {
                return action;
            }
            // Try back handler (only when not in input fields)
            if let Some(action) = self.handle_back_input(key) {
                return action;
            }
        }

        // Handle screen-specific keys
        match key {
            KeyCode::Tab => {
                self.focus_next();
                ScreenAction::None
            }
            KeyCode::BackTab => {
                self.focus_previous();
                ScreenAction::None
            }
            KeyCode::Left => {
                // Left arrow always goes to previous field
                self.focus_previous();
                ScreenAction::None
            }
            KeyCode::Right => {
                // Right arrow always goes to next field
                self.focus_next();
                ScreenAction::None
            }
            KeyCode::Down if !self.timezone_selector.is_focused()
                && !self.keymap_selector.is_focused()
                && !self.locale_selector.is_focused() => {
                // Only use Down for navigation if not in a selector
                self.focus_next();
                ScreenAction::None
            }
            KeyCode::Up if !self.timezone_selector.is_focused()
                && !self.keymap_selector.is_focused()
                && !self.locale_selector.is_focused() => {
                // Only use Up for navigation if not in a selector
                self.focus_previous();
                ScreenAction::None
            }
            KeyCode::Enter => {
                // Enter behavior depends on which field is focused
                match self.focused_field {
                    FocusedField::ContinueButton => {
                        // Only submit when Continue button is focused
                        if self.validate() {
                            debug!("Configuration validated successfully");
                            ScreenAction::Next
                        } else {
                            ScreenAction::None
                        }
                    }
                    _ => {
                        // For all other fields, Enter moves to next field
                        self.focus_next();
                        ScreenAction::None
                    }
                }
            }
            _ => {
                // Convert KeyCode to InputEvent and pass to focused field
                if let Some(event) = keycode_to_input_event(key) {
                    match self.focused_field {
                        FocusedField::Hostname => {
                            self.hostname_input.handle_input(event);
                        }
                        FocusedField::Username => {
                            self.username_input.handle_input(event);
                        }
                        FocusedField::Password => {
                            self.password_input.handle_input(event);
                        }
                        FocusedField::PasswordConfirm => {
                            self.password_confirm_input.handle_input(event);
                        }
                        FocusedField::Timezone => {
                            self.timezone_selector.handle_input(event);
                            self.update_config();
                        }
                        FocusedField::Keymap => {
                            self.keymap_selector.handle_input(event);
                            self.update_config();
                        }
                        FocusedField::Locale => {
                            self.locale_selector.handle_input(event);
                            self.update_config();
                        }
                        FocusedField::ContinueButton => {
                            // Button doesn't need other input handling
                        }
                    }
                }
                ScreenAction::None
            }
        }
    }

    fn title(&self) -> &str {
        "System Configuration"
    }

    fn help_content(&self) -> Vec<String> {
        vec![
            "# System Configuration Screen".to_string(),
            "".to_string(),
            "Configure basic system settings for your NixOS installation.".to_string(),
            "".to_string(),
            "## Required Fields".to_string(),
            "".to_string(),
            "- Hostname: Name for your computer on the network".to_string(),
            "  - Must contain only letters, numbers, and hyphens".to_string(),
            "  - Default: nixos".to_string(),
            "".to_string(),
            "- Username: Your user account name".to_string(),
            "  - Must be lowercase letters, numbers, or underscores".to_string(),
            "  - Maximum 32 characters".to_string(),
            "  - This user will have sudo privileges".to_string(),
            "".to_string(),
            "- Password: Your account password".to_string(),
            "  - Minimum 6 characters".to_string(),
            "  - Must match confirmation".to_string(),
            "".to_string(),
            "## Additional Settings".to_string(),
            "".to_string(),
            "- Timezone: Select your timezone from ~250 options".to_string(),
            "  - Type to filter timezones (e.g., 'york', 'tokyo', 'london')".to_string(),
            "  - Use ↑↓ to navigate filtered results".to_string(),
            "  - Press Esc to clear filter".to_string(),
            "  - Default: America/New_York".to_string(),
            "".to_string(),
            "- Keyboard Layout: Select your console keymap".to_string(),
            "  - Type to filter (e.g., 'german', 'french', 'dvorak')".to_string(),
            "  - Default: us".to_string(),
            "".to_string(),
            "- Locale: Select your system locale".to_string(),
            "  - Type to filter (e.g., 'german', 'french', 'japanese')".to_string(),
            "  - Default: en_US.UTF-8".to_string(),
            "".to_string(),
            "## Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- ↑↓: Navigate between fields (or within selector lists)".to_string(),
            "- ←→: Navigate to previous/next field (works everywhere)".to_string(),
            "- Tab / Shift+Tab: Also navigate fields".to_string(),
            "- Enter: Move to next field (or submit from Continue button)".to_string(),
            "- ← / Backspace: Go back (when not in a text field)".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q / Esc: Quit the installer".to_string(),
            "".to_string(),
            "## Navigation Flow".to_string(),
            "".to_string(),
            "- Fill out all fields using Tab or Enter to move between them".to_string(),
            "- When you reach the Continue button, press Enter to submit".to_string(),
            "- The form will validate and proceed to the confirmation screen".to_string(),
        ]
    }

    fn can_proceed(&self) -> bool {
        // Check if basic requirements are met
        !self.hostname_input.value().is_empty()
            && !self.username_input.value().is_empty()
            && !self.password_input.value().is_empty()
            && !self.password_confirm_input.value().is_empty()
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

        // Down arrow moves to next field
        screen.handle_input(KeyCode::Down);
        assert_eq!(screen.focused_field, FocusedField::Username);

        screen.handle_input(KeyCode::Down);
        assert_eq!(screen.focused_field, FocusedField::Password);

        screen.handle_input(KeyCode::Down);
        assert_eq!(screen.focused_field, FocusedField::PasswordConfirm);

        // Up arrow moves to previous field
        screen.handle_input(KeyCode::Up);
        assert_eq!(screen.focused_field, FocusedField::Password);

        screen.handle_input(KeyCode::Up);
        assert_eq!(screen.focused_field, FocusedField::Username);

        screen.handle_input(KeyCode::Up);
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
}
