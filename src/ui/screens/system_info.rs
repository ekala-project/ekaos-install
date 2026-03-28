//! System information display screen

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tracing::{debug, warn};

use crate::system::{detect_boot_mode, detect_system, BootMode, SystemInfo};
use crate::ui::{theme::AppTheme, utils::render_navigation_hints};

use super::{Screen, ScreenAction};

/// System information screen
pub struct SystemInfoScreen {
    /// Application theme
    theme: AppTheme,
    /// Detected boot mode
    pub boot_mode: BootMode,
    /// System info data
    system_info: SystemInfo,
}

impl SystemInfoScreen {
    /// Create a new system info screen
    pub fn new() -> Self {
        Self {
            theme: AppTheme::new(),
            boot_mode: BootMode::Unknown,
            system_info: SystemInfo::default(),
        }
    }

    /// Set boot mode (for testing)
    pub fn with_boot_mode(mut self, boot_mode: BootMode) -> Self {
        self.boot_mode = boot_mode;
        self
    }
}

impl Default for SystemInfoScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for SystemInfoScreen {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Boot mode
                Constraint::Length(8), // System info
                Constraint::Min(0),    // Spacer
                Constraint::Length(3), // Instructions
            ])
            .split(area);

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
        frame.render_widget(boot_mode_para, chunks[0]);

        // System info section
        let system_block = Block::default()
            .borders(Borders::ALL)
            .title(" System Information ")
            .border_style(self.theme.border_style());

        let info_items = vec![
            ListItem::new(Line::from(vec![
                Span::styled("CPU: ", self.theme.text_muted()),
                Span::styled(&self.system_info.cpu_model, self.theme.text()),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Cores: ", self.theme.text_muted()),
                Span::styled(self.system_info.cpu_cores.to_string(), self.theme.text()),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("RAM: ", self.theme.text_muted()),
                Span::styled(
                    format!("{:.1} GB", self.system_info.ram_gb),
                    self.theme.text(),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Architecture: ", self.theme.text_muted()),
                Span::styled(&self.system_info.architecture, self.theme.text()),
            ])),
        ];

        let system_list = List::new(info_items).block(system_block);
        frame.render_widget(system_list, chunks[1]);

        // Instructions
        render_navigation_hints(
            frame,
            &[
                ("←", "Back"),
                ("Enter", "Next"),
                ("?", "Help"),
                ("q", "Quit"),
            ],
            &self.theme,
            chunks[3],
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

    fn on_enter(&mut self) {
        // Perform system detection
        debug!("Detecting boot mode and system information");

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

        // Detect system information
        match detect_system() {
            Ok(info) => {
                debug!(
                    "Detected system: {} ({} cores, {:.1} GB RAM, {})",
                    info.cpu_model, info.cpu_cores, info.ram_gb, info.architecture
                );
                self.system_info = info;
            }
            Err(e) => {
                warn!("Failed to detect system info: {}", e);
                self.system_info = SystemInfo::default();
            }
        }
    }

    fn title(&self) -> &str {
        "System Information"
    }

    fn help_content(&self) -> Vec<String> {
        vec![
            "# System Information Screen".to_string(),
            "".to_string(),
            "This screen displays detected system information including:".to_string(),
            "".to_string(),
            "- Boot Mode: UEFI or Legacy BIOS".to_string(),
            "- CPU: Processor model and core count".to_string(),
            "- RAM: Total system memory".to_string(),
            "- Architecture: System architecture (x86_64, aarch64, etc.)".to_string(),
            "".to_string(),
            "# Boot Modes".to_string(),
            "".to_string(),
            "- UEFI: Modern boot mode, recommended for new systems".to_string(),
            "  Supports GPT partition tables and Secure Boot".to_string(),
            "".to_string(),
            "- Legacy BIOS: Older boot mode for legacy hardware".to_string(),
            "  Uses MBR partition tables (2TB disk limit)".to_string(),
            "".to_string(),
            "# Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- Enter: Continue to disk selection".to_string(),
            "- ← / Backspace: Go back to welcome screen".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q / Esc: Quit the installer".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_mode() {
        assert_eq!(BootMode::Uefi.as_str(), "UEFI");
        assert_eq!(BootMode::Bios.as_str(), "Legacy BIOS");
        assert!(BootMode::Uefi.description().contains("GPT"));
    }

    #[test]
    fn test_system_info_screen() {
        let screen = SystemInfoScreen::new();
        assert_eq!(screen.title(), "System Information");
        assert!(screen.can_go_back());
        assert!(screen.can_proceed());
    }

    #[test]
    fn test_screen_navigation() {
        let mut screen = SystemInfoScreen::new();

        assert_eq!(screen.handle_input(KeyCode::Enter), ScreenAction::Next);
        assert_eq!(screen.handle_input(KeyCode::Left), ScreenAction::Back);
        assert_eq!(
            screen.handle_input(KeyCode::Char('?')),
            ScreenAction::ToggleHelp
        );
    }

    #[test]
    fn test_with_boot_mode() {
        let screen = SystemInfoScreen::new().with_boot_mode(BootMode::Uefi);
        assert_eq!(screen.boot_mode, BootMode::Uefi);
    }

    #[test]
    fn test_on_enter_detection() {
        // Test that on_enter runs detection without panicking
        let mut screen = SystemInfoScreen::new();
        screen.on_enter();
        // In mock mode or real mode, should have detected something
        assert!(screen.system_info.cpu_cores > 0);
    }

    #[test]
    fn test_help_content() {
        let screen = SystemInfoScreen::new();
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("System Information"));
    }
}
