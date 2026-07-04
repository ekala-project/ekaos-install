//! Welcome screen with pre-flight checks

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tracing::{debug, warn};

use crate::nixos::disk::detect_disks;
use crate::system::{check_network, detect_boot_mode, is_nixos, is_root, BootMode};
use crate::ui::{
    components::{Button, Component, Focusable},
    icons,
    theme::AppTheme,
    utils::render_navigation_hints,
};

use super::{Screen, ScreenAction};

/// Welcome screen state
pub struct WelcomeScreen {
    /// Application theme
    theme: AppTheme,
    /// Whether we're in mock mode
    is_mock: bool,
    /// Whether dry-run is enabled
    is_dry_run: bool,
    /// Pre-flight check results
    checks: PreFlightChecks,
    /// Detected boot mode (public - needed by later screens)
    pub boot_mode: BootMode,
    /// Continue button
    continue_button: Button,
}

/// Pre-flight check results
#[derive(Debug, Clone)]
struct PreFlightChecks {
    /// Running as root (or mock mode)
    root_access: bool,
    /// Network connectivity
    network: bool,
    /// Running from NixOS ISO
    nixos_iso: bool,
    /// Sufficient disk space available
    disk_space: bool,
}

impl Default for PreFlightChecks {
    fn default() -> Self {
        Self {
            root_access: true, // Will be checked properly later
            network: true,
            nixos_iso: true,
            disk_space: true,
        }
    }
}

impl WelcomeScreen {
    /// Create a new welcome screen
    pub fn new(is_mock: bool, is_dry_run: bool) -> Self {
        let continue_button = Button::new("Continue");

        Self {
            theme: AppTheme::new(),
            is_mock,
            is_dry_run,
            checks: PreFlightChecks::default(),
            boot_mode: BootMode::Unknown,
            continue_button,
        }
    }

    /// Check if all pre-flight checks passed
    fn all_checks_passed(&self) -> bool {
        self.checks.root_access
            && self.checks.network
            && self.checks.nixos_iso
            && self.checks.disk_space
    }
}

impl Screen for WelcomeScreen {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Spacer
                Constraint::Length(3),  // Title
                Constraint::Length(1),  // Spacer
                Constraint::Length(2),  // Subtitle
                Constraint::Length(2),  // Spacer/Mode indicators
                Constraint::Length(10), // Pre-flight checks
                Constraint::Length(5),  // Boot mode
                Constraint::Length(3),  // Continue button
                Constraint::Min(0),     // Spacer
                Constraint::Length(3),  // Navigation hints
            ])
            .split(area);

        // Title
        let title = Paragraph::new(vec![Line::from(Span::styled(
            "Welcome to Ekaos Install",
            self.theme.title(),
        ))])
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[1]);

        // Subtitle
        let subtitle = Paragraph::new("A guided installer for NixOS")
            .alignment(Alignment::Center)
            .style(self.theme.text());
        frame.render_widget(subtitle, chunks[3]);

        // Mode indicators
        if self.is_mock || self.is_dry_run {
            let mut mode_text = Vec::new();

            if self.is_mock {
                mode_text.push(Line::from(Span::styled(
                    "Running in MOCK MODE",
                    self.theme.warning_bold(),
                )));
                mode_text.push(Line::from(Span::styled(
                    "(No actual system changes will be made)",
                    self.theme.text_muted(),
                )));
            }

            if self.is_dry_run {
                mode_text.push(Line::from(Span::styled(
                    "DRY-RUN MODE ENABLED",
                    self.theme.info_bold(),
                )));
            }

            let mode_para = Paragraph::new(mode_text).alignment(Alignment::Center);
            frame.render_widget(mode_para, chunks[4]);
        }

        // Pre-flight checks
        let checks_block = Block::default()
            .borders(Borders::ALL)
            .title(" Pre-Flight Checks ")
            .border_style(self.theme.border_style());

        let check_lines = vec![
            self.check_line("Root Access", self.checks.root_access),
            self.check_line("Network Connectivity", self.checks.network),
            self.check_line("NixOS Installation Media", self.checks.nixos_iso),
            self.check_line("Sufficient Disk Space", self.checks.disk_space),
        ];

        let checks_para = Paragraph::new(check_lines)
            .block(checks_block)
            .style(self.theme.text());
        frame.render_widget(checks_para, chunks[5]);

        // Boot mode section
        let boot_mode_block = Block::default()
            .borders(Borders::ALL)
            .title(" Boot Mode ")
            .border_style(self.theme.border_style());

        let boot_mode_lines = vec![
            Line::from(vec![
                Span::styled("Mode: ", self.theme.text_muted()),
                Span::styled(self.boot_mode.as_str(), self.theme.info_bold()),
            ]),
            Line::from(Span::styled(
                self.boot_mode.description(),
                self.theme.text_muted(),
            )),
        ];

        let boot_mode_para = Paragraph::new(boot_mode_lines)
            .block(boot_mode_block)
            .style(self.theme.text());
        frame.render_widget(boot_mode_para, chunks[6]);

        // Continue button
        self.continue_button.render(frame, chunks[7]);

        // Navigation hints
        render_navigation_hints(
            frame,
            &[("Enter", "Submit from button"), ("?", "Help"), ("q", "Quit")],
            &self.theme,
            chunks[9],
        );

        // Error message if checks failed (overlay on spacer area)
        if !self.all_checks_passed() {
            let error_msg = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Please resolve the issues above before continuing",
                    self.theme.error(),
                )),
            ];
            let error_para = Paragraph::new(error_msg).alignment(Alignment::Center);
            frame.render_widget(error_para, chunks[8]);
        }
    }

    fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
        // Try standard handlers first (quit, help)
        if let Some(action) = self.handle_standard_input(key) {
            return action;
        }

        // Handle screen-specific keys
        match key {
            KeyCode::Enter => {
                // Only advance when button is focused and checks pass
                if self.continue_button.is_focused() && self.all_checks_passed() {
                    ScreenAction::Next
                } else {
                    ScreenAction::None
                }
            }
            _ => ScreenAction::None,
        }
    }

    fn title(&self) -> &str {
        "Welcome"
    }

    fn can_proceed(&self) -> bool {
        self.all_checks_passed()
    }

    fn can_go_back(&self) -> bool {
        false // Can't go back from welcome screen
    }

    fn on_enter(&mut self) {
        // Perform pre-flight checks
        debug!("Running pre-flight checks");

        // Check 1: Root access (or mock mode)
        self.checks.root_access = self.is_mock || is_root();
        debug!("Root access check: {}", self.checks.root_access);

        // Check 2: Network connectivity
        match check_network() {
            Ok(status) => {
                self.checks.network = status.is_connected();
                debug!(
                    "Network check: {} (status: {:?})",
                    self.checks.network, status
                );
            }
            Err(e) => {
                warn!("Network check failed: {}", e);
                self.checks.network = false;
            }
        }

        // Check 3: Running on NixOS
        match is_nixos() {
            Ok(result) => {
                self.checks.nixos_iso = result;
                debug!("NixOS check: {}", self.checks.nixos_iso);
            }
            Err(e) => {
                warn!("NixOS check failed: {}", e);
                self.checks.nixos_iso = false;
            }
        }

        // Check 4: Sufficient disk space (at least one installable disk exists)
        match detect_disks() {
            Ok(disks) => {
                self.checks.disk_space = disks.iter().any(|d| d.is_installable());
                debug!(
                    "Disk space check: {} ({} disks found, {} installable)",
                    self.checks.disk_space,
                    disks.len(),
                    disks.iter().filter(|d| d.is_installable()).count()
                );
            }
            Err(e) => {
                warn!("Disk detection failed: {}", e);
                self.checks.disk_space = false;
            }
        }

        debug!(
            "Pre-flight checks complete: root={}, network={}, nixos={}, disk={}",
            self.checks.root_access,
            self.checks.network,
            self.checks.nixos_iso,
            self.checks.disk_space
        );

        // Detect boot mode
        match detect_boot_mode() {
            Ok(mode) => {
                debug!("Detected boot mode: {:?}", mode);
                self.boot_mode = mode;
            }
            Err(e) => {
                warn!("Failed to detect boot mode: {}", e);
                self.boot_mode = BootMode::Unknown;
            }
        }

        // Set button as focused
        self.continue_button.set_focused(true);
    }

    fn help_content(&self) -> Vec<String> {
        vec![
            "# Welcome Screen".to_string(),
            "".to_string(),
            "This screen performs pre-flight checks to ensure your system is ready".to_string(),
            "for NixOS installation.".to_string(),
            "".to_string(),
            "# Pre-Flight Checks".to_string(),
            "".to_string(),
            "- Root Access: The installer must run with root privileges".to_string(),
            "- Network Connectivity: Required to download NixOS packages".to_string(),
            "- NixOS Installation Media: Must be running from official ISO".to_string(),
            "- Sufficient Disk Space: At least 10GB free space required".to_string(),
            "".to_string(),
            "# Boot Modes".to_string(),
            "".to_string(),
            "The installer automatically detects your system's boot mode:".to_string(),
            "".to_string(),
            "- UEFI: Modern boot mode, recommended for new systems".to_string(),
            "  Supports GPT partition tables and Secure Boot".to_string(),
            "".to_string(),
            "- Legacy BIOS: Older boot mode for legacy hardware".to_string(),
            "  Uses MBR partition tables (2TB disk limit)".to_string(),
            "".to_string(),
            "# Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- Enter: Press Continue button (when checks pass)".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q/Esc: Quit the installer".to_string(),
            "".to_string(),
            "# How to Continue".to_string(),
            "".to_string(),
            "Once all pre-flight checks pass, the Continue button will be enabled".to_string(),
            "Press Enter to proceed to disk selection.".to_string(),
        ]
    }
}

impl WelcomeScreen {
    /// Create a check status line
    fn check_line(&self, label: &str, passed: bool) -> Line<'static> {
        let icon = if passed { icons::SUCCESS } else { icons::ERROR };

        let style = if passed {
            self.theme.success()
        } else {
            self.theme.error()
        };

        Line::from(vec![
            Span::raw("  "),
            Span::styled(icon.to_string(), style),
            Span::raw(" "),
            Span::styled(label.to_string(), self.theme.text()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_screen_creation() {
        let screen = WelcomeScreen::new(true, false);
        assert_eq!(screen.title(), "Welcome");
        assert!(screen.is_mock);
        assert!(!screen.is_dry_run);
    }

    #[test]
    fn test_preflight_checks() {
        std::env::set_var("EKAOS_MOCK", "1");
        let mut screen = WelcomeScreen::new(true, false);
        screen.on_enter();
        assert!(screen.all_checks_passed());
        assert!(screen.can_proceed());
        std::env::remove_var("EKAOS_MOCK");
    }

    #[test]
    fn test_navigation() {
        std::env::set_var("EKAOS_MOCK", "1");
        let mut screen = WelcomeScreen::new(true, false);
        screen.on_enter(); // Sets up button focus and runs checks in mock mode

        // Can't go back from welcome
        assert!(!screen.can_go_back());

        // Enter should proceed when checks pass and button is focused
        assert_eq!(screen.handle_input(KeyCode::Enter), ScreenAction::Next);

        // q should exit
        assert_eq!(screen.handle_input(KeyCode::Char('q')), ScreenAction::Exit);
        std::env::remove_var("EKAOS_MOCK");
    }

    #[test]
    fn test_help_content() {
        let screen = WelcomeScreen::new(false, false);
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("Welcome Screen"));
    }
}
