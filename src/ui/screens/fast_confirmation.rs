//! Fast installation confirmation screen
//!
//! Simplified confirmation for fast install mode. Shows the target disk,
//! collects a root password, and requires typing DELETE to confirm.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement, SharedString};

use crate::nixos::image::ImageFormat;
use crate::ui::theme::{icons, spacing, AppTheme};
use crate::ui::utils::NavigationHints;

/// Fast confirmation screen state
pub struct FastConfirmationScreen {
    /// Path to the disk image being written
    image_path: String,
    /// Detected image format
    image_format: String,
    /// Target disk path (e.g., "/dev/sda")
    disk_path: String,
    /// Target disk size in bytes
    disk_size: u64,
    /// Target disk display name
    disk_display: String,
    /// Root password
    root_password: String,
    /// Root password confirmation
    root_password_confirm: String,
    /// Confirmation input value — must type "DELETE" to proceed
    confirm_value: String,
    /// Whether the DELETE gate is satisfied
    confirmed: bool,
    /// Which field is focused (0 = password, 1 = confirm, 2 = DELETE)
    focused_field: usize,
}

#[allow(dead_code)]
impl FastConfirmationScreen {
    /// Create a new fast confirmation screen
    pub fn new(image_path: String) -> Self {
        let image_format = Self::detect_format(&image_path);
        Self {
            image_path,
            image_format,
            disk_path: String::new(),
            disk_size: 0,
            disk_display: String::new(),
            root_password: String::new(),
            root_password_confirm: String::new(),
            confirm_value: String::new(),
            confirmed: false,
            focused_field: 0,
        }
    }

    /// Detect and return a human-readable format string
    fn detect_format(path: &str) -> String {
        if path.is_empty() {
            return "Unknown".to_string();
        }
        match ImageFormat::detect(path) {
            Ok(fmt) => fmt.display_name().to_string(),
            Err(_) => {
                // Fall back to extension-based detection
                ImageFormat::detect_from_extension_or_raw(path)
                    .display_name()
                    .to_string()
            }
        }
    }

    /// Set the target disk info (called during screen transition)
    pub fn set_disk(&mut self, path: String, size: u64, display: String) {
        self.disk_path = path;
        self.disk_size = size;
        self.disk_display = display;
        // Reset state when disk changes
        self.confirm_value.clear();
        self.confirmed = false;
    }

    /// Get the root password
    pub fn root_password(&self) -> &str {
        &self.root_password
    }

    /// Add a character to the currently focused field
    pub fn add_char(&mut self, c: char) {
        match self.focused_field {
            0 => self.root_password.push(c),
            1 => self.root_password_confirm.push(c),
            2 => {
                self.confirm_value.push(c);
                self.confirmed = self.confirm_value == "DELETE";
            }
            _ => {}
        }
    }

    /// Remove the last character from the currently focused field
    pub fn remove_char(&mut self) {
        match self.focused_field {
            0 => {
                self.root_password.pop();
            }
            1 => {
                self.root_password_confirm.pop();
            }
            2 => {
                self.confirm_value.pop();
                self.confirmed = self.confirm_value == "DELETE";
            }
            _ => {}
        }
    }

    /// Move focus to the next field
    pub fn focus_next(&mut self) {
        if self.focused_field < 2 {
            self.focused_field += 1;
        }
    }

    /// Move focus to the previous field
    pub fn focus_previous(&mut self) {
        if self.focused_field > 0 {
            self.focused_field -= 1;
        }
    }

    /// Check if passwords match and meet minimum length
    fn passwords_valid(&self) -> bool {
        !self.root_password.is_empty()
            && self.root_password.len() >= 6
            && self.root_password == self.root_password_confirm
    }

    /// Whether this screen can proceed
    pub fn can_proceed(&self) -> bool {
        self.confirmed && self.passwords_valid() && !self.disk_path.is_empty()
    }

    /// Whether this screen can go back
    pub fn can_go_back(&self) -> bool {
        true
    }

    /// Title of this screen
    pub fn title(&self) -> &str {
        "Confirm Fast Installation"
    }

    /// Help content for this screen
    pub fn help_content(&self) -> Vec<String> {
        vec![
            "# Fast Installation Confirmation".to_string(),
            "".to_string(),
            "Fast install writes a pre-built disk image directly to the".to_string(),
            "target disk. This is much faster than a normal installation".to_string(),
            "because it skips partitioning, package downloads, and builds.".to_string(),
            "".to_string(),
            "## What You Need To Do".to_string(),
            "".to_string(),
            "1. Set a root password (minimum 6 characters)".to_string(),
            "2. Confirm the password".to_string(),
            "3. Type DELETE to confirm disk erasure".to_string(),
            "".to_string(),
            "## What Happens".to_string(),
            "".to_string(),
            "The disk image will be written directly to the target disk".to_string(),
            "using dd. After writing, the root password will be set.".to_string(),
            "".to_string(),
            "## Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- Tab: Move to next field".to_string(),
            "- Shift+Tab: Move to previous field".to_string(),
            "- Enter: Begin installation (when confirmed)".to_string(),
            "- Backspace: Go back to disk selection".to_string(),
            "- ?: Toggle this help panel".to_string(),
        ]
    }

    /// Build a labeled info row
    fn info_row(label: &str, value: &str, theme: &AppTheme) -> impl IntoElement {
        let label_text: SharedString = format!("  {}  ", label).into();
        let value_text: SharedString = value.to_string().into();

        div()
            .flex()
            .flex_row()
            .items_center()
            .child(div().text_color(theme.muted).text_sm().child(label_text))
            .child(
                div()
                    .text_color(theme.foreground)
                    .text_sm()
                    .child(value_text),
            )
    }

    /// Build a password field display
    fn password_field(
        label: &str,
        value: &str,
        is_focused: bool,
        theme: &AppTheme,
    ) -> impl IntoElement {
        let display: SharedString = if value.is_empty() && is_focused {
            "\u{2588}".to_string().into()
        } else if value.is_empty() {
            " ".to_string().into()
        } else {
            let masked = "\u{2022}".repeat(value.len());
            if is_focused {
                format!("{}\u{2588}", masked).into()
            } else {
                masked.into()
            }
        };

        let border_color = if is_focused {
            theme.primary
        } else {
            theme.border
        };
        let label_text: SharedString = label.to_string().into();

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(spacing::SMALL))
            .child(
                div()
                    .text_color(if is_focused {
                        theme.primary
                    } else {
                        theme.muted
                    })
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .child(label_text),
            )
            .child(
                div()
                    .w_full()
                    .border_1()
                    .border_color(border_color)
                    .rounded(px(4.0))
                    .px(px(spacing::MEDIUM))
                    .py(px(spacing::SMALL))
                    .text_color(theme.foreground)
                    .child(display),
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

        // Title
        col = col.child(
            div()
                .w_full()
                .flex()
                .justify_center()
                .pt(px(spacing::MEDIUM))
                .child(
                    div()
                        .text_color(theme.primary)
                        .font_weight(FontWeight::BOLD)
                        .text_xl()
                        .child("Fast Installation"),
                ),
        );

        // Subtitle
        col = col.child(
            div().w_full().flex().justify_center().child(
                div()
                    .text_color(theme.muted)
                    .text_sm()
                    .child("Write a pre-built disk image directly to the target disk"),
            ),
        );

        // Image & Disk Info section
        let info_title: SharedString = " Installation Details ".into();
        let disk_size_gb = self.disk_size as f64 / 1_000_000_000.0;
        let disk_display = if self.disk_display.is_empty() {
            format!("{} ({:.1} GB)", self.disk_path, disk_size_gb)
        } else {
            self.disk_display.clone()
        };

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
                        .child(info_title),
                )
                .child(
                    div()
                        .py(px(spacing::SMALL))
                        .flex()
                        .flex_col()
                        .child(Self::info_row("Image:      ", &self.image_path, &theme))
                        .child(Self::info_row("Format:     ", &self.image_format, &theme))
                        .child(Self::info_row("Target Disk:", &disk_display, &theme)),
                ),
        );

        // Root Password fields
        let password_title: SharedString = " Root Password ".into();
        let passwords_match = self.passwords_valid();
        let password_status = if self.root_password.is_empty() {
            ""
        } else if self.root_password.len() < 6 {
            "Too short (min 6 characters)"
        } else if self.root_password != self.root_password_confirm
            && !self.root_password_confirm.is_empty()
        {
            "Passwords do not match"
        } else if passwords_match {
            "Passwords match"
        } else {
            ""
        };

        let status_color = if passwords_match {
            theme.success
        } else {
            theme.error
        };

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
                        .child(password_title),
                )
                .child(
                    div()
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .flex()
                        .flex_col()
                        .gap(px(spacing::SMALL))
                        .child(Self::password_field(
                            "Root Password",
                            &self.root_password,
                            self.focused_field == 0,
                            &theme,
                        ))
                        .child(Self::password_field(
                            "Confirm Password",
                            &self.root_password_confirm,
                            self.focused_field == 1,
                            &theme,
                        ))
                        .child(if !password_status.is_empty() {
                            let status_text: SharedString = password_status.to_string().into();
                            div().text_color(status_color).text_sm().child(status_text)
                        } else {
                            div()
                        }),
                ),
        );

        // Warning message
        let warning_icon: SharedString = icons::WARNING.into();
        col = col.child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(spacing::SMALL))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(spacing::SMALL))
                        .child(
                            div()
                                .text_color(theme.error)
                                .font_weight(FontWeight::BOLD)
                                .child(warning_icon),
                        )
                        .child(
                            div()
                                .text_color(theme.error)
                                .font_weight(FontWeight::BOLD)
                                .child(
                                    "ALL DATA ON THIS DISK WILL BE PERMANENTLY \
                                 ERASED",
                                ),
                        ),
                )
                .child(
                    div()
                        .text_color(theme.warning)
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .child(
                            "Type DELETE below to confirm and begin \
                             installation",
                        ),
                ),
        );

        // Confirmation input
        let confirm_display: SharedString =
            if self.confirm_value.is_empty() && self.focused_field == 2 {
                "\u{2588}".to_string().into()
            } else if self.confirm_value.is_empty() {
                " ".to_string().into()
            } else if self.focused_field == 2 {
                format!("{}\u{2588}", self.confirm_value).into()
            } else {
                self.confirm_value.clone().into()
            };

        let confirm_border = if self.confirmed {
            theme.success
        } else if self.focused_field == 2 {
            theme.primary
        } else {
            theme.border
        };

        col = col.child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(spacing::SMALL))
                .child(
                    div()
                        .text_color(if self.focused_field == 2 {
                            theme.primary
                        } else {
                            theme.muted
                        })
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .child("Type DELETE to confirm"),
                )
                .child(
                    div()
                        .w_full()
                        .border_1()
                        .border_color(confirm_border)
                        .rounded(px(4.0))
                        .px(px(spacing::MEDIUM))
                        .py(px(spacing::SMALL))
                        .text_color(theme.foreground)
                        .child(confirm_display),
                ),
        );

        // Spacer
        col = col.child(div().flex_1());

        // Navigation hints
        let hints = if self.can_proceed() {
            vec![
                ("Enter", "BEGIN INSTALL"),
                ("Tab", "Next Field"),
                ("Left", "Go Back"),
                ("?", "Help"),
            ]
        } else {
            vec![
                ("Tab", "Next Field"),
                ("Left", "Go Back"),
                ("?", "Help"),
                ("q", "Quit"),
            ]
        };

        col = col.child(NavigationHints::new(hints));

        col
    }
}

impl Default for FastConfirmationScreen {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_confirmation_creation() {
        let screen = FastConfirmationScreen::new("/path/to/image.raw".to_string());
        assert_eq!(screen.title(), "Confirm Fast Installation");
        assert!(screen.can_go_back());
        assert!(!screen.can_proceed());
    }

    #[test]
    fn test_set_disk() {
        let mut screen = FastConfirmationScreen::new("/path/to/image.raw".to_string());
        screen.set_disk(
            "/dev/sda".to_string(),
            500_000_000_000,
            "sda (Samsung SSD) - 500 GB".to_string(),
        );
        assert_eq!(screen.disk_path, "/dev/sda");
        assert_eq!(screen.disk_size, 500_000_000_000);
    }

    #[test]
    fn test_password_validation() {
        let mut screen = FastConfirmationScreen::new("/path/to/image.raw".to_string());
        screen.set_disk("/dev/sda".to_string(), 500_000_000_000, String::new());

        // Empty passwords
        assert!(!screen.passwords_valid());

        // Set password but no confirm
        screen.root_password = "password123".to_string();
        assert!(!screen.passwords_valid());

        // Mismatched
        screen.root_password_confirm = "wrong".to_string();
        assert!(!screen.passwords_valid());

        // Matching
        screen.root_password_confirm = "password123".to_string();
        assert!(screen.passwords_valid());
    }

    #[test]
    fn test_password_too_short() {
        let mut screen = FastConfirmationScreen::new("/path/to/image.raw".to_string());
        screen.root_password = "short".to_string();
        screen.root_password_confirm = "short".to_string();
        assert!(!screen.passwords_valid());
    }

    #[test]
    fn test_full_confirmation_flow() {
        let mut screen = FastConfirmationScreen::new("/path/to/image.raw".to_string());
        screen.set_disk("/dev/sda".to_string(), 500_000_000_000, String::new());

        // Set password (field 0)
        for c in "password123".chars() {
            screen.add_char(c);
        }

        // Move to confirm field
        screen.focus_next();
        assert_eq!(screen.focused_field, 1);

        // Set confirm password
        for c in "password123".chars() {
            screen.add_char(c);
        }

        // Not yet confirmed (DELETE not typed)
        assert!(!screen.can_proceed());

        // Move to DELETE field
        screen.focus_next();
        assert_eq!(screen.focused_field, 2);

        // Type DELETE
        for c in "DELETE".chars() {
            screen.add_char(c);
        }

        assert!(screen.confirmed);
        assert!(screen.can_proceed());
    }

    #[test]
    fn test_focus_bounds() {
        let mut screen = FastConfirmationScreen::new(String::new());

        assert_eq!(screen.focused_field, 0);
        screen.focus_previous();
        assert_eq!(screen.focused_field, 0);

        screen.focus_next();
        screen.focus_next();
        assert_eq!(screen.focused_field, 2);

        screen.focus_next();
        assert_eq!(screen.focused_field, 2);
    }

    #[test]
    fn test_remove_char() {
        let mut screen = FastConfirmationScreen::new(String::new());

        screen.add_char('a');
        screen.add_char('b');
        assert_eq!(screen.root_password, "ab");

        screen.remove_char();
        assert_eq!(screen.root_password, "a");

        screen.remove_char();
        assert_eq!(screen.root_password, "");

        // Safe on empty
        screen.remove_char();
        assert_eq!(screen.root_password, "");
    }

    #[test]
    fn test_cannot_proceed_without_disk() {
        let mut screen = FastConfirmationScreen::new("/image.raw".to_string());
        // No disk set
        screen.root_password = "password123".to_string();
        screen.root_password_confirm = "password123".to_string();
        screen.confirmed = true;
        assert!(!screen.can_proceed());
    }

    #[test]
    fn test_set_disk_resets_confirmation() {
        let mut screen = FastConfirmationScreen::new("/image.raw".to_string());
        screen.confirmed = true;
        screen.confirm_value = "DELETE".to_string();

        screen.set_disk("/dev/sdb".to_string(), 1_000_000_000_000, String::new());
        assert!(!screen.confirmed);
        assert!(screen.confirm_value.is_empty());
    }

    #[test]
    fn test_help_content() {
        let screen = FastConfirmationScreen::new(String::new());
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("Fast Installation"));
    }

    #[test]
    fn test_default_impl() {
        let screen = FastConfirmationScreen::default();
        assert!(screen.image_path.is_empty());
        assert!(!screen.confirmed);
    }

    #[test]
    fn test_format_detection_from_extension() {
        // Files don't exist, so format is detected from extension
        let screen = FastConfirmationScreen::new("/path/to/image.qcow2".to_string());
        assert_eq!(screen.image_format, "QCOW2 (QEMU)");

        let screen = FastConfirmationScreen::new("/path/to/image.vmdk".to_string());
        assert_eq!(screen.image_format, "VMDK (VMware)");

        let screen = FastConfirmationScreen::new("/path/to/image.raw.xz".to_string());
        assert_eq!(screen.image_format, "Compressed raw (xz)");

        let screen = FastConfirmationScreen::new("/path/to/image.iso".to_string());
        assert_eq!(screen.image_format, "ISO 9660");
    }
}
