//! Disk selection screen
//!
//! Allows the user to select which disk to install NixOS on.
//! Implements `gpui::Render` for the gpui GUI framework.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement, SharedString};
use tracing::{debug, warn};

use crate::nixos::{detect_disks, Disk};
use crate::ui::theme::{icons, spacing, AppTheme};
use crate::ui::utils::NavigationHints;

/// Disk selection screen state
pub struct DiskSelectionScreen {
    /// Available disks
    disks: Vec<Disk>,
    /// Currently selected disk index
    selected_index: usize,
    /// Whether disk detection has been run
    detected: bool,
    /// Error message if detection failed
    error_message: Option<String>,
}

#[allow(dead_code)]
impl DiskSelectionScreen {
    /// Create a new disk selection screen
    pub fn new() -> Self {
        Self {
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
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_index < self.disks.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Check if current selection is installable
    pub fn is_selection_installable(&self) -> bool {
        self.selected_disk()
            .map(|d| d.is_installable())
            .unwrap_or(false)
    }

    /// Get warning message for selected disk (if any)
    pub fn get_disk_warning(&self) -> Option<String> {
        let disk = self.selected_disk()?;

        if disk.readonly {
            return Some("This disk is read-only and cannot be used for installation".to_string());
        }

        if disk.removable {
            return Some("This appears to be a removable disk (USB/SD card)".to_string());
        }

        if disk.mountpoint.is_some() {
            return Some("This disk has mounted partitions".to_string());
        }

        if disk.size < 8_000_000_000 {
            return Some(format!(
                "This disk is too small ({:.1} GB). Minimum 8 GB required.",
                disk.size_gb()
            ));
        }

        None
    }

    /// Run disk detection (called by root view on screen entry)
    pub fn detect(&mut self) {
        if self.detected {
            return;
        }

        debug!("Detecting available disks");

        match detect_disks() {
            Ok(disks) => {
                debug!("Found {} disks", disks.len());
                self.disks = disks;
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

    /// Get help content lines for the help panel
    pub fn help_content(&self) -> Vec<String> {
        vec![
            "# Disk Selection Screen".to_string(),
            "".to_string(),
            "Select the disk where NixOS will be installed.".to_string(),
            "".to_string(),
            "## Disk Status Indicators".to_string(),
            "".to_string(),
            "- Green checkmark: Disk is suitable for installation".to_string(),
            "- Red cross: Disk cannot be used (too small, read-only, or mounted)".to_string(),
            "- Arrow indicates currently selected disk".to_string(),
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
            "- Up/Down: Navigate disk list".to_string(),
            "- Enter: Select disk and continue (only if disk is suitable)".to_string(),
            "- Left / Backspace: Go back".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q / Esc: Quit the installer".to_string(),
        ]
    }

    /// Build a single disk row element
    fn disk_row(&self, idx: usize, disk: &Disk, theme: &AppTheme) -> impl IntoElement {
        let is_selected = idx == self.selected_index;
        let is_installable = disk.is_installable();

        // Status icon
        let status_icon: SharedString = if is_installable {
            icons::SUCCESS.into()
        } else {
            icons::ERROR.into()
        };
        let status_color = if is_installable {
            theme.success
        } else {
            theme.error
        };

        // Selection arrow
        let prefix: SharedString = if is_selected {
            format!("{} ", icons::SELECTION).into()
        } else {
            "  ".into()
        };

        // Name and type badge
        let display_name: SharedString = disk.display_name().into();
        let type_badge: SharedString = format!("[{}]", disk.disk_type.as_str()).into();

        // Text color
        let text_color = if is_selected {
            if is_installable {
                theme.primary
            } else {
                theme.error
            }
        } else if is_installable {
            theme.foreground
        } else {
            theme.muted
        };

        let text_weight = if is_selected {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        };

        let mut col = div().w_full().flex().flex_col();

        // Main row
        col = col.child(
            div()
                .w_full()
                .px(px(spacing::MEDIUM))
                .py(px(spacing::SMALL))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(spacing::SMALL))
                .child(
                    div()
                        .text_color(if is_selected {
                            theme.primary
                        } else {
                            theme.foreground
                        })
                        .font_weight(FontWeight::BOLD)
                        .child(prefix),
                )
                .child(
                    div()
                        .text_color(status_color)
                        .font_weight(FontWeight::BOLD)
                        .child(status_icon),
                )
                .child(
                    div()
                        .text_color(text_color)
                        .font_weight(text_weight)
                        .child(display_name),
                )
                .child(div().text_color(theme.muted).text_sm().child(type_badge)),
        );

        // Show partitions when selected
        if is_selected && !disk.partitions.is_empty() {
            for part in &disk.partitions {
                let fs_str = part.fstype.as_deref().unwrap_or("unknown");
                let label_str = part
                    .label
                    .as_ref()
                    .map(|l| format!(" \"{}\"", l))
                    .unwrap_or_default();
                let part_text: SharedString = format!(
                    "      {}  {}  {}{}",
                    part.name, part.size_human, fs_str, label_str
                )
                .into();

                col = col.child(
                    div()
                        .px(px(spacing::MEDIUM))
                        .text_color(theme.muted)
                        .text_sm()
                        .child(part_text),
                );
            }
        }

        col
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

        // Instructions
        col = col.child(div().w_full().flex().justify_center().child(
            div().text_color(theme.foreground).text_sm().child(
                "Select a disk for NixOS installation. \
                     All data on the selected disk will be erased.",
            ),
        ));

        // Disk list area
        if let Some(ref error) = self.error_message {
            // Error state
            let error_text: SharedString = error.clone().into();
            col = col.child(
                div()
                    .w_full()
                    .flex_1()
                    .border_1()
                    .border_color(theme.error)
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
                            .border_color(theme.error)
                            .text_color(theme.error)
                            .font_weight(FontWeight::BOLD)
                            .text_sm()
                            .child(" Error "),
                    )
                    .child(
                        div()
                            .p(px(spacing::MEDIUM))
                            .text_color(theme.error)
                            .child(error_text),
                    ),
            );
        } else if self.disks.is_empty() {
            // No disks found
            col = col.child(
                div()
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
                            .child(" Available Disks "),
                    )
                    .child(
                        div()
                            .p(px(spacing::MEDIUM))
                            .text_color(theme.muted)
                            .child("No suitable disks found for installation."),
                    ),
            );
        } else {
            // Disk list
            let title: SharedString = format!(" Available Disks ({}) ", self.disks.len()).into();

            // Clone data needed for rendering to avoid borrow conflicts
            let disk_data: Vec<(usize, Disk)> = self
                .disks
                .iter()
                .enumerate()
                .map(|(i, d)| (i, d.clone()))
                .collect();

            let mut list_container = div()
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
                        .child(title),
                );

            for (idx, disk) in &disk_data {
                list_container = list_container.child(self.disk_row(*idx, disk, &theme));
            }

            col = col.child(list_container);
        }

        // Warning / info area
        let info_content = if let Some(warning) = self.get_disk_warning() {
            let warning_icon: SharedString = icons::WARNING.into();
            let warning_text: SharedString = warning.into();
            div()
                .w_full()
                .border_1()
                .border_color(theme.warning)
                .rounded(px(4.0))
                .p(px(spacing::MEDIUM))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(spacing::SMALL))
                .child(
                    div()
                        .text_color(theme.warning)
                        .font_weight(FontWeight::BOLD)
                        .child(warning_icon),
                )
                .child(div().text_color(theme.warning).child(warning_text))
        } else if let Some(disk) = self.selected_disk() {
            let selected_path: SharedString = disk.path.clone().into();
            let selected_size: SharedString = disk.size_human.clone().into();
            div()
                .w_full()
                .border_1()
                .border_color(theme.border)
                .rounded(px(4.0))
                .p(px(spacing::MEDIUM))
                .flex()
                .flex_col()
                .gap(px(spacing::SMALL))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(spacing::SMALL))
                        .child(div().text_color(theme.muted).child("Selected:"))
                        .child(div().text_color(theme.success).child(selected_path))
                        .child(div().text_color(theme.muted).child("-"))
                        .child(div().text_color(theme.foreground).child(selected_size)),
                )
                .child(div().text_color(theme.error).text_sm().child(
                    "WARNING: All data on this disk will be \
                         permanently erased!",
                ))
        } else {
            div()
        };

        col = col.child(info_content);

        // Navigation hints
        let hints = if self.is_selection_installable() {
            vec![
                ("Up/Down", "Select"),
                ("Enter", "Continue"),
                ("Left", "Back"),
                ("?", "Help"),
                ("q", "Quit"),
            ]
        } else {
            vec![
                ("Up/Down", "Select"),
                ("Left", "Back"),
                ("?", "Help"),
                ("q", "Quit"),
            ]
        };
        col = col.child(NavigationHints::new(hints));

        col
    }
}

impl Default for DiskSelectionScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nixos::{DiskType, PartitionInfo};

    fn make_test_disk(name: &str, size: u64, installable: bool) -> Disk {
        Disk {
            name: name.to_string(),
            path: format!("/dev/{}", name),
            size,
            size_human: format!("{:.1} GB", size as f64 / 1_000_000_000.0),
            disk_type: if installable {
                DiskType::Disk
            } else {
                DiskType::Loop
            },
            model: Some("Test Disk".to_string()),
            removable: false,
            readonly: false,
            mountpoint: None,
            partitions: Vec::new(),
        }
    }

    #[test]
    fn test_disk_selection_creation() {
        let screen = DiskSelectionScreen::new();
        assert!(screen.disks.is_empty());
        assert_eq!(screen.selected_index, 0);
        assert!(!screen.detected);
        assert!(screen.error_message.is_none());
    }

    #[test]
    fn test_disk_selection_default() {
        let screen = DiskSelectionScreen::default();
        assert!(screen.disks.is_empty());
    }

    #[test]
    fn test_selected_disk_empty() {
        let screen = DiskSelectionScreen::new();
        assert!(screen.selected_disk().is_none());
    }

    #[test]
    fn test_selected_disk_with_disks() {
        let mut screen = DiskSelectionScreen::new();
        screen.disks = vec![
            make_test_disk("sda", 500_000_000_000, true),
            make_test_disk("sdb", 1_000_000_000_000, true),
        ];
        screen.detected = true;

        assert_eq!(screen.selected_disk().unwrap().name, "sda");

        screen.select_next();
        assert_eq!(screen.selected_disk().unwrap().name, "sdb");
    }

    #[test]
    fn test_navigation_bounds() {
        let mut screen = DiskSelectionScreen::new();
        screen.disks = vec![
            make_test_disk("sda", 500_000_000_000, true),
            make_test_disk("sdb", 1_000_000_000_000, true),
            make_test_disk("sdc", 250_000_000_000, true),
        ];
        screen.detected = true;

        assert_eq!(screen.selected_index, 0);

        // Can't go below 0
        screen.select_previous();
        assert_eq!(screen.selected_index, 0);

        // Move to end
        screen.select_next();
        assert_eq!(screen.selected_index, 1);
        screen.select_next();
        assert_eq!(screen.selected_index, 2);

        // Can't go past end
        screen.select_next();
        assert_eq!(screen.selected_index, 2);

        // Move back
        screen.select_previous();
        assert_eq!(screen.selected_index, 1);
    }

    #[test]
    fn test_is_selection_installable() {
        let mut screen = DiskSelectionScreen::new();
        assert!(!screen.is_selection_installable());

        screen.disks = vec![make_test_disk("sda", 500_000_000_000, true)];
        screen.detected = true;
        assert!(screen.is_selection_installable());
    }

    #[test]
    fn test_is_selection_not_installable() {
        let mut screen = DiskSelectionScreen::new();
        screen.disks = vec![make_test_disk("loop0", 500_000_000_000, false)];
        screen.detected = true;
        assert!(!screen.is_selection_installable());
    }

    #[test]
    fn test_get_disk_warning_readonly() {
        let mut screen = DiskSelectionScreen::new();
        let mut disk = make_test_disk("sda", 500_000_000_000, true);
        disk.readonly = true;
        screen.disks = vec![disk];
        screen.detected = true;

        let warning = screen.get_disk_warning();
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("read-only"));
    }

    #[test]
    fn test_get_disk_warning_removable() {
        let mut screen = DiskSelectionScreen::new();
        let mut disk = make_test_disk("sda", 500_000_000_000, true);
        disk.removable = true;
        screen.disks = vec![disk];
        screen.detected = true;

        let warning = screen.get_disk_warning();
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("removable"));
    }

    #[test]
    fn test_get_disk_warning_mounted() {
        let mut screen = DiskSelectionScreen::new();
        let mut disk = make_test_disk("sda", 500_000_000_000, true);
        disk.mountpoint = Some("/mnt".to_string());
        screen.disks = vec![disk];
        screen.detected = true;

        let warning = screen.get_disk_warning();
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("mounted"));
    }

    #[test]
    fn test_get_disk_warning_too_small() {
        let mut screen = DiskSelectionScreen::new();
        let disk = make_test_disk("sda", 4_000_000_000, true);
        screen.disks = vec![disk];
        screen.detected = true;

        let warning = screen.get_disk_warning();
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("too small"));
    }

    #[test]
    fn test_get_disk_warning_none_for_good_disk() {
        let mut screen = DiskSelectionScreen::new();
        screen.disks = vec![make_test_disk("sda", 500_000_000_000, true)];
        screen.detected = true;

        assert!(screen.get_disk_warning().is_none());
    }

    #[test]
    fn test_get_disk_warning_no_disks() {
        let screen = DiskSelectionScreen::new();
        assert!(screen.get_disk_warning().is_none());
    }

    #[test]
    fn test_detect_mock_mode() {
        std::env::set_var("EKAOS_MOCK", "1");
        let mut screen = DiskSelectionScreen::new();
        screen.detect();
        assert!(screen.detected);
        assert!(!screen.disks.is_empty());
        assert_eq!(screen.disks[0].name, "sda");
        assert_eq!(screen.disks[1].name, "nvme0n1");
        assert!(screen.error_message.is_none());
        std::env::remove_var("EKAOS_MOCK");
    }

    #[test]
    fn test_detect_only_runs_once() {
        std::env::set_var("EKAOS_MOCK", "1");
        let mut screen = DiskSelectionScreen::new();
        screen.detect();
        let count = screen.disks.len();

        // Second call should not re-detect
        screen.detect();
        assert_eq!(screen.disks.len(), count);
        std::env::remove_var("EKAOS_MOCK");
    }

    #[test]
    fn test_help_content() {
        let screen = DiskSelectionScreen::new();
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("Disk Selection"));
    }

    #[test]
    fn test_help_content_covers_shortcuts() {
        let screen = DiskSelectionScreen::new();
        let help = screen.help_content();
        let joined = help.join("\n");
        assert!(joined.contains("Up/Down"));
        assert!(joined.contains("Enter"));
        assert!(joined.contains("Quit"));
    }

    #[test]
    fn test_disk_with_partitions() {
        let mut screen = DiskSelectionScreen::new();
        let mut disk = make_test_disk("sda", 500_000_000_000, true);
        disk.partitions = vec![
            PartitionInfo {
                name: "sda1".to_string(),
                size: 512_000_000,
                size_human: "512.0 MB".to_string(),
                fstype: Some("vfat".to_string()),
                mountpoint: None,
                label: Some("EFI".to_string()),
            },
            PartitionInfo {
                name: "sda2".to_string(),
                size: 491_488_000_000,
                size_human: "491.5 GB".to_string(),
                fstype: Some("ext4".to_string()),
                mountpoint: None,
                label: Some("nixos".to_string()),
            },
        ];
        screen.disks = vec![disk];
        screen.detected = true;

        assert_eq!(screen.selected_disk().unwrap().partitions.len(), 2);
    }
}
