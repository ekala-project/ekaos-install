//! System configuration screen
//!
//! Collects basic system configuration: hostname, username, password, timezone.

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tracing::debug;

use crate::config::{BootLoader, InstallConfig};
use crate::system::BootMode;
use crate::ui::{
    components::{Component, Focusable, InputEvent, InputField, Interactive},
    theme::AppTheme,
};

use super::{Screen, ScreenAction};

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

        Self {
            theme: AppTheme::new(),
            config: InstallConfig::default(),
            hostname_input,
            username_input,
            password_input,
            password_confirm_input,
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
            FocusedField::PasswordConfirm => FocusedField::Hostname,
        };
        self.set_focus();
    }

    /// Focus previous field
    fn focus_previous(&mut self) {
        self.clear_focus();
        self.focused_field = match self.focused_field {
            FocusedField::Hostname => FocusedField::PasswordConfirm,
            FocusedField::Username => FocusedField::Hostname,
            FocusedField::Password => FocusedField::Username,
            FocusedField::PasswordConfirm => FocusedField::Password,
        };
        self.set_focus();
    }

    /// Clear focus from all fields
    fn clear_focus(&mut self) {
        self.hostname_input.set_focused(false);
        self.username_input.set_focused(false);
        self.password_input.set_focused(false);
        self.password_confirm_input.set_focused(false);
    }

    /// Set focus on current field
    fn set_focus(&mut self) {
        match self.focused_field {
            FocusedField::Hostname => self.hostname_input.set_focused(true),
            FocusedField::Username => self.username_input.set_focused(true),
            FocusedField::Password => self.password_input.set_focused(true),
            FocusedField::PasswordConfirm => self.password_confirm_input.set_focused(true),
        }
    }

    /// Update configuration from input fields
    fn update_config(&mut self) {
        self.config.hostname = self.hostname_input.value().to_string();
        self.config.user.username = self.username_input.value().to_string();
        self.config.user.password = self.password_input.value().to_string();
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
                Constraint::Length(3), // Instructions
                Constraint::Length(3), // Hostname
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(3), // Confirm password
                Constraint::Length(5), // Summary info
                Constraint::Length(3), // Error/status
                Constraint::Min(0),    // Spacer
                Constraint::Length(3), // Navigation
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
        self.password_confirm_input.render(frame, chunks[4]);

        // Summary info
        let summary_block = Block::default()
            .borders(Borders::ALL)
            .title(" Additional Settings ")
            .border_style(self.theme.border_style());

        let summary_lines = vec![
            Line::from(vec![
                Span::styled("Bootloader: ", self.theme.text_muted()),
                Span::styled(self.config.bootloader.as_str(), self.theme.text()),
            ]),
            Line::from(vec![
                Span::styled("Timezone: ", self.theme.text_muted()),
                Span::styled(&self.config.timezone, self.theme.text()),
            ]),
            Line::from(vec![
                Span::styled("Locale: ", self.theme.text_muted()),
                Span::styled(&self.config.locale, self.theme.text()),
            ]),
        ];

        let summary_para = Paragraph::new(summary_lines).block(summary_block);
        frame.render_widget(summary_para, chunks[5]);

        // Error message or status
        if let Some(ref error) = self.error_message {
            let error_block = Block::default()
                .borders(Borders::ALL)
                .border_style(self.theme.error());

            let error_para = Paragraph::new(error.as_str())
                .block(error_block)
                .style(self.theme.error())
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(error_para, chunks[6]);
        }

        // Navigation hints
        let nav_text = vec![Line::from(vec![
            Span::styled("Tab", self.theme.shortcut()),
            Span::raw(" Next field | "),
            Span::styled("Shift+Tab", self.theme.shortcut()),
            Span::raw(" Previous | "),
            Span::styled("Enter", self.theme.shortcut()),
            Span::raw(" Continue | "),
            Span::styled("←", self.theme.shortcut()),
            Span::raw(" Back | "),
            Span::styled("q", self.theme.shortcut()),
            Span::raw(" Quit"),
        ])];

        let nav_para = Paragraph::new(nav_text).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(nav_para, chunks[8]);
    }

    fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
        match key {
            KeyCode::Tab => {
                self.focus_next();
                ScreenAction::None
            }
            KeyCode::BackTab => {
                self.focus_previous();
                ScreenAction::None
            }
            KeyCode::Enter => {
                if self.validate() {
                    debug!("Configuration validated successfully");
                    ScreenAction::Next
                } else {
                    ScreenAction::None
                }
            }
            KeyCode::Left | KeyCode::Backspace
                if !self.hostname_input.is_focused()
                    && !self.username_input.is_focused()
                    && !self.password_input.is_focused()
                    && !self.password_confirm_input.is_focused() =>
            {
                ScreenAction::Back
            }
            KeyCode::Char('q') | KeyCode::Esc => ScreenAction::Exit,
            KeyCode::Char('?') => ScreenAction::ToggleHelp,
            _ => {
                // Convert KeyCode to InputEvent and pass to focused field
                let event = match key {
                    KeyCode::Char(c) => InputEvent::Char(c),
                    KeyCode::Backspace => InputEvent::Backspace,
                    KeyCode::Delete => InputEvent::Delete,
                    KeyCode::Left => InputEvent::Left,
                    KeyCode::Right => InputEvent::Right,
                    KeyCode::Home => InputEvent::Home,
                    KeyCode::End => InputEvent::End,
                    _ => return ScreenAction::None,
                };

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
            "- Bootloader: Automatically selected based on boot mode".to_string(),
            "  - UEFI systems use systemd-boot".to_string(),
            "  - BIOS systems use GRUB".to_string(),
            "".to_string(),
            "- Timezone: America/New_York (default)".to_string(),
            "- Locale: en_US.UTF-8 (default)".to_string(),
            "".to_string(),
            "## Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- Tab: Move to next field".to_string(),
            "- Shift+Tab: Move to previous field".to_string(),
            "- Enter: Validate and continue".to_string(),
            "- ← / Backspace: Go back (when not in a text field)".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q / Esc: Quit the installer".to_string(),
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
}
