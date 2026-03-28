//! Partition planning screen
//!
//! Shows the automatic partition layout that will be created.

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tracing::debug;

use crate::system::BootMode;
use crate::ui::theme::AppTheme;

use super::{Screen, ScreenAction};

/// Partition planning screen state
pub struct PartitionPlanningScreen {
    /// Application theme
    theme: AppTheme,
    /// Boot mode (affects partition layout)
    boot_mode: BootMode,
    /// Selected disk path
    disk_path: String,
    /// Selected disk size in bytes
    disk_size: u64,
    /// Partition layout
    partitions: Vec<Partition>,
}

/// Partition information
#[derive(Debug, Clone)]
struct Partition {
    /// Partition label/name
    label: String,
    /// Size in bytes
    size: u64,
    /// Filesystem type
    fstype: String,
    /// Mount point
    mountpoint: String,
    /// Description
    description: String,
}

impl Partition {
    /// Get human-readable size
    fn size_human(&self) -> String {
        let gb = self.size as f64 / 1_000_000_000.0;
        if gb < 1.0 {
            format!("{} MB", (self.size as f64 / 1_000_000.0) as u64)
        } else {
            format!("{:.1} GB", gb)
        }
    }
}

impl PartitionPlanningScreen {
    /// Create a new partition planning screen
    pub fn new() -> Self {
        Self {
            theme: AppTheme::new(),
            boot_mode: BootMode::Unknown,
            disk_path: String::new(),
            disk_size: 0,
            partitions: Vec::new(),
        }
    }

    /// Set the selected disk information
    pub fn set_disk(&mut self, boot_mode: BootMode, disk_path: String, disk_size: u64) {
        self.boot_mode = boot_mode;
        self.disk_path = disk_path;
        self.disk_size = disk_size;
        self.generate_partition_layout();
    }

    /// Generate partition layout based on boot mode
    fn generate_partition_layout(&mut self) {
        debug!(
            "Generating partition layout for {:?} boot mode on {}",
            self.boot_mode, self.disk_path
        );

        self.partitions.clear();

        match self.boot_mode {
            BootMode::Uefi => self.generate_uefi_layout(),
            BootMode::Bios => self.generate_bios_layout(),
            BootMode::Unknown => {
                // Default to UEFI if unknown
                debug!("Unknown boot mode, defaulting to UEFI layout");
                self.generate_uefi_layout();
            }
        }
    }

    /// Generate UEFI partition layout
    fn generate_uefi_layout(&mut self) {
        let esp_size = 512 * 1_000_000; // 512 MB
        let swap_size = 8 * 1_000_000_000; // 8 GB
        let root_size = self.disk_size.saturating_sub(esp_size + swap_size);

        self.partitions = vec![
            Partition {
                label: "EFI System Partition".to_string(),
                size: esp_size,
                fstype: "FAT32".to_string(),
                mountpoint: "/boot".to_string(),
                description: "Required for UEFI boot".to_string(),
            },
            Partition {
                label: "Root Partition".to_string(),
                size: root_size,
                fstype: "ext4".to_string(),
                mountpoint: "/".to_string(),
                description: "Main system partition".to_string(),
            },
            Partition {
                label: "Swap Partition".to_string(),
                size: swap_size,
                fstype: "swap".to_string(),
                mountpoint: "[swap]".to_string(),
                description: "Virtual memory / hibernation".to_string(),
            },
        ];
    }

    /// Generate BIOS partition layout
    fn generate_bios_layout(&mut self) {
        let swap_size = 8 * 1_000_000_000; // 8 GB
        let root_size = self.disk_size.saturating_sub(swap_size);

        self.partitions = vec![
            Partition {
                label: "Root Partition".to_string(),
                size: root_size,
                fstype: "ext4".to_string(),
                mountpoint: "/".to_string(),
                description: "Main system partition".to_string(),
            },
            Partition {
                label: "Swap Partition".to_string(),
                size: swap_size,
                fstype: "swap".to_string(),
                mountpoint: "[swap]".to_string(),
                description: "Virtual memory / hibernation".to_string(),
            },
        ];
    }

    /// Get total allocated size
    fn total_size(&self) -> u64 {
        self.partitions.iter().map(|p| p.size).sum()
    }
}

impl Default for PartitionPlanningScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for PartitionPlanningScreen {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // Disk info
                Constraint::Length(3),  // Instructions
                Constraint::Min(10),    // Partition list
                Constraint::Length(5),  // Summary
                Constraint::Length(3),  // Navigation hints
            ])
            .split(area);

        // Disk info
        let disk_block = Block::default()
            .borders(Borders::ALL)
            .title(" Selected Disk ")
            .border_style(self.theme.border_style());

        let disk_info = vec![
            Line::from(vec![
                Span::styled("Device: ", self.theme.text_muted()),
                Span::styled(&self.disk_path, self.theme.text()),
            ]),
            Line::from(vec![
                Span::styled("Size: ", self.theme.text_muted()),
                Span::styled(
                    format!("{:.1} GB", self.disk_size as f64 / 1_000_000_000.0),
                    self.theme.text(),
                ),
                Span::raw("  |  "),
                Span::styled("Boot Mode: ", self.theme.text_muted()),
                Span::styled(self.boot_mode.as_str(), self.theme.info()),
            ]),
        ];

        let disk_para = Paragraph::new(disk_info).block(disk_block);
        frame.render_widget(disk_para, chunks[0]);

        // Instructions
        let instructions = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "The following partition layout will be created automatically:",
                self.theme.text(),
            )),
        ])
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(instructions, chunks[1]);

        // Partition list
        let partition_items: Vec<ListItem> = self
            .partitions
            .iter()
            .map(|part| {
                let lines = vec![
                    Line::from(vec![
                        Span::styled(&part.label, self.theme.success_bold()),
                        Span::raw("  "),
                        Span::styled(
                            format!("[{}]", part.size_human()),
                            self.theme.text_muted(),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled("Type: ", self.theme.text_muted()),
                        Span::styled(&part.fstype, self.theme.text()),
                        Span::raw("  |  "),
                        Span::styled("Mount: ", self.theme.text_muted()),
                        Span::styled(&part.mountpoint, self.theme.text()),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(&part.description, self.theme.text_muted()),
                    ]),
                    Line::from(""),
                ];

                ListItem::new(lines)
            })
            .collect();

        let partition_list = List::new(partition_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Partition Layout ")
                .border_style(self.theme.border_style()),
        );

        frame.render_widget(partition_list, chunks[2]);

        // Summary
        let summary_block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_style());

        let total = self.total_size();
        let total_gb = total as f64 / 1_000_000_000.0;

        let summary_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Total allocated: ", self.theme.text_muted()),
                Span::styled(format!("{:.1} GB", total_gb), self.theme.success()),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "⚠ WARNING: All existing data on this disk will be permanently erased!",
                self.theme.error(),
            )),
        ];

        let summary_para = Paragraph::new(summary_text)
            .block(summary_block)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(summary_para, chunks[3]);

        // Navigation hints
        let nav_text = vec![Line::from(vec![
            Span::styled("←", self.theme.shortcut()),
            Span::raw(" Back | "),
            Span::styled("Enter", self.theme.shortcut()),
            Span::raw(" Continue | "),
            Span::styled("?", self.theme.shortcut()),
            Span::raw(" Help | "),
            Span::styled("q", self.theme.shortcut()),
            Span::raw(" Quit"),
        ])];

        let nav_para = Paragraph::new(nav_text).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(nav_para, chunks[4]);
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

    fn title(&self) -> &str {
        "Partition Planning"
    }

    fn help_content(&self) -> Vec<String> {
        vec![
            "# Partition Planning Screen".to_string(),
            "".to_string(),
            "This screen shows the automatic partition layout that will be created.".to_string(),
            "".to_string(),
            "## UEFI Layout".to_string(),
            "".to_string(),
            "- EFI System Partition (512 MB, FAT32): Required for UEFI boot".to_string(),
            "- Root Partition (remaining space, ext4): Main system files".to_string(),
            "- Swap Partition (8 GB): Virtual memory and hibernation".to_string(),
            "".to_string(),
            "## BIOS Layout".to_string(),
            "".to_string(),
            "- Root Partition (remaining space, ext4): Main system files".to_string(),
            "- Swap Partition (8 GB): Virtual memory and hibernation".to_string(),
            "".to_string(),
            "## Important Notes".to_string(),
            "".to_string(),
            "- All data on the selected disk will be PERMANENTLY ERASED".to_string(),
            "- Partition sizes are optimized for typical installations".to_string(),
            "- Swap size is fixed at 8 GB (suitable for most systems)".to_string(),
            "- Root partition uses ext4 filesystem (reliable and well-tested)".to_string(),
            "".to_string(),
            "## Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- Enter: Accept layout and continue".to_string(),
            "- ← / Backspace: Go back to disk selection".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q / Esc: Quit the installer".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_planning_screen_creation() {
        let screen = PartitionPlanningScreen::new();
        assert_eq!(screen.title(), "Partition Planning");
        assert!(screen.can_go_back());
        assert!(screen.can_proceed());
    }

    #[test]
    fn test_uefi_layout_generation() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        assert_eq!(screen.partitions.len(), 3);
        assert_eq!(screen.partitions[0].label, "EFI System Partition");
        assert_eq!(screen.partitions[0].fstype, "FAT32");
        assert_eq!(screen.partitions[1].label, "Root Partition");
        assert_eq!(screen.partitions[2].label, "Swap Partition");
    }

    #[test]
    fn test_bios_layout_generation() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Bios, "/dev/sda".to_string(), 500_000_000_000);

        assert_eq!(screen.partitions.len(), 2);
        assert_eq!(screen.partitions[0].label, "Root Partition");
        assert_eq!(screen.partitions[1].label, "Swap Partition");
    }

    #[test]
    fn test_partition_size_human() {
        let part = Partition {
            label: "Test".to_string(),
            size: 512_000_000,
            fstype: "ext4".to_string(),
            mountpoint: "/".to_string(),
            description: "Test".to_string(),
        };
        assert_eq!(part.size_human(), "512 MB");

        let part2 = Partition {
            label: "Test".to_string(),
            size: 8_000_000_000,
            fstype: "ext4".to_string(),
            mountpoint: "/".to_string(),
            description: "Test".to_string(),
        };
        assert_eq!(part2.size_human(), "8.0 GB");
    }

    #[test]
    fn test_help_content() {
        let screen = PartitionPlanningScreen::new();
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("Partition Planning"));
    }
}
