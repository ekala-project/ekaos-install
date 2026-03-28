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

/// Available filesystem options for root partition
const ROOT_FILESYSTEM_OPTIONS: &[(&str, &str)] = &[
    ("ext4", "Reliable journaling filesystem (recommended)"),
    ("btrfs", "Modern CoW filesystem with snapshots"),
    ("xfs", "High-performance journaling filesystem"),
    ("zfs", "Advanced filesystem with data integrity"),
];

/// Focus mode for two-level navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusMode {
    /// Browsing partitions with Up/Down
    PartitionList,
    /// Editing fields within selected partition
    PartitionEdit,
}

/// Editable field within a partition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditableField {
    /// Size field (only for swap partition)
    Size,
    /// Filesystem type (only for root partition)
    FilesystemType,
}

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
    /// Selected partition index for cursor navigation
    selected_partition_idx: usize,
    /// Current focus mode (partition list vs editing)
    focus_mode: FocusMode,
    /// Selected field within partition (when in edit mode)
    selected_field: Option<EditableField>,
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
            selected_partition_idx: 0,
            focus_mode: FocusMode::PartitionList,
            selected_field: None,
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

    // ===== Partition Navigation Methods =====

    /// Select previous partition in the list
    fn select_previous_partition(&mut self) {
        if self.selected_partition_idx > 0 {
            self.selected_partition_idx -= 1;
        }
    }

    /// Select next partition in the list
    fn select_next_partition(&mut self) {
        if self.selected_partition_idx < self.partitions.len().saturating_sub(1) {
            self.selected_partition_idx += 1;
        }
    }

    /// Enter edit mode for the selected partition
    fn enter_edit_mode(&mut self) {
        if self.can_edit_partition(self.selected_partition_idx) {
            self.focus_mode = FocusMode::PartitionEdit;
            // Select the first editable field
            let fields = self.get_editable_fields(self.selected_partition_idx);
            self.selected_field = fields.first().copied();
        }
    }

    /// Exit edit mode and return to partition list
    fn exit_edit_mode(&mut self) {
        self.focus_mode = FocusMode::PartitionList;
        self.selected_field = None;
        // If we were editing swap size via InputField, unfocus it
        if self.swap_focused {
            self.swap_focused = false;
            self.swap_input.set_focused(false);
            self.update_swap_size();
        }
    }

    /// Select previous field within the current partition
    fn select_previous_field(&mut self) {
        let fields = self.get_editable_fields(self.selected_partition_idx);
        if fields.is_empty() {
            return;
        }

        if let Some(current_field) = self.selected_field {
            if let Some(idx) = fields.iter().position(|f| *f == current_field) {
                if idx > 0 {
                    self.selected_field = Some(fields[idx - 1]);
                }
            }
        }
    }

    /// Select next field within the current partition
    fn select_next_field(&mut self) {
        let fields = self.get_editable_fields(self.selected_partition_idx);
        if fields.is_empty() {
            return;
        }

        if let Some(current_field) = self.selected_field {
            if let Some(idx) = fields.iter().position(|f| *f == current_field) {
                if idx < fields.len() - 1 {
                    self.selected_field = Some(fields[idx + 1]);
                }
            }
        }
    }

    // ===== Filesystem Management Methods =====

    /// Cycle through filesystem options (direction: -1 left, +1 right)
    fn cycle_filesystem(&mut self, direction: i32) {
        if self.selected_field != Some(EditableField::FilesystemType) {
            return;
        }

        let partition_idx = self.selected_partition_idx;
        if partition_idx >= self.partitions.len() {
            return;
        }

        let current_fs = &self.partitions[partition_idx].fstype;
        let options: Vec<&str> = ROOT_FILESYSTEM_OPTIONS.iter().map(|(fs, _)| *fs).collect();

        let current_idx = options.iter().position(|&fs| fs == current_fs).unwrap_or(0);

        let new_idx = if direction > 0 {
            (current_idx + 1) % options.len()
        } else {
            (current_idx + options.len() - 1) % options.len()
        };

        self.partitions[partition_idx].fstype = options[new_idx].to_string();
    }

    /// Get list of editable fields for a partition
    fn get_editable_fields(&self, partition_idx: usize) -> Vec<EditableField> {
        if partition_idx >= self.partitions.len() {
            return Vec::new();
        }

        let partition = &self.partitions[partition_idx];
        let mut fields = Vec::new();

        // Root partition can edit filesystem type
        if partition.mountpoint == "/" {
            fields.push(EditableField::FilesystemType);
        }

        // Swap partition can edit size
        if partition.fstype == "swap" {
            fields.push(EditableField::Size);
        }

        fields
    }

    /// Check if a partition can be edited
    fn can_edit_partition(&self, partition_idx: usize) -> bool {
        !self.get_editable_fields(partition_idx).is_empty()
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

        // Partition list - hierarchical display with cursor
        let mut partition_lines: Vec<Line> = Vec::new();
        partition_lines.push(Line::from("")); // Top padding

        for (idx, partition) in self.partitions.iter().enumerate() {
            let is_selected = idx == self.selected_partition_idx;
            let is_in_edit_mode = is_selected && self.focus_mode == FocusMode::PartitionEdit;
            let can_edit = self.can_edit_partition(idx);

            // Partition header line with cursor
            let cursor = if is_selected && self.focus_mode == FocusMode::PartitionList {
                "> "
            } else {
                "  "
            };

            let header_style = if is_selected && self.focus_mode == FocusMode::PartitionList {
                self.theme.focused_item()
            } else {
                self.theme.text()
            };

            partition_lines.push(Line::from(vec![
                Span::styled(cursor, header_style),
                Span::styled(&partition.label, header_style),
                Span::styled(
                    format!("  [{}]", partition.size_human()),
                    if is_selected { self.theme.text() } else { self.theme.text_muted() }
                ),
            ]));

            // Get editable fields for this partition
            let editable_fields = self.get_editable_fields(idx);

            // Size field (for swap partition)
            if editable_fields.contains(&EditableField::Size) {
                let is_size_focused = is_in_edit_mode && self.selected_field == Some(EditableField::Size);
                let field_cursor = if is_size_focused { "  > " } else { "    " };
                let field_style = if is_size_focused {
                    self.theme.focused_item()
                } else {
                    self.theme.text()
                };

                partition_lines.push(Line::from(vec![
                    Span::styled(field_cursor, field_style),
                    Span::styled("Size: ", if is_size_focused { self.theme.text() } else { self.theme.text_muted() }),
                    Span::styled(partition.size_human(), field_style),
                ]));
            } else {
                // Show size as read-only
                partition_lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("Size: ", self.theme.text_muted()),
                    Span::styled(format!("{} (fixed)", partition.size_human()), self.theme.text()),
                ]));
            }

            // Filesystem type field
            if editable_fields.contains(&EditableField::FilesystemType) {
                let is_type_focused = is_in_edit_mode && self.selected_field == Some(EditableField::FilesystemType);
                let field_cursor = if is_type_focused { "  > " } else { "    " };

                // Show filesystem options with current selection
                let mut fs_spans = vec![
                    Span::styled(field_cursor, if is_type_focused { self.theme.focused_item() } else { self.theme.text() }),
                    Span::styled("Type: ", if is_type_focused { self.theme.text() } else { self.theme.text_muted() }),
                ];

                let options: Vec<&str> = ROOT_FILESYSTEM_OPTIONS.iter().map(|(fs, _)| *fs).collect();
                for (i, &fs) in options.iter().enumerate() {
                    if i > 0 {
                        fs_spans.push(Span::styled("  ", self.theme.text_muted()));
                    }

                    if fs == partition.fstype {
                        fs_spans.push(Span::styled(
                            format!("{} ◄", fs),
                            if is_type_focused { self.theme.focused_item() } else { self.theme.success() }
                        ));
                    } else if is_type_focused {
                        fs_spans.push(Span::styled(fs, self.theme.text_muted()));
                    }
                }

                partition_lines.push(Line::from(fs_spans));
            } else {
                // Show filesystem type as read-only
                partition_lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("Type: ", self.theme.text_muted()),
                    Span::styled(&partition.fstype, self.theme.text()),
                ]));
            }

            // Mount point (always read-only)
            partition_lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("Mount: ", self.theme.text_muted()),
                Span::styled(&partition.mountpoint, self.theme.text()),
            ]));

            // Spacing between partitions
            partition_lines.push(Line::from(""));
        }

        let partition_para = Paragraph::new(partition_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Partition Layout ")
                .border_style(self.theme.border_style()),
        );

        frame.render_widget(partition_para, chunks[3]);

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

        // Navigation hints - context-sensitive based on focus mode
        let hints = match self.focus_mode {
            FocusMode::PartitionList => vec![
                ("↑↓", "Navigate"),
                ("Enter", "Edit Partition"),
                ("Tab", "Quick Edit Swap"),
                ("→", "Continue"),
                ("?", "Help"),
                ("q", "Quit"),
            ],
            FocusMode::PartitionEdit => {
                if self.selected_field == Some(EditableField::FilesystemType) {
                    vec![
                        ("↑↓", "Navigate Fields"),
                        ("←→", "Change Filesystem"),
                        ("Enter", "Confirm"),
                        ("Esc", "Cancel"),
                        ("?", "Help"),
                    ]
                } else {
                    vec![
                        ("↑↓", "Navigate Fields"),
                        ("Enter", "Edit Size"),
                        ("Esc", "Exit Edit Mode"),
                        ("?", "Help"),
                    ]
                }
            }
        };

        render_navigation_hints(frame, &hints, &self.theme, chunks[5]);
    }

    fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
        // Handle Escape specially based on mode
        if key == KeyCode::Esc {
            if self.swap_focused {
                // Exit swap input field
                self.swap_focused = false;
                self.swap_input.set_focused(false);
                self.update_swap_size();
                return ScreenAction::None;
            } else if self.focus_mode == FocusMode::PartitionEdit {
                // Exit edit mode
                self.exit_edit_mode();
                return ScreenAction::None;
            } else {
                // In partition list mode, Escape quits
                return ScreenAction::Exit;
            }
        }

        // Try standard handlers for other keys (quit with 'q', help with '?')
        if let Some(action) = self.handle_standard_input(key) {
            return action;
        }

        // If swap input field is focused, handle that separately
        if self.swap_focused {
            match key {
                KeyCode::Tab | KeyCode::Enter => {
                    // Unfocus and validate when exiting swap input
                    self.swap_focused = false;
                    self.swap_input.set_focused(false);
                    self.update_swap_size();
                    ScreenAction::None
                }
                _ => {
                    // Pass input to the InputField
                    if let Some(event) = keycode_to_input_event(key) {
                        Interactive::handle_input(&mut self.swap_input, event);
                    }
                    ScreenAction::None
                }
            }
        } else {
            // Handle two-level navigation based on focus mode
            match self.focus_mode {
                FocusMode::PartitionList => {
                    // Try back handler in partition list mode
                    if let Some(action) = self.handle_back_input(key) {
                        return action;
                    }

                    match key {
                        KeyCode::Up => {
                            self.select_previous_partition();
                            ScreenAction::None
                        }
                        KeyCode::Down => {
                            self.select_next_partition();
                            ScreenAction::None
                        }
                        KeyCode::Enter => {
                            // Enter edit mode for the selected partition
                            self.enter_edit_mode();
                            ScreenAction::None
                        }
                        KeyCode::Right => {
                            // Continue to next screen (only in list mode)
                            ScreenAction::Next
                        }
                        KeyCode::Tab => {
                            // Quick access to swap size input (backward compatibility)
                            self.swap_focused = true;
                            self.swap_input.set_focused(true);
                            ScreenAction::None
                        }
                        _ => ScreenAction::None,
                    }
                }
                FocusMode::PartitionEdit => {
                    match key {
                        KeyCode::Up => {
                            self.select_previous_field();
                            ScreenAction::None
                        }
                        KeyCode::Down => {
                            self.select_next_field();
                            ScreenAction::None
                        }
                        KeyCode::Left => {
                            // Cycle filesystem left (only if on FilesystemType field)
                            self.cycle_filesystem(-1);
                            ScreenAction::None
                        }
                        KeyCode::Right => {
                            // Cycle filesystem right (only if on FilesystemType field)
                            self.cycle_filesystem(1);
                            ScreenAction::None
                        }
                        KeyCode::Enter => {
                            if self.selected_field == Some(EditableField::Size) {
                                // If on Size field, focus the input
                                self.swap_focused = true;
                                self.swap_input.set_focused(true);
                                ScreenAction::None
                            } else {
                                // If on FilesystemType or any other field, exit edit mode
                                self.exit_edit_mode();
                                ScreenAction::None
                            }
                        }
                        _ => ScreenAction::None,
                    }
                }
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
            "This screen shows an interactive partition layout editor.".to_string(),
            "You can customize partition sizes and filesystem types.".to_string(),
            "".to_string(),
            "## Interactive Navigation".to_string(),
            "".to_string(),
            "The partition list uses a hierarchical menu with '>' cursor:".to_string(),
            "- Navigate partitions with ↑↓ arrow keys".to_string(),
            "- Press Enter on a partition to edit its properties".to_string(),
            "- In edit mode, navigate fields with ↑↓".to_string(),
            "- Press Escape to exit edit mode".to_string(),
            "".to_string(),
            "## Default Layout".to_string(),
            "".to_string(),
            "UEFI Mode:".to_string(),
            "- EFI System Partition (512 MB, FAT32) - fixed".to_string(),
            "- Root Partition (remaining space) - filesystem editable".to_string(),
            "- Swap Partition (8 GB default) - size editable".to_string(),
            "".to_string(),
            "BIOS Mode:".to_string(),
            "- Root Partition (remaining space) - filesystem editable".to_string(),
            "- Swap Partition (8 GB default) - size editable".to_string(),
            "".to_string(),
            "## Editing Partitions".to_string(),
            "".to_string(),
            "Root Partition - Filesystem Type:".to_string(),
            "1. Navigate to root partition and press Enter".to_string(),
            "2. Use ←→ arrow keys to cycle through filesystem options:".to_string(),
            "   • ext4 (recommended) - Reliable journaling filesystem".to_string(),
            "   • btrfs - Modern CoW with snapshots and compression".to_string(),
            "   • xfs - High-performance journaling filesystem".to_string(),
            "   • zfs - Advanced filesystem with data integrity".to_string(),
            "3. Press Enter to confirm or Escape to cancel".to_string(),
            "".to_string(),
            "Swap Partition - Size:".to_string(),
            "1. Navigate to swap partition and press Enter".to_string(),
            "2. Navigate to Size field with ↑↓".to_string(),
            "3. Press Enter to type a specific size".to_string(),
            "4. Or use Tab for quick access to swap size input".to_string(),
            "   - Minimum: 1 GB, Maximum: disk size minus space for root".to_string(),
            "".to_string(),
            "## Visual Features".to_string(),
            "".to_string(),
            "- Disk allocation bar shows space distribution with colors".to_string(),
            "- '>' cursor shows current selection".to_string(),
            "- Bold cyan text indicates focused items".to_string(),
            "- '◄' marker shows currently selected filesystem".to_string(),
            "- '(fixed)' label indicates non-editable fields".to_string(),
            "".to_string(),
            "## Important Notes".to_string(),
            "".to_string(),
            "- All data on the selected disk will be PERMANENTLY ERASED".to_string(),
            "- Mount points are fixed for safety (ESP=/boot, Root=/, Swap=[swap])".to_string(),
            "- ESP partition size is fixed at 512 MB (UEFI requirement)".to_string(),
            "- Root partition adjusts automatically when swap size changes".to_string(),
            "".to_string(),
            "## Keyboard Shortcuts".to_string(),
            "".to_string(),
            "Partition List Mode:".to_string(),
            "- ↑↓: Navigate between partitions".to_string(),
            "- Enter: Edit selected partition".to_string(),
            "- Tab: Quick access to swap size input".to_string(),
            "- →: Continue to next screen".to_string(),
            "".to_string(),
            "Partition Edit Mode:".to_string(),
            "- ↑↓: Navigate between fields".to_string(),
            "- ←→: Change filesystem type (when on Type field)".to_string(),
            "- Enter: Confirm selection and exit edit mode".to_string(),
            "- Escape: Cancel changes and exit edit mode".to_string(),
            "".to_string(),
            "Always Available:".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q: Quit the installer".to_string(),
            "- ← / Backspace: Go back to disk selection".to_string(),
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
    fn test_arrow_key_navigation() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Up/Down arrows now navigate partitions, not adjust swap size
        assert_eq!(screen.selected_partition_idx, 0);

        // Test Down arrow - navigates to next partition
        screen.handle_input(KeyCode::Down);
        assert_eq!(screen.selected_partition_idx, 1);

        // Test Up arrow - navigates to previous partition
        screen.handle_input(KeyCode::Up);
        assert_eq!(screen.selected_partition_idx, 0);
    }

    #[test]
    fn test_enter_key_behavior() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // When not focused, Enter should proceed
        let action = screen.handle_input(KeyCode::Enter);
        assert_eq!(action, ScreenAction::None); // Now enters edit mode instead

        // When focused, Enter should unfocus
        screen.swap_focused = true;
        let action = screen.handle_input(KeyCode::Enter);
        assert_eq!(action, ScreenAction::None);
        assert!(!screen.swap_focused);
    }

    // ===== New tests for interactive partition editing =====

    #[test]
    fn test_initial_focus_mode() {
        let screen = PartitionPlanningScreen::new();
        assert_eq!(screen.focus_mode, FocusMode::PartitionList);
        assert_eq!(screen.selected_partition_idx, 0);
        assert_eq!(screen.selected_field, None);
    }

    #[test]
    fn test_partition_navigation() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        assert_eq!(screen.selected_partition_idx, 0);

        // Navigate down
        screen.handle_input(KeyCode::Down);
        assert_eq!(screen.selected_partition_idx, 1);

        screen.handle_input(KeyCode::Down);
        assert_eq!(screen.selected_partition_idx, 2);

        // Can't go past last partition
        screen.handle_input(KeyCode::Down);
        assert_eq!(screen.selected_partition_idx, 2);

        // Navigate up
        screen.handle_input(KeyCode::Up);
        assert_eq!(screen.selected_partition_idx, 1);

        screen.handle_input(KeyCode::Up);
        assert_eq!(screen.selected_partition_idx, 0);

        // Can't go before first partition
        screen.handle_input(KeyCode::Up);
        assert_eq!(screen.selected_partition_idx, 0);
    }

    #[test]
    fn test_enter_edit_mode_on_root_partition() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Navigate to root partition (index 1)
        screen.selected_partition_idx = 1;

        // Enter edit mode
        screen.handle_input(KeyCode::Enter);

        assert_eq!(screen.focus_mode, FocusMode::PartitionEdit);
        assert_eq!(screen.selected_field, Some(EditableField::FilesystemType));
    }

    #[test]
    fn test_enter_edit_mode_on_swap_partition() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Navigate to swap partition (index 2)
        screen.selected_partition_idx = 2;

        // Enter edit mode
        screen.handle_input(KeyCode::Enter);

        assert_eq!(screen.focus_mode, FocusMode::PartitionEdit);
        assert_eq!(screen.selected_field, Some(EditableField::Size));
    }

    #[test]
    fn test_cannot_edit_esp_partition() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // ESP is at index 0
        screen.selected_partition_idx = 0;

        // Try to enter edit mode - should fail
        screen.handle_input(KeyCode::Enter);

        // Should still be in partition list mode
        assert_eq!(screen.focus_mode, FocusMode::PartitionList);
        assert_eq!(screen.selected_field, None);
    }

    #[test]
    fn test_exit_edit_mode() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Enter edit mode on root partition
        screen.selected_partition_idx = 1;
        screen.handle_input(KeyCode::Enter);
        assert_eq!(screen.focus_mode, FocusMode::PartitionEdit);

        // Exit edit mode with Escape
        screen.handle_input(KeyCode::Esc);
        assert_eq!(screen.focus_mode, FocusMode::PartitionList);
        assert_eq!(screen.selected_field, None);
    }

    #[test]
    fn test_filesystem_cycling() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Enter edit mode on root partition
        screen.selected_partition_idx = 1;
        screen.handle_input(KeyCode::Enter);

        // Default filesystem is ext4
        assert_eq!(screen.partitions[1].fstype, "ext4");

        // Cycle right to btrfs
        screen.handle_input(KeyCode::Right);
        assert_eq!(screen.partitions[1].fstype, "btrfs");

        // Cycle right to xfs
        screen.handle_input(KeyCode::Right);
        assert_eq!(screen.partitions[1].fstype, "xfs");

        // Cycle right to zfs
        screen.handle_input(KeyCode::Right);
        assert_eq!(screen.partitions[1].fstype, "zfs");

        // Cycle right wraps back to ext4
        screen.handle_input(KeyCode::Right);
        assert_eq!(screen.partitions[1].fstype, "ext4");

        // Cycle left to zfs
        screen.handle_input(KeyCode::Left);
        assert_eq!(screen.partitions[1].fstype, "zfs");
    }

    #[test]
    fn test_get_editable_fields() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // ESP (index 0) - no editable fields
        assert_eq!(screen.get_editable_fields(0).len(), 0);

        // Root (index 1) - filesystem type editable
        let root_fields = screen.get_editable_fields(1);
        assert_eq!(root_fields.len(), 1);
        assert_eq!(root_fields[0], EditableField::FilesystemType);

        // Swap (index 2) - size editable
        let swap_fields = screen.get_editable_fields(2);
        assert_eq!(swap_fields.len(), 1);
        assert_eq!(swap_fields[0], EditableField::Size);
    }

    #[test]
    fn test_can_edit_partition() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // ESP cannot be edited
        assert!(!screen.can_edit_partition(0));

        // Root can be edited
        assert!(screen.can_edit_partition(1));

        // Swap can be edited
        assert!(screen.can_edit_partition(2));
    }

    #[test]
    fn test_right_arrow_continues_in_list_mode() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // In partition list mode, Right arrow should continue to next screen
        let action = screen.handle_input(KeyCode::Right);
        assert_eq!(action, ScreenAction::Next);
    }

    #[test]
    fn test_tab_quick_access_to_swap() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Tab should focus swap input
        screen.handle_input(KeyCode::Tab);
        assert!(screen.swap_focused);
    }

    #[test]
    fn test_filesystem_persistence() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Enter edit mode and change filesystem
        screen.selected_partition_idx = 1;
        screen.handle_input(KeyCode::Enter);
        screen.handle_input(KeyCode::Right); // Change to btrfs
        screen.handle_input(KeyCode::Esc); // Exit edit mode

        // Filesystem change should persist
        assert_eq!(screen.partitions[1].fstype, "btrfs");
        assert_eq!(screen.focus_mode, FocusMode::PartitionList);
    }

    #[test]
    fn test_bios_mode_no_esp() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Bios, "/dev/sda".to_string(), 500_000_000_000);

        // BIOS mode should have only 2 partitions
        assert_eq!(screen.partitions.len(), 2);

        // Root should be at index 0
        assert_eq!(screen.partitions[0].mountpoint, "/");
        assert!(screen.can_edit_partition(0));

        // Swap should be at index 1
        assert_eq!(screen.partitions[1].fstype, "swap");
        assert!(screen.can_edit_partition(1));
    }

    #[test]
    fn test_enter_confirms_filesystem_selection() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Enter edit mode on root partition
        screen.selected_partition_idx = 1;
        screen.handle_input(KeyCode::Enter);
        assert_eq!(screen.focus_mode, FocusMode::PartitionEdit);
        assert_eq!(screen.selected_field, Some(EditableField::FilesystemType));

        // Change filesystem to btrfs
        screen.handle_input(KeyCode::Right);
        assert_eq!(screen.partitions[1].fstype, "btrfs");

        // Press Enter to confirm and exit edit mode
        screen.handle_input(KeyCode::Enter);
        assert_eq!(screen.focus_mode, FocusMode::PartitionList);
        assert_eq!(screen.selected_field, None);

        // Filesystem change should be persisted
        assert_eq!(screen.partitions[1].fstype, "btrfs");
    }
}
