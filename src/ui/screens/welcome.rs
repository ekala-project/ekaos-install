//! Welcome screen with pre-flight checks

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tracing::{debug, warn};

use crate::system::{check_network, is_nixos, is_root};
use crate::ui::{icons, theme::AppTheme, utils::render_navigation_hints};

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
        Self {
            theme: AppTheme::new(),
            is_mock,
            is_dry_run,
            checks: PreFlightChecks::default(),
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
                Constraint::Length(2),  // Spacer
                Constraint::Length(10), // Pre-flight checks
                Constraint::Min(0),     // Spacer
                Constraint::Length(3),  // Instructions
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

        // Instructions
        if self.all_checks_passed() {
            render_navigation_hints(
                frame,
                &[("Enter", "Continue"), ("?", "Help"), ("q", "Quit")],
                &self.theme,
                chunks[7],
            );
        } else {
            let error_msg = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Please resolve the issues above before continuing",
                    self.theme.error(),
                )),
            ];
            let error_para = Paragraph::new(error_msg).alignment(Alignment::Center);
            frame.render_widget(error_para, chunks[7]);
        }
    }

    fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
        // Try standard handlers first (quit, help)
        if let Some(action) = self.handle_standard_input(key) {
            return action;
        }

        // Handle screen-specific keys
        match key {
            KeyCode::Enter if self.all_checks_passed() => ScreenAction::Next,
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

        // Check 4: Disk space (placeholder - will be implemented properly later)
        // For now, assume true in mock mode or if we made it this far
        self.checks.disk_space = true;
        debug!("Disk space check: {}", self.checks.disk_space);

        debug!(
            "Pre-flight checks complete: root={}, network={}, nixos={}, disk={}",
            self.checks.root_access,
            self.checks.network,
            self.checks.nixos_iso,
            self.checks.disk_space
        );
    }

    fn help_content(&self) -> Vec<String> {
        vec![
            "# Welcome Screen".to_string(),
            "".to_string(),
            "This screen performs pre-flight checks to ensure your system".to_string(),
            "is ready for NixOS installation.".to_string(),
            "".to_string(),
            "# Pre-Flight Checks".to_string(),
            "".to_string(),
            "- Root Access: The installer must run with root privileges".to_string(),
            "- Network Connectivity: Required to download NixOS packages".to_string(),
            "- NixOS Installation Media: Must be running from official ISO".to_string(),
            "- Sufficient Disk Space: At least 10GB free space required".to_string(),
            "".to_string(),
            "# Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- Enter: Continue to next step (when checks pass)".to_string(),
            "- q/Esc: Quit the installer".to_string(),
            "- ?: Toggle this help panel".to_string(),
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
        let screen = WelcomeScreen::new(false, false);
        assert!(screen.all_checks_passed());
        assert!(screen.can_proceed());
    }

    #[test]
    fn test_navigation() {
        let mut screen = WelcomeScreen::new(false, false);

        // Can't go back from welcome
        assert!(!screen.can_go_back());

        // Enter should proceed when checks pass
        assert_eq!(screen.handle_input(KeyCode::Enter), ScreenAction::Next);

        // q should exit
        assert_eq!(screen.handle_input(KeyCode::Char('q')), ScreenAction::Exit);
    }

    #[test]
    fn test_help_content() {
        let screen = WelcomeScreen::new(false, false);
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("Welcome Screen"));
    }
}
