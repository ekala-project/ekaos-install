//! Installation progress screen
//!
//! Displays real-time installation progress with log output.

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::sync::mpsc::Receiver;

use crate::config::InstallConfig;
use crate::nixos::{run_installation_async, InstallMessage, InstallProgress, InstallStage};
use crate::ui::{
    components::{Component, ProgressBar},
    theme::AppTheme,
};

use super::{Screen, ScreenAction};

/// Installation screen state
pub struct InstallationScreen {
    /// Application theme
    theme: AppTheme,
    /// Current installation progress
    progress: Option<InstallProgress>,
    /// Log lines
    log_lines: Vec<String>,
    /// Whether installation is running
    is_running: bool,
    /// Whether installation succeeded
    is_complete: bool,
    /// Whether installation failed
    has_error: bool,
    /// Error message if failed
    error_message: Option<String>,
    /// Receiver for installation messages
    receiver: Option<Receiver<InstallMessage>>,
    /// Scroll position for log
    scroll_position: usize,
}

impl InstallationScreen {
    /// Create a new installation screen
    pub fn new() -> Self {
        Self {
            theme: AppTheme::new(),
            progress: None,
            log_lines: Vec::new(),
            is_running: false,
            is_complete: false,
            has_error: false,
            error_message: None,
            receiver: None,
            scroll_position: 0,
        }
    }

    /// Start the installation process
    pub fn start_installation(&mut self, config: InstallConfig, root_path: String, is_mock: bool) {
        let rx = run_installation_async(config, root_path, is_mock);
        self.receiver = Some(rx);
        self.is_running = true;
        self.is_complete = false;
        self.has_error = false;
        self.error_message = None;
        self.log_lines.clear();
        self.log_lines.push("Starting installation...".to_string());
    }

    /// Check for installation updates
    pub fn update(&mut self) {
        if let Some(rx) = &self.receiver {
            // Process all available messages without blocking
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    InstallMessage::Progress(progress) => {
                        self.progress = Some(progress);
                    }
                    InstallMessage::Log(line) => {
                        self.log_lines.push(line);
                        // Auto-scroll to bottom
                        self.scroll_position = self.log_lines.len().saturating_sub(1);
                    }
                    InstallMessage::Success => {
                        self.is_running = false;
                        self.is_complete = true;
                        self.log_lines.push("✓ Installation completed successfully!".to_string());
                    }
                    InstallMessage::Error(err) => {
                        self.is_running = false;
                        self.has_error = true;
                        self.error_message = Some(err.clone());
                        self.log_lines.push(format!("✗ Installation failed: {}", err));
                    }
                }
            }
        }
    }

    /// Scroll log up
    fn scroll_up(&mut self) {
        if self.scroll_position > 0 {
            self.scroll_position -= 1;
        }
    }

    /// Scroll log down
    fn scroll_down(&mut self) {
        if self.scroll_position < self.log_lines.len().saturating_sub(1) {
            self.scroll_position += 1;
        }
    }

    /// Get stage description
    fn stage_description(&self) -> String {
        if let Some(ref progress) = self.progress {
            match progress.stage {
                InstallStage::GeneratingConfig => "Generating configuration files...".to_string(),
                InstallStage::GeneratingHardwareConfig => {
                    "Detecting hardware configuration...".to_string()
                }
                InstallStage::Installing => "Installing NixOS (this may take a while)...".to_string(),
                InstallStage::Verifying => "Verifying installation...".to_string(),
                InstallStage::Complete => "Installation complete!".to_string(),
                InstallStage::Failed(ref err) => format!("Installation failed: {}", err),
            }
        } else {
            "Initializing...".to_string()
        }
    }
}

impl Default for InstallationScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for InstallationScreen {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        // Update installation state
        self.update();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(4),  // Progress bar
                Constraint::Length(3),  // Current operation
                Constraint::Min(10),    // Log output
                Constraint::Length(3),  // Status/Navigation
            ])
            .split(area);

        // Title
        let title = if self.has_error {
            "Installation Failed"
        } else if self.is_complete {
            "Installation Complete"
        } else {
            "Installing NixOS"
        };

        let title_para = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                title,
                Style::default()
                    .fg(if self.has_error {
                        self.theme.error
                    } else if self.is_complete {
                        self.theme.success
                    } else {
                        self.theme.primary
                    })
                    .add_modifier(Modifier::BOLD),
            )),
        ])
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(title_para, chunks[0]);

        // Progress bar
        let percent = self.progress.as_ref().map(|p| p.percent).unwrap_or(0);
        let mut progress_bar = ProgressBar::new(percent as u16)
            .with_label(format!("Progress: {}%", percent));

        if self.has_error {
            progress_bar = progress_bar.with_color(self.theme.error);
        } else if self.is_complete {
            progress_bar = progress_bar.with_color(self.theme.success);
        }

        Component::render(&mut progress_bar, frame, chunks[1]);

        // Current operation
        let operation_text = self.stage_description();
        let operation_para = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                operation_text,
                Style::default().add_modifier(Modifier::ITALIC),
            )),
        ])
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(operation_para, chunks[2]);

        // Log output
        let log_block = Block::default()
            .borders(Borders::ALL)
            .title(" Installation Log ")
            .border_style(self.theme.border_style());

        let visible_lines = chunks[3].height.saturating_sub(2) as usize;
        let start_idx = self.scroll_position.saturating_sub(visible_lines / 2);
        let end_idx = (start_idx + visible_lines).min(self.log_lines.len());

        let log_items: Vec<ListItem> = self.log_lines[start_idx..end_idx]
            .iter()
            .map(|line| {
                let style = if line.starts_with('✓') {
                    Style::default().fg(self.theme.success)
                } else if line.starts_with('✗') {
                    Style::default().fg(self.theme.error)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(line.clone(), style)))
            })
            .collect();

        let log_list = List::new(log_items).block(log_block);
        frame.render_widget(log_list, chunks[3]);

        // Status and navigation
        let nav_text = if self.is_complete {
            vec![Line::from(vec![
                Span::styled("Enter", self.theme.shortcut()),
                Span::raw(" Continue to completion screen"),
            ])]
        } else if self.has_error {
            vec![Line::from(vec![
                Span::styled("q", self.theme.shortcut()),
                Span::raw(" Exit | "),
                Span::styled("↑↓", self.theme.shortcut()),
                Span::raw(" Scroll log"),
            ])]
        } else {
            vec![Line::from(vec![
                Span::raw("Installing... Please wait. "),
                Span::styled("↑↓", self.theme.shortcut()),
                Span::raw(" Scroll log"),
            ])]
        };

        let nav_para = Paragraph::new(nav_text).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(nav_para, chunks[4]);
    }

    fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
        match key {
            KeyCode::Enter if self.is_complete => ScreenAction::Next,
            KeyCode::Up => {
                self.scroll_up();
                ScreenAction::None
            }
            KeyCode::Down => {
                self.scroll_down();
                ScreenAction::None
            }
            KeyCode::Char('q') | KeyCode::Esc if self.has_error || self.is_complete => {
                ScreenAction::Exit
            }
            _ => ScreenAction::None,
        }
    }

    fn on_enter(&mut self) {
        // Installation will be started by the main app
    }

    fn title(&self) -> &str {
        "Installing NixOS"
    }

    fn help_content(&self) -> Vec<String> {
        vec![
            "# Installation Progress Screen".to_string(),
            "".to_string(),
            "NixOS is being installed to your system. This process may take".to_string(),
            "20-60 minutes depending on your internet connection and hardware.".to_string(),
            "".to_string(),
            "## What's Happening".to_string(),
            "".to_string(),
            "1. Generating Configuration: Creating your system configuration files".to_string(),
            "2. Hardware Detection: Detecting and configuring hardware".to_string(),
            "3. Installing Packages: Downloading and building system packages".to_string(),
            "4. Verification: Ensuring installation completed successfully".to_string(),
            "".to_string(),
            "## Progress Indicator".to_string(),
            "".to_string(),
            "The progress bar shows estimated completion:".to_string(),
            "- 0-20%: Configuration generation".to_string(),
            "- 20-30%: Hardware detection".to_string(),
            "- 30-90%: Package installation (most time spent here)".to_string(),
            "- 90-100%: Verification".to_string(),
            "".to_string(),
            "## Installation Log".to_string(),
            "".to_string(),
            "The log shows real-time output from the installation process.".to_string(),
            "Use ↑ and ↓ arrow keys to scroll through the log.".to_string(),
            "".to_string(),
            "## Important Notes".to_string(),
            "".to_string(),
            "- Do not interrupt the installation once it has started".to_string(),
            "- Keep your internet connection stable".to_string(),
            "- The system may appear to hang during package building - this is normal".to_string(),
            "- If installation fails, check the log for error details".to_string(),
            "".to_string(),
            "## After Installation".to_string(),
            "".to_string(),
            "Once complete, you'll be able to:".to_string(),
            "- Review the installation summary".to_string(),
            "- Reboot into your new NixOS system".to_string(),
            "- Access installation logs for troubleshooting".to_string(),
        ]
    }

    fn can_proceed(&self) -> bool {
        self.is_complete
    }

    fn can_go_back(&self) -> bool {
        false // Cannot go back during/after installation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installation_screen_creation() {
        let screen = InstallationScreen::new();
        assert_eq!(screen.title(), "Installing NixOS");
        assert!(!screen.is_running);
        assert!(!screen.is_complete);
        assert!(!screen.has_error);
    }

    #[test]
    fn test_installation_screen_cannot_go_back() {
        let screen = InstallationScreen::new();
        assert!(!screen.can_go_back());
    }

    #[test]
    fn test_installation_screen_can_proceed_when_complete() {
        let mut screen = InstallationScreen::new();
        assert!(!screen.can_proceed());

        screen.is_complete = true;
        assert!(screen.can_proceed());
    }

    #[test]
    fn test_scroll_functions() {
        let mut screen = InstallationScreen::new();
        screen.log_lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ];
        screen.scroll_position = 1;

        screen.scroll_down();
        assert_eq!(screen.scroll_position, 2);

        screen.scroll_up();
        assert_eq!(screen.scroll_position, 1);

        screen.scroll_up();
        assert_eq!(screen.scroll_position, 0);

        // Can't scroll past beginning
        screen.scroll_up();
        assert_eq!(screen.scroll_position, 0);
    }

    #[test]
    fn test_help_content() {
        let screen = InstallationScreen::new();
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("Installation Progress"));
    }
}
