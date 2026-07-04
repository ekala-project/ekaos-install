//! Confirmation screen for reviewing configuration before installation
//!
//! This screen displays all collected configuration and requires explicit
//! user confirmation before proceeding to the installation phase.

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::config::InstallConfig;
use crate::ui::{
    components::{Component, Focusable, InputField, Interactive},
    theme::AppTheme,
    utils::{keycode_to_input_event, render_navigation_hints},
};

use super::{Screen, ScreenAction};

/// Confirmation screen state
pub struct ConfirmationScreen {
    /// Application theme
    theme: AppTheme,
    /// Configuration to display and confirm
    config: Option<InstallConfig>,
    /// Confirmation input field - must type "DELETE" to proceed
    confirm_input: InputField,
    /// Whether the confirmation gate is satisfied
    confirmed: bool,
}

impl ConfirmationScreen {
    /// Create a new confirmation screen
    pub fn new() -> Self {
        let confirm_input = InputField::new("Type DELETE to confirm");

        Self {
            theme: AppTheme::new(),
            config: None,
            confirm_input,
            confirmed: false,
        }
    }

    /// Set the configuration to display
    pub fn set_config(&mut self, config: InstallConfig) {
        self.config = Some(config);
        // Reset confirmation state when config changes
        self.confirm_input.set_value("");
        self.confirmed = false;
    }

    /// Get the confirmed configuration
    pub fn get_config(&self) -> Option<&InstallConfig> {
        self.config.as_ref()
    }

    /// Check if user has typed "DELETE"
    fn check_confirmation(&mut self) {
        self.confirmed = self.confirm_input.value() == "DELETE";
    }
}

impl Default for ConfirmationScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for ConfirmationScreen {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(6),  // System settings
                Constraint::Length(4),  // User account
                Constraint::Length(7),  // Disk & partitioning
                Constraint::Length(3),  // Bootloader
                Constraint::Length(3),  // Warning message
                Constraint::Length(3),  // Confirmation input
                Constraint::Min(0),     // Spacer
                Constraint::Length(3),  // Navigation
            ])
            .split(area);

        // Title
        let title = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Review Configuration",
                self.theme.title(),
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // If no config, show error
        let Some(config) = &self.config else {
            let error = Paragraph::new("No configuration available")
                .style(self.theme.error())
                .alignment(Alignment::Center);
            frame.render_widget(error, chunks[1]);
            return;
        };

        // System Settings section
        let system_block = Block::default()
            .borders(Borders::ALL)
            .title(" System Settings ")
            .border_style(self.theme.border_style());

        let system_lines = vec![
            Line::from(vec![
                Span::styled("  Hostname:  ", self.theme.text_muted()),
                Span::styled(&config.hostname, self.theme.text()),
            ]),
            Line::from(vec![
                Span::styled("  Timezone:  ", self.theme.text_muted()),
                Span::styled(&config.timezone, self.theme.text()),
            ]),
            Line::from(vec![
                Span::styled("  Locale:    ", self.theme.text_muted()),
                Span::styled(&config.locale, self.theme.text()),
            ]),
            Line::from(vec![
                Span::styled("  Keymap:    ", self.theme.text_muted()),
                Span::styled(&config.keymap, self.theme.text()),
            ]),
        ];

        let system_para = Paragraph::new(system_lines).block(system_block);
        frame.render_widget(system_para, chunks[1]);

        // User Account section
        let user_block = Block::default()
            .borders(Borders::ALL)
            .title(" User Account ")
            .border_style(self.theme.border_style());

        let user_lines = vec![
            Line::from(vec![
                Span::styled("  Username:  ", self.theme.text_muted()),
                Span::styled(&config.user.username, self.theme.text()),
            ]),
        ];

        let user_para = Paragraph::new(user_lines).block(user_block);
        frame.render_widget(user_para, chunks[2]);

        // Disk & Partitioning section
        let disk_block = Block::default()
            .borders(Borders::ALL)
            .title(" Disk & Partitioning ")
            .border_style(self.theme.border_style());

        let disk_size_gb = config.disk_size as f64 / 1_000_000_000.0;

        let encryption_status = if config.luks_encryption {
            "Yes (LUKS)"
        } else {
            "No"
        };

        let disk_lines = vec![
            Line::from(vec![
                Span::styled("  Disk:       ", self.theme.text_muted()),
                Span::styled(&config.disk_path, self.theme.text()),
                Span::styled(format!(" ({:.1} GB)", disk_size_gb), self.theme.text_muted()),
            ]),
            Line::from(vec![
                Span::styled("  Filesystem: ", self.theme.text_muted()),
                Span::styled(&config.root_filesystem, self.theme.text()),
            ]),
            Line::from(vec![
                Span::styled("  Swap:       ", self.theme.text_muted()),
                Span::styled(format!("{} GB", config.swap_size_gb), self.theme.text()),
            ]),
            Line::from(vec![
                Span::styled("  Encryption: ", self.theme.text_muted()),
                Span::styled(encryption_status, self.theme.text()),
            ]),
        ];

        let disk_para = Paragraph::new(disk_lines).block(disk_block);
        frame.render_widget(disk_para, chunks[3]);

        // Bootloader section
        let bootloader_block = Block::default()
            .borders(Borders::ALL)
            .title(" Bootloader ")
            .border_style(self.theme.border_style());

        let bootloader_line = vec![Line::from(vec![
            Span::styled("  Bootloader: ", self.theme.text_muted()),
            Span::styled(config.bootloader.as_str(), self.theme.text()),
        ])];

        let bootloader_para = Paragraph::new(bootloader_line).block(bootloader_block);
        frame.render_widget(bootloader_para, chunks[4]);

        // Warning and confirmation
        let warning = Paragraph::new(vec![
            Line::from(Span::styled(
                "⚠ ALL DATA ON THIS DISK WILL BE PERMANENTLY ERASED",
                self.theme.error(),
            )),
            Line::from(Span::styled(
                "Type DELETE below to confirm and begin installation",
                self.theme.warning_bold(),
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(warning, chunks[5]);

        // Confirmation input
        self.confirm_input.set_focused(true);
        self.confirm_input.render(frame, chunks[6]);

        // Navigation hints
        let hints = if self.confirmed {
            vec![
                ("Enter", "BEGIN INSTALL"),
                ("←", "Go Back"),
                ("?", "Help"),
            ]
        } else {
            vec![
                ("Type DELETE", "to confirm"),
                ("←", "Go Back"),
                ("?", "Help"),
            ]
        };

        render_navigation_hints(
            frame,
            &hints,
            &self.theme,
            chunks[8],
        );
    }

    fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
        // Try standard handlers first (quit, help)
        if let Some(action) = self.handle_standard_input(key) {
            return action;
        }

        // Try back handler
        if let Some(action) = self.handle_back_input(key) {
            return action;
        }

        // Handle screen-specific keys
        match key {
            KeyCode::Enter => {
                if self.confirmed {
                    ScreenAction::Next
                } else {
                    ScreenAction::None
                }
            }
            _ => {
                // Pass input to confirmation field
                if let Some(event) = keycode_to_input_event(key) {
                    Interactive::handle_input(&mut self.confirm_input, event);
                    self.check_confirmation();
                }
                ScreenAction::None
            }
        }
    }

    fn title(&self) -> &str {
        "Review Configuration"
    }

    fn help_content(&self) -> Vec<String> {
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
            "- ← / Backspace: Go back to configuration".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q / Esc: Quit the installer".to_string(),
        ]
    }

    fn can_proceed(&self) -> bool {
        self.config.is_some() && self.confirmed
    }

    fn can_go_back(&self) -> bool {
        true
    }
}
