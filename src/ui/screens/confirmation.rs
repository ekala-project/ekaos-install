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
use crate::ui::{theme::AppTheme, utils::render_navigation_hints};

use super::{Screen, ScreenAction};

/// Confirmation screen state
pub struct ConfirmationScreen {
    /// Application theme
    theme: AppTheme,
    /// Configuration to display and confirm
    config: Option<InstallConfig>,
}

impl ConfirmationScreen {
    /// Create a new confirmation screen
    pub fn new() -> Self {
        Self {
            theme: AppTheme::new(),
            config: None,
        }
    }

    /// Set the configuration to display
    pub fn set_config(&mut self, config: InstallConfig) {
        self.config = Some(config);
    }

    /// Get the confirmed configuration
    pub fn get_config(&self) -> Option<&InstallConfig> {
        self.config.as_ref()
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
                Constraint::Length(6),  // Disk & partitioning
                Constraint::Length(3),  // Bootloader
                Constraint::Length(4),  // Warning message
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

        let admin_status = if config.user.is_admin {
            "Yes"
        } else {
            "No"
        };

        let user_lines = vec![
            Line::from(vec![
                Span::styled("  Username:  ", self.theme.text_muted()),
                Span::styled(&config.user.username, self.theme.text()),
            ]),
            Line::from(vec![
                Span::styled("  Admin:     ", self.theme.text_muted()),
                Span::styled(admin_status, self.theme.text()),
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

        // Warning message
        let warning = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "⚠ Ready to begin installation?",
                self.theme.warning_bold(),
            )),
            Line::from(Span::styled(
                "This will format the disk and install NixOS.",
                self.theme.text_muted(),
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(warning, chunks[5]);

        // Navigation hints
        render_navigation_hints(
            frame,
            &[
                ("Enter", "Continue"),
                ("←", "Go Back"),
                ("?", "Help"),
                ("q", "Quit"),
            ],
            &self.theme,
            chunks[7],
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
            KeyCode::Enter => ScreenAction::Next,
            _ => ScreenAction::None,
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
            "# What Happens Next".to_string(),
            "".to_string(),
            "If you continue, the installer will:".to_string(),
            "".to_string(),
            "1. Format the selected disk".to_string(),
            "2. Create partitions (EFI/Boot, Swap, Root)".to_string(),
            "3. Install NixOS base system".to_string(),
            "4. Configure the bootloader".to_string(),
            "5. Set up your user account".to_string(),
            "6. Apply system settings".to_string(),
            "".to_string(),
            "⚠ WARNING: This process will erase all data on the selected disk!".to_string(),
            "".to_string(),
            "# Configuration Review".to_string(),
            "".to_string(),
            "System Settings:".to_string(),
            "- Hostname: Name for your computer on the network".to_string(),
            "- Timezone: Your local timezone".to_string(),
            "- Locale: Language and regional settings".to_string(),
            "- Keymap: Keyboard layout".to_string(),
            "".to_string(),
            "User Account:".to_string(),
            "- Username: Your login name".to_string(),
            "- Admin: Whether you have sudo privileges".to_string(),
            "".to_string(),
            "Disk & Partitioning:".to_string(),
            "- Disk: Target installation disk".to_string(),
            "- Filesystem: Root partition filesystem type".to_string(),
            "- Swap: Swap partition size".to_string(),
            "".to_string(),
            "Bootloader:".to_string(),
            "- Automatically selected based on boot mode".to_string(),
            "- UEFI systems use systemd-boot".to_string(),
            "- BIOS systems use GRUB".to_string(),
            "".to_string(),
            "# Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- Enter: Confirm and begin installation".to_string(),
            "- ← / Backspace: Go back to configuration".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q / Esc: Quit the installer".to_string(),
        ]
    }

    fn can_proceed(&self) -> bool {
        self.config.is_some()
    }

    fn can_go_back(&self) -> bool {
        true
    }
}
