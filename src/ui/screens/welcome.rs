//! Welcome screen with pre-flight checks
//!
//! First screen in the installer wizard. Runs system checks and displays
//! boot mode information before allowing the user to proceed.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement, SharedString};
use tracing::{debug, warn};

use crate::nixos::disk::detect_disks;
use crate::system::{check_network, detect_boot_mode, is_nixos, is_root, BootMode};
use crate::ui::theme::{icons, spacing, AppTheme};
use crate::ui::utils::NavigationHints;

/// Pre-flight check results
#[derive(Debug, Clone)]
struct PreFlightChecks {
    /// Running as root (or mock mode)
    root_access: bool,
    /// Network connectivity
    network: bool,
    /// Running from NixOS ISO
    nixos_iso: bool,
    /// Sufficient disk space available
    disk_space: bool,
}

impl Default for PreFlightChecks {
    fn default() -> Self {
        Self {
            root_access: true,
            network: true,
            nixos_iso: true,
            disk_space: true,
        }
    }
}

/// Welcome screen state
pub struct WelcomeScreen {
    /// Detected boot mode (public - needed by later screens)
    pub boot_mode: BootMode,
    /// Whether we're in mock mode
    is_mock: bool,
    /// Whether dry-run is enabled
    is_dry_run: bool,
    /// Pre-flight check results
    checks: PreFlightChecks,
}

#[allow(dead_code)]
impl WelcomeScreen {
    /// Create a new welcome screen
    pub fn new(is_mock: bool, is_dry_run: bool) -> Self {
        Self {
            boot_mode: BootMode::Unknown,
            is_mock,
            is_dry_run,
            checks: PreFlightChecks::default(),
        }
    }

    /// Run pre-flight checks (called by the root view when entering this screen)
    pub fn run_checks(&mut self) {
        debug!("Running pre-flight checks");

        // Check 1: Root access (or mock mode)
        self.checks.root_access = self.is_mock || is_root();
        debug!("Root access check: {}", self.checks.root_access);

        // Check 2: Network connectivity
        match check_network() {
            Ok(status) => {
                self.checks.network = status.is_connected();
                debug!(
                    "Network check: {} (status: {:?})",
                    self.checks.network, status
                );
            }
            Err(e) => {
                warn!("Network check failed: {}", e);
                self.checks.network = false;
            }
        }

        // Check 3: Running on NixOS
        match is_nixos() {
            Ok(result) => {
                self.checks.nixos_iso = result;
                debug!("NixOS check: {}", self.checks.nixos_iso);
            }
            Err(e) => {
                warn!("NixOS check failed: {}", e);
                self.checks.nixos_iso = false;
            }
        }

        // Check 4: Sufficient disk space (at least one installable disk exists)
        match detect_disks() {
            Ok(disks) => {
                self.checks.disk_space = disks.iter().any(|d| d.is_installable());
                debug!(
                    "Disk space check: {} ({} disks found, {} installable)",
                    self.checks.disk_space,
                    disks.len(),
                    disks.iter().filter(|d| d.is_installable()).count()
                );
            }
            Err(e) => {
                warn!("Disk detection failed: {}", e);
                self.checks.disk_space = false;
            }
        }

        debug!(
            "Pre-flight checks complete: root={}, network={}, nixos={}, disk={}",
            self.checks.root_access,
            self.checks.network,
            self.checks.nixos_iso,
            self.checks.disk_space
        );

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
    }

    /// Check if all pre-flight checks passed
    pub fn all_checks_passed(&self) -> bool {
        self.checks.root_access
            && self.checks.network
            && self.checks.nixos_iso
            && self.checks.disk_space
    }

    /// Check if this screen can proceed to the next screen
    pub fn can_proceed(&self) -> bool {
        self.all_checks_passed()
    }

    /// Get help content lines for the help panel
    pub fn help_content(&self) -> Vec<String> {
        vec![
            "# Welcome Screen".to_string(),
            "".to_string(),
            "This screen performs pre-flight checks to ensure your system is ready".to_string(),
            "for NixOS installation.".to_string(),
            "".to_string(),
            "# Pre-Flight Checks".to_string(),
            "".to_string(),
            "- Root Access: The installer must run with root privileges".to_string(),
            "- Network Connectivity: Required to download NixOS packages".to_string(),
            "- NixOS Installation Media: Must be running from official ISO".to_string(),
            "- Sufficient Disk Space: At least 10GB free space required".to_string(),
            "".to_string(),
            "# Boot Modes".to_string(),
            "".to_string(),
            "The installer automatically detects your system's boot mode:".to_string(),
            "".to_string(),
            "- UEFI: Modern boot mode, recommended for new systems".to_string(),
            "  Supports GPT partition tables and Secure Boot".to_string(),
            "".to_string(),
            "- Legacy BIOS: Older boot mode for legacy hardware".to_string(),
            "  Uses MBR partition tables (2TB disk limit)".to_string(),
            "".to_string(),
            "# Keyboard Shortcuts".to_string(),
            "".to_string(),
            "- Enter: Press Continue button (when checks pass)".to_string(),
            "- ?: Toggle this help panel".to_string(),
            "- q/Esc: Quit the installer".to_string(),
        ]
    }

    /// Build a check status element for one pre-flight check line
    fn check_element(&self, label: &str, passed: bool, theme: &AppTheme) -> impl IntoElement {
        let icon: SharedString = if passed {
            icons::SUCCESS.into()
        } else {
            icons::ERROR.into()
        };
        let icon_color = if passed { theme.success } else { theme.error };
        let label: SharedString = label.to_string().into();

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing::SMALL))
            .pl(px(spacing::MEDIUM))
            .py(px(2.0))
            .child(
                div()
                    .text_color(icon_color)
                    .font_weight(FontWeight::BOLD)
                    .child(icon),
            )
            .child(div().text_color(theme.foreground).child(label))
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
                .pt(px(spacing::LARGE))
                .child(
                    div()
                        .text_color(theme.primary)
                        .font_weight(FontWeight::BOLD)
                        .text_xl()
                        .child("Welcome to Ekaos Install"),
                ),
        );

        // Subtitle
        col = col.child(
            div().w_full().flex().justify_center().child(
                div()
                    .text_color(theme.foreground)
                    .text_sm()
                    .child("A guided installer for NixOS"),
            ),
        );

        // Mode indicators
        if self.is_mock || self.is_dry_run {
            let mut mode_col = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(spacing::SMALL));

            if self.is_mock {
                mode_col = mode_col.child(
                    div()
                        .text_color(theme.warning)
                        .font_weight(FontWeight::BOLD)
                        .child("Running in MOCK MODE"),
                );
                mode_col = mode_col.child(
                    div()
                        .text_color(theme.muted)
                        .text_sm()
                        .child("(No actual system changes will be made)"),
                );
            }

            if self.is_dry_run {
                mode_col = mode_col.child(
                    div()
                        .text_color(theme.info)
                        .font_weight(FontWeight::BOLD)
                        .child("DRY-RUN MODE ENABLED"),
                );
            }

            col = col.child(mode_col);
        }

        // Pre-flight checks block
        let checks_title: SharedString = " Pre-Flight Checks ".into();
        let mut checks_block = div()
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
                    .child(checks_title),
            );

        checks_block = checks_block.child(
            div()
                .py(px(spacing::SMALL))
                .child(self.check_element("Root Access", self.checks.root_access, &theme))
                .child(self.check_element("Network Connectivity", self.checks.network, &theme))
                .child(self.check_element(
                    "NixOS Installation Media",
                    self.checks.nixos_iso,
                    &theme,
                ))
                .child(self.check_element("Sufficient Disk Space", self.checks.disk_space, &theme)),
        );

        col = col.child(checks_block);

        // Boot mode block
        let boot_title: SharedString = " Boot Mode ".into();
        let mode_label: SharedString = self.boot_mode.as_str().into();
        let mode_desc: SharedString = self.boot_mode.description().into();

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
                        .child(boot_title),
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
                                .flex()
                                .flex_row()
                                .gap(px(spacing::SMALL))
                                .child(div().text_color(theme.muted).child("Mode:"))
                                .child(
                                    div()
                                        .text_color(theme.info)
                                        .font_weight(FontWeight::BOLD)
                                        .child(mode_label),
                                ),
                        )
                        .child(div().text_color(theme.muted).text_sm().child(mode_desc)),
                ),
        );

        // Error message if checks failed
        if !self.all_checks_passed() {
            col = col.child(
                div().w_full().flex().justify_center().child(
                    div()
                        .text_color(theme.error)
                        .child("Please resolve the issues above before continuing"),
                ),
            );
        }

        // Spacer
        col = col.child(div().flex_1());

        // Navigation hints
        col = col.child(NavigationHints::new(vec![
            ("Enter", "Continue"),
            ("?", "Help"),
            ("q", "Quit"),
        ]));

        col
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_screen_creation() {
        let screen = WelcomeScreen::new(true, false);
        assert!(screen.is_mock);
        assert!(!screen.is_dry_run);
        assert_eq!(screen.boot_mode, BootMode::Unknown);
    }

    #[test]
    fn test_welcome_screen_dry_run() {
        let screen = WelcomeScreen::new(false, true);
        assert!(!screen.is_mock);
        assert!(screen.is_dry_run);
    }

    #[test]
    fn test_preflight_checks_default_pass() {
        let screen = WelcomeScreen::new(false, false);
        // Default checks are all true
        assert!(screen.all_checks_passed());
        assert!(screen.can_proceed());
    }

    #[test]
    fn test_preflight_checks_mock_mode() {
        std::env::set_var("EKAOS_MOCK", "1");
        let mut screen = WelcomeScreen::new(true, false);
        screen.run_checks();
        assert!(screen.all_checks_passed());
        assert!(screen.can_proceed());
        assert_eq!(screen.boot_mode, BootMode::Uefi);
        std::env::remove_var("EKAOS_MOCK");
    }

    #[test]
    fn test_preflight_checks_failure() {
        let mut screen = WelcomeScreen::new(false, false);
        screen.checks.root_access = false;
        assert!(!screen.all_checks_passed());
        assert!(!screen.can_proceed());
    }

    #[test]
    fn test_preflight_checks_network_failure() {
        let mut screen = WelcomeScreen::new(false, false);
        screen.checks.network = false;
        assert!(!screen.all_checks_passed());
    }

    #[test]
    fn test_preflight_checks_nixos_failure() {
        let mut screen = WelcomeScreen::new(false, false);
        screen.checks.nixos_iso = false;
        assert!(!screen.all_checks_passed());
    }

    #[test]
    fn test_preflight_checks_disk_failure() {
        let mut screen = WelcomeScreen::new(false, false);
        screen.checks.disk_space = false;
        assert!(!screen.all_checks_passed());
    }

    #[test]
    fn test_help_content() {
        let screen = WelcomeScreen::new(false, false);
        let help = screen.help_content();
        assert!(!help.is_empty());
        assert!(help[0].contains("Welcome Screen"));
    }

    #[test]
    fn test_help_content_covers_boot_modes() {
        let screen = WelcomeScreen::new(false, false);
        let help = screen.help_content();
        let joined = help.join("\n");
        assert!(joined.contains("UEFI"));
        assert!(joined.contains("Legacy BIOS"));
    }

    #[test]
    fn test_help_content_covers_shortcuts() {
        let screen = WelcomeScreen::new(false, false);
        let help = screen.help_content();
        let joined = help.join("\n");
        assert!(joined.contains("Enter"));
        assert!(joined.contains("Quit"));
    }
}
