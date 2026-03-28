//! System information display screen

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::ui::theme::AppTheme;

use super::{Screen, ScreenAction};

/// System information screen
pub struct SystemInfoScreen {
    /// Application theme
    theme: AppTheme,
    /// Detected boot mode
    boot_mode: BootMode,
    /// System info data
    system_info: SystemInfo,
}

/// Boot mode (UEFI or BIOS)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootMode {
    /// UEFI boot mode
    Uefi,
    /// Legacy BIOS boot mode
    Bios,
    /// Unknown/not detected
    Unknown,
}

impl BootMode {
    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            BootMode::Uefi => "UEFI",
            BootMode::Bios => "Legacy BIOS",
            BootMode::Unknown => "Unknown",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            BootMode::Uefi => "Modern boot mode with GPT partition table support",
            BootMode::Bios => "Legacy boot mode with MBR partition table",
            BootMode::Unknown => "Unable to determine boot mode",
        }
    }
}

/// System information
#[derive(Debug, Clone)]
struct SystemInfo {
    /// CPU model
    cpu: String,
    /// Number of CPU cores
    cores: usize,
    /// Total RAM in GB
    ram_gb: f64,
    /// Architecture
    arch: String,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            cpu: "Unknown CPU".to_string(),
            cores: 1,
            ram_gb: 0.0,
            arch: "x86_64".to_string(),
        }
    }
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

    /// Set system info (for testing)
    pub fn with_system_info(
        mut self,
        cpu: String,
        cores: usize,
        ram_gb: f64,
        arch: String,
    ) -> Self {
        self.system_info = SystemInfo {
            cpu,
            cores,
            ram_gb,
            arch,
        };
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
                Constraint::Length(5),  // Boot mode
                Constraint::Length(8),  // System info
                Constraint::Min(0),     // Spacer
                Constraint::Length(3),  // Instructions
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
                Span::styled(&self.system_info.cpu, self.theme.text()),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Cores: ", self.theme.text_muted()),
                Span::styled(self.system_info.cores.to_string(), self.theme.text()),
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
                Span::styled(&self.system_info.arch, self.theme.text()),
            ])),
        ];

        let system_list = List::new(info_items).block(system_block);
        frame.render_widget(system_list, chunks[1]);

        // Instructions
        let instructions = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("←", self.theme.shortcut()),
                Span::raw(" Back | "),
                Span::styled("Enter", self.theme.shortcut()),
                Span::raw(" Next | "),
                Span::styled("?", self.theme.shortcut()),
                Span::raw(" Help | "),
                Span::styled("q", self.theme.shortcut()),
                Span::raw(" Quit"),
            ]),
        ];

        let instructions_para = Paragraph::new(instructions)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(instructions_para, chunks[3]);
    }

    fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
        match key {
            KeyCode::Enter => ScreenAction::Next,
            KeyCode::Left | KeyCode::Backspace => ScreenAction::Back,
            KeyCode::Char('q') | KeyCode::Esc => ScreenAction::Exit,
            KeyCode::Char('?') => ScreenAction::ToggleHelp,
            _ => ScreenAction::None,
        }
    }

    fn on_enter(&mut self) {
        // TODO: Perform system detection here
        // For now, use placeholder data
        self.boot_mode = BootMode::Uefi; // Will be detected in Phase 2
        self.system_info = SystemInfo {
            cpu: "Simulated CPU".to_string(),
            cores: 8,
            ram_gb: 16.0,
            arch: "x86_64".to_string(),
        };
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

        assert_eq!(
            screen.handle_input(KeyCode::Enter),
            ScreenAction::Next
        );
        assert_eq!(
            screen.handle_input(KeyCode::Left),
            ScreenAction::Back
        );
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
    fn test_help_content() {
        let screen = SystemInfoScreen::new();
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("System Information"));
    }
}
