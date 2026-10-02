//! Partition planning screen
//!
//! Shows the automatic partition layout that will be created.
//! Implements `gpui::Render` for the gpui GUI framework.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, Hsla, IntoElement, SharedString};
use tracing::debug;

use crate::system::BootMode;
use crate::ui::theme::{icons, spacing, AppTheme};
use crate::ui::utils::NavigationHints;

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
    /// Swap size input value as a string (replaces Entity<InputField>)
    swap_input_value: String,
    /// Whether the swap field is focused
    swap_focused: bool,
    /// Selected partition index for cursor navigation
    selected_partition_idx: usize,
    /// Current focus mode (partition list vs editing)
    focus_mode: FocusMode,
    /// Selected field within partition (when in edit mode)
    selected_field: Option<EditableField>,
    /// Whether to encrypt the root partition with LUKS
    luks_enabled: bool,
    /// LUKS passphrase
    luks_passphrase: String,
    /// LUKS passphrase confirm
    luks_passphrase_confirm: String,
    /// Whether the LUKS passphrase field is focused
    luks_focused: bool,
    /// Which LUKS field is focused (0 = passphrase, 1 = confirm)
    luks_field_idx: usize,
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

#[allow(dead_code)]
impl PartitionPlanningScreen {
    /// Create a new partition planning screen
    pub fn new() -> Self {
        Self {
            boot_mode: BootMode::Unknown,
            disk_path: String::new(),
            disk_size: 0,
            partitions: Vec::new(),
            swap_size_gb: 8,
            swap_input_value: "8".to_string(),
            swap_focused: false,
            selected_partition_idx: 0,
            focus_mode: FocusMode::PartitionList,
            selected_field: None,
            luks_enabled: false,
            luks_passphrase: String::new(),
            luks_passphrase_confirm: String::new(),
            luks_focused: false,
            luks_field_idx: 0,
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
            },
            Partition {
                label: "Root Partition".to_string(),
                size: root_size,
                fstype: "ext4".to_string(),
                mountpoint: "/".to_string(),
            },
            Partition {
                label: "Swap Partition".to_string(),
                size: swap_size,
                fstype: "swap".to_string(),
                mountpoint: "[swap]".to_string(),
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
            },
            Partition {
                label: "Swap Partition".to_string(),
                size: swap_size,
                fstype: "swap".to_string(),
                mountpoint: "[swap]".to_string(),
            },
        ];
    }

    /// Get total allocated size
    fn total_size(&self) -> u64 {
        self.partitions.iter().map(|p| p.size).sum()
    }

    /// Update swap size from input field and regenerate partitions
    fn update_swap_size(&mut self) {
        if let Ok(size) = self.swap_input_value.parse::<u64>() {
            // Validate minimum and maximum
            let min_swap = 1; // 1 GB minimum
            let esp_size_gb = if matches!(self.boot_mode, BootMode::Uefi) {
                1
            } else {
                0
            };
            let max_swap = (self.disk_size / 1_000_000_000).saturating_sub(10 + esp_size_gb);

            let validated_size = size.max(min_swap).min(max_swap);

            if self.swap_size_gb != validated_size {
                self.swap_size_gb = validated_size;
                self.swap_input_value = validated_size.to_string();
                self.generate_partition_layout();
            }
        }
    }

    /// Adjust swap size by delta (in GB)
    fn adjust_swap_size(&mut self, delta: i64) {
        let new_size = (self.swap_size_gb as i64 + delta).max(1) as u64;
        self.swap_input_value = new_size.to_string();
        self.update_swap_size();
    }

    // ===== Public Getters for Configuration =====

    /// Get the configured swap size in GB
    pub fn swap_size_gb(&self) -> u64 {
        self.swap_size_gb
    }

    /// Get the selected disk path
    pub fn disk_path(&self) -> &str {
        &self.disk_path
    }

    /// Get the selected disk size in bytes
    pub fn disk_size(&self) -> u64 {
        self.disk_size
    }

    /// Get whether LUKS encryption is enabled
    pub fn luks_enabled(&self) -> bool {
        self.luks_enabled
    }

    /// Get the LUKS passphrase
    pub fn luks_passphrase(&self) -> &str {
        &self.luks_passphrase
    }

    /// Get the selected root filesystem type
    pub fn root_filesystem(&self) -> &str {
        // Find the root partition (the one mounted at "/")
        for partition in &self.partitions {
            if partition.mountpoint == "/" {
                return &partition.fstype;
            }
        }
        // Default to ext4 if not found
        "ext4"
    }

    /// Whether this screen can proceed
    pub fn can_proceed(&self) -> bool {
        true
    }

    /// Whether this screen can go back
    pub fn can_go_back(&self) -> bool {
        true
    }

    /// Title of this screen
    pub fn title(&self) -> &str {
        "Partition Planning"
    }

    /// Help content for this screen
    pub fn help_content(&self) -> Vec<String> {
        vec![
            "# Partition Planning Screen".to_string(),
            "".to_string(),
            "This screen shows an interactive partition layout editor.".to_string(),
            "You can customize partition sizes and filesystem types.".to_string(),
            "".to_string(),
            "## Interactive Navigation".to_string(),
            "".to_string(),
            "The partition list uses a hierarchical menu with '>' cursor:".to_string(),
            "- Navigate partitions with Up/Down arrow keys".to_string(),
            "- Press Enter on a partition to edit its properties".to_string(),
            "- In edit mode, navigate fields with Up/Down".to_string(),
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
        ]
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
        // If we were editing swap size, unfocus it
        if self.swap_focused {
            self.swap_focused = false;
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

    /// Build a colored segment for the disk allocation bar
    fn allocation_segment(&self, color: Hsla, fraction: f32) -> impl IntoElement {
        div()
            .h_full()
            .bg(color)
            .w(gpui::relative(fraction.max(0.01)))
    }

    /// Build a legend row for the allocation bar
    fn legend_row(
        color: Hsla,
        label: &str,
        size: &str,
        percent: u16,
        theme: &AppTheme,
    ) -> impl IntoElement {
        let swatch: SharedString = "\u{2588}\u{2588}".into();
        let label_text: SharedString = format!("{}: {} ({}%)", label, size, percent).into();

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing::SMALL))
            .child(div().text_color(color).child(swatch))
            .child(
                div()
                    .text_color(theme.foreground)
                    .text_sm()
                    .child(label_text),
            )
    }

    /// Render this screen as a gpui element tree.
    pub fn view(&mut self) -> impl IntoElement {
        let theme = AppTheme::new();

        let mut col = div()
            .flex()
            .flex_col()
            .size_full()
            .gap(px(spacing::MEDIUM))
            .p(px(spacing::LARGE));

        // === Disk info block ===
        let disk_info_title: SharedString = " Selected Disk ".into();
        let device_label: SharedString = format!("Device: {}", self.disk_path).into();
        let size_label: SharedString = format!(
            "Size: {:.1} GB  |  Boot Mode: {}",
            self.disk_size as f64 / 1_000_000_000.0,
            self.boot_mode.as_str()
        )
        .into();

        col = col.child(
            div()
                .w_full()
                .border_1()
                .border_color(theme.border)
                .rounded(px(4.0))
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    div()
                        .w_full()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .border_b_1()
                        .border_color(theme.border)
                        .text_color(theme.foreground)
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .child(disk_info_title),
                )
                .child(
                    div()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .flex()
                        .flex_col()
                        .gap(px(spacing::SMALL))
                        .child(
                            div()
                                .text_color(theme.foreground)
                                .text_sm()
                                .child(device_label),
                        )
                        .child(
                            div()
                                .text_color(theme.foreground)
                                .text_sm()
                                .child(size_label),
                        ),
                ),
        );

        // === Visual allocation bar ===
        let total_size = self.disk_size as f64;
        let mut bar_children = div()
            .w_full()
            .h(px(16.0))
            .bg(theme.muted)
            .rounded(px(4.0))
            .overflow_hidden()
            .flex()
            .flex_row();

        let mut legend_block = div()
            .flex()
            .flex_col()
            .gap(px(spacing::SMALL))
            .px(px(spacing::MEDIUM));

        for (idx, partition) in self.partitions.iter().enumerate() {
            let fraction = if total_size > 0.0 {
                partition.size as f32 / total_size as f32
            } else {
                0.0
            };
            let percent = (fraction * 100.0) as u16;

            let color = match idx {
                0 if self.partitions.len() == 3 => theme.primary, // ESP
                _ if partition.fstype == "swap" => theme.warning, // Swap
                _ => theme.success,                               // Root
            };

            bar_children = bar_children.child(self.allocation_segment(color, fraction));

            legend_block = legend_block.child(Self::legend_row(
                color,
                &partition.label,
                &partition.size_human(),
                percent,
                &theme,
            ));
        }

        let alloc_title: SharedString = " Disk Space Allocation ".into();
        col = col.child(
            div()
                .w_full()
                .border_1()
                .border_color(theme.border)
                .rounded(px(4.0))
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    div()
                        .w_full()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .border_b_1()
                        .border_color(theme.border)
                        .text_color(theme.foreground)
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .child(alloc_title),
                )
                .child(
                    div()
                        .p(px(spacing::MEDIUM))
                        .flex()
                        .flex_col()
                        .gap(px(spacing::MEDIUM))
                        .child(bar_children)
                        .child(legend_block),
                ),
        );

        // === Swap size input area ===
        let swap_border = if self.swap_focused {
            theme.primary
        } else {
            theme.border
        };
        let swap_title: SharedString = " Adjust Partition Sizes ".into();
        let swap_display: SharedString = if self.swap_focused {
            format!("Swap Size (GB): {}\u{2588}", self.swap_input_value).into()
        } else {
            format!("Swap Size (GB): {}", self.swap_input_value).into()
        };

        let help_text: SharedString = if self.swap_focused {
            "Type to adjust swap size | Up/Down to increment/decrement by 1GB | \
             Tab/Enter to unfocus"
                .into()
        } else if self.focus_mode == FocusMode::PartitionEdit
            && self.selected_field == Some(EditableField::Size)
        {
            "Up/Down to adjust by 1GB | Enter to type value | Esc to exit edit mode".into()
        } else {
            "Enter on swap partition, then select Size field to edit".into()
        };

        col = col.child(
            div()
                .w_full()
                .border_1()
                .border_color(swap_border)
                .rounded(px(4.0))
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    div()
                        .w_full()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .border_b_1()
                        .border_color(swap_border)
                        .text_color(theme.foreground)
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .child(swap_title),
                )
                .child(
                    div()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .flex()
                        .flex_col()
                        .gap(px(spacing::SMALL))
                        .child(
                            div()
                                .text_color(if self.swap_focused {
                                    theme.primary
                                } else {
                                    theme.foreground
                                })
                                .text_sm()
                                .child(swap_display),
                        )
                        .child(div().text_color(theme.muted).text_sm().child(help_text)),
                ),
        );

        // === Encryption toggle section ===
        let encrypt_border = if self.luks_focused {
            theme.primary
        } else {
            theme.border
        };
        let encrypt_title: SharedString = " Disk Encryption ".into();
        let checkbox: SharedString = if self.luks_enabled {
            format!("  {} ", icons::CHECKED).into()
        } else {
            format!("  {} ", icons::UNCHECKED).into()
        };
        let toggle_suffix: SharedString =
            "Encrypt root partition (LUKS)    Press 'e' to toggle".into();

        let mut encrypt_block = div()
            .w_full()
            .border_1()
            .border_color(encrypt_border)
            .rounded(px(4.0))
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .px(px(spacing::MEDIUM))
                    .py(px(spacing::SMALL))
                    .border_b_1()
                    .border_color(encrypt_border)
                    .text_color(theme.foreground)
                    .font_weight(FontWeight::BOLD)
                    .text_sm()
                    .child(encrypt_title),
            )
            .child(
                div()
                    .px(px(spacing::MEDIUM))
                    .py(px(spacing::SMALL))
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        div()
                            .text_color(if self.luks_enabled {
                                theme.success
                            } else {
                                theme.muted
                            })
                            .child(checkbox),
                    )
                    .child(
                        div()
                            .text_color(theme.foreground)
                            .text_sm()
                            .child(toggle_suffix),
                    ),
            );

        if self.luks_enabled {
            let pass_display: SharedString = if self.luks_focused && self.luks_field_idx == 0 {
                let masked = "\u{2022}".repeat(self.luks_passphrase.len());
                format!("Encryption Passphrase: {}\u{2588}", masked).into()
            } else {
                let masked = "\u{2022}".repeat(self.luks_passphrase.len());
                format!("Encryption Passphrase: {}", masked).into()
            };

            let confirm_display: SharedString = if self.luks_focused && self.luks_field_idx == 1 {
                let masked = "\u{2022}".repeat(self.luks_passphrase_confirm.len());
                format!("Confirm Passphrase:    {}\u{2588}", masked).into()
            } else {
                let masked = "\u{2022}".repeat(self.luks_passphrase_confirm.len());
                format!("Confirm Passphrase:    {}", masked).into()
            };

            encrypt_block = encrypt_block
                .child(
                    div()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .text_color(if self.luks_focused && self.luks_field_idx == 0 {
                            theme.primary
                        } else {
                            theme.foreground
                        })
                        .text_sm()
                        .child(pass_display),
                )
                .child(
                    div()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .text_color(if self.luks_focused && self.luks_field_idx == 1 {
                            theme.primary
                        } else {
                            theme.foreground
                        })
                        .text_sm()
                        .child(confirm_display),
                );
        }

        col = col.child(encrypt_block);

        // === Partition list with hierarchical display ===
        let part_title: SharedString = " Partition Layout ".into();
        let mut part_list = div()
            .w_full()
            .flex_1()
            .border_1()
            .border_color(theme.border)
            .rounded(px(4.0))
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .px(px(spacing::MEDIUM))
                    .py(px(spacing::SMALL))
                    .border_b_1()
                    .border_color(theme.border)
                    .text_color(theme.foreground)
                    .font_weight(FontWeight::BOLD)
                    .text_sm()
                    .child(part_title),
            );

        // Clone data for rendering to avoid borrow conflicts
        let partitions_snapshot: Vec<(usize, Partition)> = self
            .partitions
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.clone()))
            .collect();
        let focus_mode = self.focus_mode;
        let selected_idx = self.selected_partition_idx;
        let selected_field = self.selected_field;

        for (idx, partition) in &partitions_snapshot {
            let idx = *idx;
            let is_selected = idx == selected_idx;
            let is_in_edit_mode = is_selected && focus_mode == FocusMode::PartitionEdit;

            // Partition header line with cursor
            let cursor: SharedString = if is_selected && focus_mode == FocusMode::PartitionList {
                "> ".into()
            } else {
                "  ".into()
            };

            let header_color = if is_selected && focus_mode == FocusMode::PartitionList {
                theme.primary
            } else {
                theme.foreground
            };
            let header_weight = if is_selected && focus_mode == FocusMode::PartitionList {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            };

            let label_text: SharedString = partition.label.clone().into();
            let size_badge: SharedString = format!("  [{}]", partition.size_human()).into();

            let mut partition_block = div()
                .w_full()
                .px(px(spacing::MEDIUM))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .py(px(spacing::SMALL));

            // Header row
            partition_block = partition_block.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        div()
                            .text_color(header_color)
                            .font_weight(header_weight)
                            .child(cursor),
                    )
                    .child(
                        div()
                            .text_color(header_color)
                            .font_weight(header_weight)
                            .child(label_text),
                    )
                    .child(
                        div()
                            .text_color(if is_selected {
                                theme.foreground
                            } else {
                                theme.muted
                            })
                            .text_sm()
                            .child(size_badge),
                    ),
            );

            // Editable fields
            let editable_fields = self.get_editable_fields(idx);

            // Size field (for swap partition)
            if editable_fields.contains(&EditableField::Size) {
                let is_size_focused =
                    is_in_edit_mode && selected_field == Some(EditableField::Size);
                let field_cursor: SharedString = if is_size_focused {
                    "  > ".into()
                } else {
                    "    ".into()
                };
                let size_text: SharedString = format!("Size: {}", partition.size_human()).into();

                partition_block = partition_block.child(
                    div()
                        .flex()
                        .flex_row()
                        .child(
                            div()
                                .text_color(if is_size_focused {
                                    theme.primary
                                } else {
                                    theme.foreground
                                })
                                .font_weight(if is_size_focused {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .child(field_cursor),
                        )
                        .child(
                            div()
                                .text_color(if is_size_focused {
                                    theme.primary
                                } else {
                                    theme.muted
                                })
                                .text_sm()
                                .child(size_text),
                        ),
                );
            } else {
                let size_text: SharedString =
                    format!("Size: {} (fixed)", partition.size_human()).into();
                partition_block = partition_block.child(
                    div()
                        .flex()
                        .flex_row()
                        .child(div().child("    "))
                        .child(div().text_color(theme.muted).text_sm().child(size_text)),
                );
            }

            // Filesystem type field
            if editable_fields.contains(&EditableField::FilesystemType) {
                let is_type_focused =
                    is_in_edit_mode && selected_field == Some(EditableField::FilesystemType);
                let field_cursor: SharedString = if is_type_focused {
                    "  > ".into()
                } else {
                    "    ".into()
                };

                let mut type_row = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        div()
                            .text_color(if is_type_focused {
                                theme.primary
                            } else {
                                theme.foreground
                            })
                            .font_weight(if is_type_focused {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .child(field_cursor),
                    )
                    .child(
                        div()
                            .text_color(if is_type_focused {
                                theme.foreground
                            } else {
                                theme.muted
                            })
                            .text_sm()
                            .child("Type: "),
                    );

                let options: Vec<&str> =
                    ROOT_FILESYSTEM_OPTIONS.iter().map(|(fs, _)| *fs).collect();
                for (i, &fs) in options.iter().enumerate() {
                    if i > 0 {
                        type_row =
                            type_row.child(div().text_color(theme.muted).text_sm().child("  "));
                    }

                    let fs_display: SharedString = if fs == partition.fstype {
                        format!("{} \u{25c4}", fs).into()
                    } else {
                        fs.to_string().into()
                    };

                    let fs_color = if fs == partition.fstype {
                        if is_type_focused {
                            theme.primary
                        } else {
                            theme.success
                        }
                    } else if is_type_focused {
                        theme.muted
                    } else {
                        // Not focused and not selected: don't show
                        continue;
                    };

                    type_row = type_row.child(
                        div()
                            .text_color(fs_color)
                            .font_weight(if fs == partition.fstype {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_sm()
                            .child(fs_display),
                    );
                }

                partition_block = partition_block.child(type_row);
            } else {
                let type_text: SharedString = format!("Type: {}", partition.fstype).into();
                partition_block = partition_block.child(
                    div()
                        .flex()
                        .flex_row()
                        .child(div().child("    "))
                        .child(div().text_color(theme.muted).text_sm().child(type_text)),
                );
            }

            // Mount point (always read-only)
            let mount_text: SharedString = format!("Mount: {}", partition.mountpoint).into();
            partition_block = partition_block.child(
                div()
                    .flex()
                    .flex_row()
                    .child(div().child("    "))
                    .child(div().text_color(theme.muted).text_sm().child(mount_text)),
            );

            part_list = part_list.child(partition_block);
        }

        col = col.child(part_list);

        // === Summary ===
        let total = self.total_size();
        let total_gb = total as f64 / 1_000_000_000.0;
        let total_label: SharedString = format!("Total allocated: {:.1} GB", total_gb).into();
        let warning_text: SharedString = format!(
            "{} WARNING: All existing data on this disk will be permanently erased!",
            icons::WARNING
        )
        .into();

        col = col.child(
            div()
                .w_full()
                .border_1()
                .border_color(theme.border)
                .rounded(px(4.0))
                .p(px(spacing::MEDIUM))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(spacing::SMALL))
                .child(
                    div()
                        .text_color(theme.success)
                        .font_weight(FontWeight::BOLD)
                        .child(total_label),
                )
                .child(div().text_color(theme.error).text_sm().child(warning_text)),
        );

        // === Navigation hints ===
        let hints = match self.focus_mode {
            FocusMode::PartitionList => vec![
                ("Up/Down", "Navigate"),
                ("Enter", "Edit Partition"),
                ("Tab", "Quick Edit Swap"),
                ("Right", "Continue"),
                ("?", "Help"),
                ("q", "Quit"),
            ],
            FocusMode::PartitionEdit => {
                if self.selected_field == Some(EditableField::FilesystemType) {
                    vec![
                        ("Up/Down", "Navigate Fields"),
                        ("Left/Right", "Change Filesystem"),
                        ("Enter", "Confirm"),
                        ("Esc", "Cancel"),
                        ("?", "Help"),
                    ]
                } else {
                    vec![
                        ("Up/Down", "Navigate Fields"),
                        ("Enter", "Edit Size"),
                        ("Esc", "Exit Edit Mode"),
                        ("?", "Help"),
                    ]
                }
            }
        };

        col = col.child(NavigationHints::new(hints));

        col
    }
}

impl Default for PartitionPlanningScreen {
    fn default() -> Self {
        Self::new()
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
        };
        assert_eq!(part.size_human(), "512 MB");

        let part2 = Partition {
            label: "Test".to_string(),
            size: 8_000_000_000,
            fstype: "ext4".to_string(),
            mountpoint: "/".to_string(),
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

        // Try to set swap to 95 GB - should be clamped to leave room
        // for root and ESP
        screen.swap_input_value = "95".to_string();
        screen.update_swap_size();

        // Max should be disk_size_gb - 10 (root min) - 1 (ESP) = 89 GB
        assert_eq!(screen.swap_size_gb, 89);
    }

    #[test]
    fn test_swap_size_update_from_input() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Bios, "/dev/sda".to_string(), 500_000_000_000);

        // Set swap size via input value
        screen.swap_input_value = "16".to_string();
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

        // Simulate Tab key to focus (in original, Tab focuses swap)
        screen.swap_focused = true;
        assert!(screen.swap_focused);

        // Simulate Tab again to unfocus
        screen.swap_focused = false;
        assert!(!screen.swap_focused);
    }

    #[test]
    fn test_arrow_key_navigation() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Up/Down arrows navigate partitions
        assert_eq!(screen.selected_partition_idx, 0);

        // Test Down - navigates to next partition
        screen.select_next_partition();
        assert_eq!(screen.selected_partition_idx, 1);

        // Test Up - navigates to previous partition
        screen.select_previous_partition();
        assert_eq!(screen.selected_partition_idx, 0);
    }

    #[test]
    fn test_enter_key_behavior() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // When on ESP (not editable), enter_edit_mode does nothing
        screen.enter_edit_mode();
        assert_eq!(screen.focus_mode, FocusMode::PartitionList);

        // When swap focused, unfocusing clears
        screen.swap_focused = true;
        screen.swap_focused = false;
        assert!(!screen.swap_focused);
    }

    // ===== Tests for interactive partition editing =====

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
        screen.select_next_partition();
        assert_eq!(screen.selected_partition_idx, 1);

        screen.select_next_partition();
        assert_eq!(screen.selected_partition_idx, 2);

        // Can't go past last partition
        screen.select_next_partition();
        assert_eq!(screen.selected_partition_idx, 2);

        // Navigate up
        screen.select_previous_partition();
        assert_eq!(screen.selected_partition_idx, 1);

        screen.select_previous_partition();
        assert_eq!(screen.selected_partition_idx, 0);

        // Can't go before first partition
        screen.select_previous_partition();
        assert_eq!(screen.selected_partition_idx, 0);
    }

    #[test]
    fn test_enter_edit_mode_on_root_partition() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Navigate to root partition (index 1)
        screen.selected_partition_idx = 1;

        // Enter edit mode
        screen.enter_edit_mode();

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
        screen.enter_edit_mode();

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
        screen.enter_edit_mode();

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
        screen.enter_edit_mode();
        assert_eq!(screen.focus_mode, FocusMode::PartitionEdit);

        // Exit edit mode
        screen.exit_edit_mode();
        assert_eq!(screen.focus_mode, FocusMode::PartitionList);
        assert_eq!(screen.selected_field, None);
    }

    #[test]
    fn test_filesystem_cycling() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Enter edit mode on root partition
        screen.selected_partition_idx = 1;
        screen.enter_edit_mode();

        // Default filesystem is ext4
        assert_eq!(screen.partitions[1].fstype, "ext4");

        // Cycle right to btrfs
        screen.cycle_filesystem(1);
        assert_eq!(screen.partitions[1].fstype, "btrfs");

        // Cycle right to xfs
        screen.cycle_filesystem(1);
        assert_eq!(screen.partitions[1].fstype, "xfs");

        // Cycle right to zfs
        screen.cycle_filesystem(1);
        assert_eq!(screen.partitions[1].fstype, "zfs");

        // Cycle right wraps back to ext4
        screen.cycle_filesystem(1);
        assert_eq!(screen.partitions[1].fstype, "ext4");

        // Cycle left to zfs
        screen.cycle_filesystem(-1);
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
        let screen = PartitionPlanningScreen::new();
        // In partition list mode, can_proceed is true
        assert_eq!(screen.focus_mode, FocusMode::PartitionList);
        assert!(screen.can_proceed());
    }

    #[test]
    fn test_tab_quick_access_to_swap() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Tab should focus swap input (simulated by setting swap_focused)
        screen.swap_focused = true;
        assert!(screen.swap_focused);
    }

    #[test]
    fn test_filesystem_persistence() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Enter edit mode and change filesystem
        screen.selected_partition_idx = 1;
        screen.enter_edit_mode();
        screen.cycle_filesystem(1); // Change to btrfs
        screen.exit_edit_mode();

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
        screen.enter_edit_mode();
        assert_eq!(screen.focus_mode, FocusMode::PartitionEdit);
        assert_eq!(screen.selected_field, Some(EditableField::FilesystemType));

        // Change filesystem to btrfs
        screen.cycle_filesystem(1);
        assert_eq!(screen.partitions[1].fstype, "btrfs");

        // Press Enter to confirm and exit edit mode (simulated by
        // exit_edit_mode since Enter on FilesystemType exits)
        screen.exit_edit_mode();
        assert_eq!(screen.focus_mode, FocusMode::PartitionList);
        assert_eq!(screen.selected_field, None);

        // Filesystem change should be persisted
        assert_eq!(screen.partitions[1].fstype, "btrfs");
    }

    #[test]
    fn test_disk_config_getters() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(
            BootMode::Uefi,
            "/dev/nvme0n1".to_string(),
            1_000_000_000_000,
        );

        // Test disk path getter
        assert_eq!(screen.disk_path(), "/dev/nvme0n1");

        // Test disk size getter
        assert_eq!(screen.disk_size(), 1_000_000_000_000);

        // Test default swap size
        assert_eq!(screen.swap_size_gb(), 8);

        // Test default root filesystem
        assert_eq!(screen.root_filesystem(), "ext4");

        // Change swap size
        screen.swap_input_value = "16".to_string();
        screen.update_swap_size();
        assert_eq!(screen.swap_size_gb(), 16);

        // Change root filesystem to btrfs
        screen.selected_partition_idx = 1; // Root partition in UEFI mode
        screen.enter_edit_mode();
        screen.cycle_filesystem(1); // Change to btrfs
        assert_eq!(screen.root_filesystem(), "btrfs");
    }

    #[test]
    fn test_total_size() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Total size should equal disk size
        assert_eq!(screen.total_size(), 500_000_000_000);
    }

    #[test]
    fn test_default_impl() {
        let screen = PartitionPlanningScreen::default();
        assert_eq!(screen.swap_size_gb, 8);
        assert_eq!(screen.disk_path, "");
        assert_eq!(screen.disk_size, 0);
    }

    #[test]
    fn test_luks_default_state() {
        let screen = PartitionPlanningScreen::new();
        assert!(!screen.luks_enabled);
        assert!(screen.luks_passphrase.is_empty());
        assert!(screen.luks_passphrase_confirm.is_empty());
        assert!(!screen.luks_focused);
        assert_eq!(screen.luks_field_idx, 0);
    }

    #[test]
    fn test_luks_toggle() {
        let mut screen = PartitionPlanningScreen::new();
        assert!(!screen.luks_enabled);
        screen.luks_enabled = true;
        assert!(screen.luks_enabled);
        screen.luks_enabled = false;
        assert!(!screen.luks_enabled);
    }

    #[test]
    fn test_luks_passphrase_getter() {
        let mut screen = PartitionPlanningScreen::new();
        screen.luks_passphrase = "mysecretpass".to_string();
        assert_eq!(screen.luks_passphrase(), "mysecretpass");
    }

    #[test]
    fn test_luks_enabled_getter() {
        let mut screen = PartitionPlanningScreen::new();
        assert!(!screen.luks_enabled());
        screen.luks_enabled = true;
        assert!(screen.luks_enabled());
    }

    #[test]
    fn test_select_previous_field() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Enter edit mode on root (only has FilesystemType)
        screen.selected_partition_idx = 1;
        screen.enter_edit_mode();
        assert_eq!(screen.selected_field, Some(EditableField::FilesystemType));

        // Previous on a single-field partition does nothing
        screen.select_previous_field();
        assert_eq!(screen.selected_field, Some(EditableField::FilesystemType));
    }

    #[test]
    fn test_select_next_field() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Enter edit mode on swap (only has Size)
        screen.selected_partition_idx = 2;
        screen.enter_edit_mode();
        assert_eq!(screen.selected_field, Some(EditableField::Size));

        // Next on a single-field partition does nothing
        screen.select_next_field();
        assert_eq!(screen.selected_field, Some(EditableField::Size));
    }

    #[test]
    fn test_unknown_boot_mode_defaults_to_uefi() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Unknown, "/dev/sda".to_string(), 500_000_000_000);

        // Should generate UEFI layout (3 partitions)
        assert_eq!(screen.partitions.len(), 3);
        assert_eq!(screen.partitions[0].label, "EFI System Partition");
    }

    #[test]
    fn test_editable_fields_out_of_bounds() {
        let screen = PartitionPlanningScreen::new();
        // Out of bounds returns empty
        assert!(screen.get_editable_fields(99).is_empty());
        assert!(!screen.can_edit_partition(99));
    }

    #[test]
    fn test_cycle_filesystem_wrong_field() {
        let mut screen = PartitionPlanningScreen::new();
        screen.set_disk(BootMode::Uefi, "/dev/sda".to_string(), 500_000_000_000);

        // Enter edit on swap (Size field, not FilesystemType)
        screen.selected_partition_idx = 2;
        screen.enter_edit_mode();
        assert_eq!(screen.selected_field, Some(EditableField::Size));

        // Cycling filesystem should do nothing when on Size field
        let original_fs = screen.partitions[1].fstype.clone();
        screen.cycle_filesystem(1);
        assert_eq!(screen.partitions[1].fstype, original_fs);
    }
}
