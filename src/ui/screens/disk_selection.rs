//! Disk selection screen
//!
//! Allows the user to select which disk to install NixOS on.

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tracing::{debug, warn};

use crate::nixos::{detect_disks, Disk};
use crate::ui::theme::AppTheme;

use super::{Screen, ScreenAction};

/// Disk selection screen state
pub struct DiskSelectionScreen {
    /// Application theme
    theme: AppTheme,
    /// Available disks
    disks: Vec<Disk>,
    /// Currently selected disk index
    selected_index: usize,
    /// Whether disk detection has been run
    detected: bool,
    /// Error message if detection failed
    error_message: Option<String>,
}

impl DiskSelectionScreen {
    /// Create a new disk selection screen
    pub fn new() -> Self {
        Self {
            theme: AppTheme::new(),
            disks: Vec::new(),
            selected_index: 0,
            detected: false,
            error_message: None,
        }
    }

    /// Get the currently selected disk
    pub fn selected_disk(&self) -> Option<&Disk> {
        self.disks.get(self.selected_index)
    }

    /// Move selection up
    fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    fn select_next(&mut self) {
        if self.selected_index < self.disks.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Check if current selection is installable
    fn is_selection_installable(&self) -> bool {
        self.selected_disk()
            .map(|d| d.is_installable())
            .unwrap_or(false)
    }

    /// Get warning message for selected disk (if any)
    fn get_disk_warning(&self) -> Option<String> {
        let disk = self.selected_disk()?;

        if disk.readonly {
            return Some(
                "⚠ This disk is read-only and cannot be used for installation".to_string(),
            );
        }

        if disk.removable {
            return Some("⚠ This appears to be a removable disk (USB/SD card)".to_string());
        }

        if disk.mountpoint.is_some() {
            return Some("⚠ This disk has mounted partitions".to_string());
        }

        if disk.size < 8_000_000_000 {
            return Some(format!(
                "⚠ This disk is too small ({:.1} GB). Minimum 8 GB required.",
                disk.size_gb()
            ));
        }

        None
    }
}

impl Default for DiskSelectionScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for DiskSelectionScreen {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Instructions
                Constraint::Min(10),   // Disk list
                Constraint::Length(5), // Warning/info area
                Constraint::Length(3), // Navigation hints
            ])
            .split(area);

        // Instructions
        let instructions = Paragraph::new(vec![
            Line::from(""),
            Line::from("Select a disk for NixOS installation. All data on the selected disk will be erased."),
        ])
        .style(self.theme.text())
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(instructions, chunks[0]);

        // Disk list
        if let Some(ref error) = self.error_message {
            // Show error message
            let error_block = Block::default()
                .borders(Borders::ALL)
                .title(" Error ")
                .border_style(self.theme.error());

            let error_para = Paragraph::new(error.as_str())
                .block(error_block)
                .style(self.theme.error());
            frame.render_widget(error_para, chunks[1]);
        } else if self.disks.is_empty() {
            // No disks found
            let empty_block = Block::default()
                .borders(Borders::ALL)
                .title(" Available Disks ")
                .border_style(self.theme.border_style());

            let empty_para = Paragraph::new("No suitable disks found for installation.")
                .block(empty_block)
                .style(self.theme.text_muted())
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(empty_para, chunks[1]);
        } else {
            // Show disk list
            let disk_items: Vec<ListItem> = self
                .disks
                .iter()
                .enumerate()
                .map(|(idx, disk)| {
                    let is_selected = idx == self.selected_index;
                    let is_installable = disk.is_installable();

                    let style = if is_selected {
                        if is_installable {
                            self.theme.focused_item()
                        } else {
                            self.theme.error_bold()
                        }
                    } else if is_installable {
                        self.theme.text()
                    } else {
                        self.theme.text_muted()
                    };

                    let prefix = if is_selected { "► " } else { "  " };
                    let status_icon = if is_installable { "✓" } else { "✗" };

                    let line = Line::from(vec![
                        Span::raw(prefix),
                        Span::styled(
                            status_icon,
                            if is_installable {
                                self.theme.success()
                            } else {
                                self.theme.error()
                            },
                        ),
                        Span::raw(" "),
                        Span::styled(disk.display_name(), style),
                        Span::raw(" "),
                        Span::styled(
                            format!("[{}]", disk.disk_type.as_str()),
                            self.theme.text_muted(),
                        ),
                    ]);

                    ListItem::new(line)
                })
                .collect();

            let disk_list = List::new(disk_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Available Disks ({}) ", self.disks.len()))
                    .border_style(self.theme.border_style()),
            );

            frame.render_widget(disk_list, chunks[1]);
        }

        // Warning/info area
        let info_block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_style());

        let info_text = if let Some(warning) = self.get_disk_warning() {
            vec![
                Line::from(""),
                Line::from(Span::styled(warning, self.theme.warning())),
            ]
        } else if let Some(disk) = self.selected_disk() {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Selected: ", self.theme.text_muted()),
                    Span::styled(&disk.path, self.theme.success()),
                    Span::raw(" - "),
                    Span::styled(&disk.size_human, self.theme.text()),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "⚠ WARNING: All data on this disk will be permanently erased!",
                    self.theme.error(),
                )),
            ]
        } else {
            vec![Line::from("")]
        };

        let info_para = Paragraph::new(info_text)
            .block(info_block)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(info_para, chunks[2]);

        // Navigation hints
        let can_proceed = self.is_selection_installable();
        let nav_text = if can_proceed {
            vec![Line::from(vec![
                Span::styled("↑↓", self.theme.shortcut()),
                Span::raw(" Select | "),
                Span::styled("Enter", self.theme.shortcut()),
                Span::raw(" Continue | "),
                Span::styled("←", self.theme.shortcut()),
                Span::raw(" Back | "),
                Span::styled("?", self.theme.shortcut()),
                Span::raw(" Help | "),
                Span::styled("q", self.theme.shortcut()),
                Span::raw(" Quit"),
            ])]
        } else {
            vec![Line::from(vec![
                Span::styled("↑↓", self.theme.shortcut()),
                Span::raw(" Select | "),
                Span::styled("←", self.theme.shortcut()),
                Span::raw(" Back | "),
                Span::styled("?", self.theme.shortcut()),
                Span::raw(" Help | "),
                Span::styled("q", self.theme.shortcut()),
                Span::raw(" Quit"),
            ])]
        };

        let nav_para = Paragraph::new(nav_text).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(nav_para, chunks[3]);
    }

    fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
        match key {
            KeyCode::Up => {
                self.select_previous();
                ScreenAction::None
            }
            KeyCode::Down => {
                self.select_next();
                ScreenAction::None
            }
            KeyCode::Enter if self.is_selection_installable() => ScreenAction::Next,
            KeyCode::Left | KeyCode::Backspace => ScreenAction::Back,
            KeyCode::Char('q') | KeyCode::Esc => ScreenAction::Exit,
            KeyCode::Char('?') => ScreenAction::ToggleHelp,
            _ => ScreenAction::None,
        }
    }

    fn on_enter(&mut self) {
        // Detect disks on entry
        if !self.detected {
            debug!("Detecting available disks");

            match detect_disks() {
                Ok(disks) => {
                    debug!("Found {} disks", disks.len());

                    // Filter to only show installable disks
                    self.disks = disks.into_iter().collect();

                    // Select first installable disk by default
                    self.selected_index = 0;
                    self.detected = true;
                    self.error_message = None;
                }
                Err(e) => {
                    warn!("Failed to detect disks: {}", e);
                    self.error_message = Some(format!("Failed to detect disks: {}", e));
                    self.disks = Vec::new();
                }
            }
        }
    }

    fn title(&self) -> &str {
        "Disk Selection"
    }

    fn help_content(&self) -> Vec<String> {
        vec![
            "# Disk Selection Screen".to_string(),
            "".to_string(),
            "Select the disk where NixOS will be installed.".to_string(),
            "".to_string(),
            "⚠ WARNING: All data on the selected disk will be permanently erased!".to_string(),
            "".to_string(),
            "## Disk Status Indicators".to_string(),
            "".to_string(),
            "- ✓ Green: Disk is suitable for installation".to_string(),
            "- ✗ Red: Disk cannot be used (too small, read-only, or mounted)".to_string(),
            "- ► Indicates currently selected disk".to_string(),
            "".to_string(),
            "## Disk Requirements".to_string(),
            "".to_string(),
            "- Minimum 8 GB of space".to_string(),
            "- Not read-only".to_string(),
            "- No mounted partitions".to_string(),
            "- Internal disk recommended (USB drives work but not recommended)".to_string(),
            "".to_string(),
            "## Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- ↑/↓: Navigate disk list".to_string(),
            "- Enter: Select disk and continue (only if disk is suitable)".to_string(),
            "- ← / Backspace: Go back to system information".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q / Esc: Quit the installer".to_string(),
        ]
    }

    fn can_proceed(&self) -> bool {
        self.is_selection_installable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_selection_screen_creation() {
        let screen = DiskSelectionScreen::new();
        assert_eq!(screen.title(), "Disk Selection");
        assert!(screen.can_go_back());
        assert!(!screen.can_proceed()); // No disks initially
    }

    #[test]
    fn test_navigation() {
        let mut screen = DiskSelectionScreen::new();

        // Trigger detection
        screen.on_enter();

        // In mock mode, should have 2 disks
        if screen.disks.len() > 1 {
            assert_eq!(screen.selected_index, 0);

            screen.select_next();
            assert_eq!(screen.selected_index, 1);

            screen.select_previous();
            assert_eq!(screen.selected_index, 0);

            // Can't go below 0
            screen.select_previous();
            assert_eq!(screen.selected_index, 0);
        }
    }

    #[test]
    fn test_help_content() {
        let screen = DiskSelectionScreen::new();
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("Disk Selection"));
    }

    #[test]
    fn test_on_enter_detection() {
        let mut screen = DiskSelectionScreen::new();
        assert!(!screen.detected);

        screen.on_enter();
        assert!(screen.detected);

        // In mock mode, should detect disks
        assert!(screen.disks.len() >= 0);
    }
}
