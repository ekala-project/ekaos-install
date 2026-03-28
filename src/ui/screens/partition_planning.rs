//! Partition planning screen
//!
//! Shows the automatic partition layout that will be created.

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tracing::debug;

use crate::system::BootMode;
use crate::ui::{
    components::{Component, Focusable, InputField, Interactive},
    theme::AppTheme,
    utils::{keycode_to_input_event, render_navigation_hints},
};

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
    /// Editable swap size in GB
    swap_size_gb: u64,
    /// Swap size input field
    swap_input: InputField,
    /// Whether the swap field is focused
    swap_focused: bool,
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
        let mut swap_input = InputField::new("Swap Size (GB)");
        swap_input.set_value("8");
        swap_input.set_focused(false);

        Self {
            theme: AppTheme::new(),
            boot_mode: BootMode::Unknown,
            disk_path: String::new(),
            disk_size: 0,
            partitions: Vec::new(),
            swap_size_gb: 8,
            swap_input,
            swap_focused: false,
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
        let swap_size = self.swap_size_gb * 1_000_000_000; // User-defined GB
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
        let swap_size = self.swap_size_gb * 1_000_000_000; // User-defined GB
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

    /// Update swap size from input field and regenerate partitions
    fn update_swap_size(&mut self) {
        let value = self.swap_input.value();
        if let Ok(size) = value.parse::<u64>() {
            // Validate minimum and maximum
            let min_swap = 1; // 1 GB minimum
            let esp_size_gb = if matches!(self.boot_mode, BootMode::Uefi) { 1 } else { 0 };
            let max_swap = (self.disk_size / 1_000_000_000).saturating_sub(10 + esp_size_gb); // Leave 10GB for root

            let validated_size = size.max(min_swap).min(max_swap);

            if self.swap_size_gb != validated_size {
                self.swap_size_gb = validated_size;
                self.swap_input.set_value(&validated_size.to_string());
                self.generate_partition_layout();
            }
        }
    }

    /// Adjust swap size by delta (in GB)
    fn adjust_swap_size(&mut self, delta: i64) {
        let new_size = (self.swap_size_gb as i64 + delta).max(1) as u64;
        self.swap_input.set_value(&new_size.to_string());
        self.update_swap_size();
    }

    /// Render visual disk allocation bar
    fn render_disk_allocation_bar(&self, frame: &mut Frame<'_>, area: Rect) {
        let bar_block = Block::default()
            .borders(Borders::ALL)
            .title(" Disk Space Allocation ")
            .border_style(self.theme.border_style());

        // Calculate percentages
        let total_size = self.disk_size as f64;
        let mut segments = Vec::new();
        let mut colors = Vec::new();

        for (idx, partition) in self.partitions.iter().enumerate() {
            let percent = (partition.size as f64 / total_size * 100.0) as u16;
            segments.push((partition.label.clone(), percent, partition.size_human()));

            // Assign colors based on partition type
            colors.push(match idx {
                0 if self.partitions.len() == 3 => Color::Cyan,     // ESP
                _ if partition.fstype == "swap" => Color::Yellow,    // Swap
                _ => Color::Green,                                   // Root
            });
        }

        // Build the visual bar
        let bar_width = area.width.saturating_sub(4) as usize; // Account for borders
        let mut bar_chars = vec![' '; bar_width];
        let mut pos = 0;

        for (idx, (_, percent, _)) in segments.iter().enumerate() {
            let segment_width = (bar_width * (*percent as usize) / 100).max(1);
            let end_pos = (pos + segment_width).min(bar_width);

            for i in pos..end_pos {
                bar_chars[i] = '█';
            }
            pos = end_pos;
        }

        // Create the display with labels
        let mut lines = vec![Line::from("")];

        // Show bar
        let bar_line = Line::from(
            segments.iter().enumerate().map(|(idx, (_, percent, _))| {
                let segment_width = (bar_width * (*percent as usize) / 100).max(1);
                let chars: String = "█".repeat(segment_width);
                Span::styled(chars, self.theme.text().fg(colors[idx]))
            }).collect::<Vec<_>>()
        );
        lines.push(bar_line);
        lines.push(Line::from(""));

        // Show legend
        for (idx, (label, percent, size)) in segments.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled("  ██ ", self.theme.text().fg(colors[idx])),
                Span::styled(format!("{}: ", label), self.theme.text()),
                Span::styled(format!("{} ({}%)", size, percent), self.theme.text_muted()),
            ]));
        }

        let bar_para = Paragraph::new(lines)
            .block(bar_block)
            .style(self.theme.text());
        frame.render_widget(bar_para, area);
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
                Constraint::Length(9),  // Visual disk allocation bar
                Constraint::Length(5),  // Swap size input
                Constraint::Min(8),     // Partition list (smaller)
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

        // Visual disk allocation bar
        self.render_disk_allocation_bar(frame, chunks[1]);

        // Swap size input field
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(" Adjust Partition Sizes ")
            .border_style(if self.swap_focused {
                self.theme.border_focused_style()
            } else {
                self.theme.border_style()
            });

        let input_area = input_block.inner(chunks[2]);
        frame.render_widget(input_block, chunks[2]);

        // Create layout for input field
        let input_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Input field
                Constraint::Length(1), // Help text
            ])
            .split(input_area);

        // Update input field focus state
        self.swap_input.set_focused(self.swap_focused);
        Component::render(&mut self.swap_input, frame, input_layout[1]);

        // Help text for input
        let help_text = if self.swap_focused {
            "Type to adjust swap size | ↑↓ to increment/decrement | Tab to unfocus"
        } else {
            "Tab to edit swap size | ↑↓ to adjust by 1GB"
        };
        let help_para = Paragraph::new(Line::from(Span::styled(
            help_text,
            self.theme.text_muted(),
        )))
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(help_para, input_layout[2]);

        // Partition list
        let partition_items: Vec<ListItem> = self
            .partitions
            .iter()
            .map(|part| {
                let lines = vec![
                    Line::from(vec![
                        Span::styled(&part.label, self.theme.success_bold()),
                        Span::raw("  "),
                        Span::styled(format!("[{}]", part.size_human()), self.theme.text_muted()),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled("Type: ", self.theme.text_muted()),
                        Span::styled(&part.fstype, self.theme.text()),
                        Span::raw("  |  "),
                        Span::styled("Mount: ", self.theme.text_muted()),
                        Span::styled(&part.mountpoint, self.theme.text()),
                    ]),
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

        frame.render_widget(partition_list, chunks[3]);

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
        frame.render_widget(summary_para, chunks[4]);

        // Navigation hints
        render_navigation_hints(
            frame,
            &[
                ("Tab", "Edit"),
                ("↑↓", "Adjust"),
                ("←", "Back"),
                ("Enter", "Continue"),
                ("?", "Help"),
                ("q", "Quit"),
            ],
            &self.theme,
            chunks[5],
        );
    }

    fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
        // Try standard handlers first (quit, help)
        if let Some(action) = self.handle_standard_input(key) {
            return action;
        }
        // Try back handler (only when not focused on input)
        if !self.swap_focused {
            if let Some(action) = self.handle_back_input(key) {
                return action;
            }
        }

        // Handle screen-specific keys
        match key {
            KeyCode::Tab => {
                // Toggle focus on swap input field
                self.swap_focused = !self.swap_focused;
                self.swap_input.set_focused(self.swap_focused);
                if !self.swap_focused {
                    // Validate and update when losing focus
                    self.update_swap_size();
                }
                ScreenAction::None
            }
            KeyCode::Up => {
                // Adjust swap size up by 1 GB
                self.adjust_swap_size(1);
                ScreenAction::None
            }
            KeyCode::Down => {
                // Adjust swap size down by 1 GB
                self.adjust_swap_size(-1);
                ScreenAction::None
            }
            KeyCode::Enter => {
                if self.swap_focused {
                    // Unfocus and validate when Enter is pressed in input
                    self.swap_focused = false;
                    self.swap_input.set_focused(false);
                    self.update_swap_size();
                    ScreenAction::None
                } else {
                    // Continue to next screen
                    ScreenAction::Next
                }
            }
            _ => {
                // If swap field is focused, handle input events
                if self.swap_focused {
                    if let Some(event) = keycode_to_input_event(key) {
                        Interactive::handle_input(&mut self.swap_input, event);
                        // Don't validate on every keystroke, only when losing focus
                    }
                }
                ScreenAction::None
            }
        }
    }

    fn title(&self) -> &str {
        "Partition Planning"
    }

    fn help_content(&self) -> Vec<String> {
        vec![
            "# Partition Planning Screen".to_string(),
            "".to_string(),
            "This screen shows the partition layout that will be created.".to_string(),
            "You can adjust the swap partition size to suit your needs.".to_string(),
            "".to_string(),
            "## UEFI Layout".to_string(),
            "".to_string(),
            "- EFI System Partition (512 MB, FAT32): Required for UEFI boot".to_string(),
            "- Root Partition (remaining space, ext4): Main system files".to_string(),
            "- Swap Partition (adjustable, default 8 GB): Virtual memory and hibernation".to_string(),
            "".to_string(),
            "## BIOS Layout".to_string(),
            "".to_string(),
            "- Root Partition (remaining space, ext4): Main system files".to_string(),
            "- Swap Partition (adjustable, default 8 GB): Virtual memory and hibernation".to_string(),
            "".to_string(),
            "## Adjusting Partition Sizes".to_string(),
            "".to_string(),
            "You can adjust the swap partition size:".to_string(),
            "- Use Tab to focus the swap size input field".to_string(),
            "- Use ↑↓ arrow keys to adjust by 1 GB increments".to_string(),
            "- Type directly to enter a specific size".to_string(),
            "- Press Enter or Tab again to apply changes".to_string(),
            "".to_string(),
            "The visual allocation bar shows how disk space is divided.".to_string(),
            "Minimum swap size is 1 GB, maximum depends on disk size.".to_string(),
            "".to_string(),
            "## Important Notes".to_string(),
            "".to_string(),
            "- All data on the selected disk will be PERMANENTLY ERASED".to_string(),
            "- Partition sizes are optimized for typical installations".to_string(),
            "- Root partition automatically adjusts when you change swap size".to_string(),
            "- Root partition uses ext4 filesystem (reliable and well-tested)".to_string(),
            "".to_string(),
            "## Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- Tab: Focus/unfocus the swap size input field".to_string(),
            "- ↑↓: Adjust swap size by 1 GB".to_string(),
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

    #[test]
    fn test_swap_size_adjustment() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Default swap size is 8 GB
        assert_eq!(screen.swap_size_gb, 8);
        assert_eq!(screen.partitions[2].size, 8_000_000_000);

        // Adjust swap up by 2 GB
        screen.adjust_swap_size(2);
        assert_eq!(screen.swap_size_gb, 10);
        assert_eq!(screen.partitions[2].size, 10_000_000_000);

        // Adjust swap down by 5 GB
        screen.adjust_swap_size(-5);
        assert_eq!(screen.swap_size_gb, 5);
        assert_eq!(screen.partitions[2].size, 5_000_000_000);
    }

    #[test]
    fn test_swap_size_minimum_validation() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Try to set swap to 0 GB - should clamp to 1 GB minimum
        screen.adjust_swap_size(-10);
        assert_eq!(screen.swap_size_gb, 1);
        assert_eq!(screen.partitions[2].size, 1_000_000_000);
    }

    #[test]
    fn test_swap_size_maximum_validation() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 100_000_000_000); // 100 GB disk

        // Try to set swap to 95 GB - should be clamped to leave room for root and ESP
        screen.swap_input.set_value("95");
        screen.update_swap_size();

        // Max should be disk_size_gb - 10 (root min) - 1 (ESP) = 100 - 10 - 1 = 89 GB
        assert_eq!(screen.swap_size_gb, 89);
    }

    #[test]
    fn test_swap_size_update_from_input() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Bios, "/dev/sda".to_string(), 500_000_000_000);

        // Set swap size via input field
        screen.swap_input.set_value("16");
        screen.update_swap_size();

        assert_eq!(screen.swap_size_gb, 16);
        assert_eq!(screen.partitions[1].size, 16_000_000_000);

        // Root partition should have adjusted accordingly
        let expected_root = 500_000_000_000 - 16_000_000_000;
        assert_eq!(screen.partitions[0].size, expected_root);
    }

    #[test]
    fn test_swap_focus_state() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Initially not focused
        assert!(!screen.swap_focused);

        // Simulate Tab key to focus
        let action = screen.handle_input(KeyCode::Tab);
        assert_eq!(action, ScreenAction::None);
        assert!(screen.swap_focused);

        // Simulate Tab again to unfocus
        let action = screen.handle_input(KeyCode::Tab);
        assert_eq!(action, ScreenAction::None);
        assert!(!screen.swap_focused);
    }

    #[test]
    fn test_arrow_key_adjustment() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        assert_eq!(screen.swap_size_gb, 8);

        // Test Up arrow
        screen.handle_input(KeyCode::Up);
        assert_eq!(screen.swap_size_gb, 9);

        // Test Down arrow
        screen.handle_input(KeyCode::Down);
        assert_eq!(screen.swap_size_gb, 8);
    }

    #[test]
    fn test_enter_key_behavior() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // When not focused, Enter should proceed
        let action = screen.handle_input(KeyCode::Enter);
        assert_eq!(action, ScreenAction::Next);

        // When focused, Enter should unfocus
        screen.swap_focused = true;
        let action = screen.handle_input(KeyCode::Enter);
        assert_eq!(action, ScreenAction::None);
        assert!(!screen.swap_focused);
    }
}
