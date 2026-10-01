//! Installation progress screen
//!
//! Displays real-time installation progress with log output.
//! Implements `gpui::Render` for the gpui GUI framework.

use gpui::prelude::*;
use gpui::{div, px, FontWeight, IntoElement, SharedString};
use std::sync::mpsc::Receiver;

use crate::config::InstallConfig;
use crate::nixos::install::{
    run_installation_async, InstallMessage, InstallProgress, InstallStage,
};
use crate::ui::components::ProgressBar;
use crate::ui::theme::{icons, spacing, AppTheme};
use crate::ui::utils::NavigationHints;

/// Tips shown during installation
const INSTALL_TIPS: &[&str] = &[
    "Tip: Edit /etc/nixos/configuration.nix to customize your system",
    "Tip: Use 'nix-shell -p <package>' to try packages without installing",
    "Tip: Run 'nixos-rebuild switch' after editing configuration.nix",
    "Tip: NixOS generations let you roll back to previous configurations",
    "Tip: Use 'nix search nixpkgs <name>' to find packages",
    "Tip: Enable flakes with \
     'nix.settings.experimental-features = [\"nix-command\" \"flakes\"]'",
    "Tip: The NixOS manual is available at \
     https://nixos.org/manual/nixos/stable/",
    "Tip: Use 'nixos-option' to explore available configuration options",
    "Tip: Home Manager lets you manage user-level configuration with Nix",
    "Tip: NixOS has ~100,000 packages in nixpkgs \u{2014} one of the \
     largest repos",
];

/// Installation screen state
pub struct InstallationScreen {
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
    /// Current tip index
    tip_index: usize,
    /// Frame counter for tip rotation
    tip_frame_counter: usize,
    /// Stored config for retry
    last_config: Option<InstallConfig>,
    /// Stored root path for retry
    last_root_path: Option<String>,
    /// Whether we're in mock mode (for retry)
    is_mock: bool,
}

#[allow(dead_code)]
impl InstallationScreen {
    /// Create a new installation screen
    pub fn new() -> Self {
        Self {
            progress: None,
            log_lines: Vec::new(),
            is_running: false,
            is_complete: false,
            has_error: false,
            error_message: None,
            receiver: None,
            scroll_position: 0,
            tip_index: 0,
            tip_frame_counter: 0,
            last_config: None,
            last_root_path: None,
            is_mock: false,
        }
    }

    /// Start the installation process
    pub fn start_installation(&mut self, config: InstallConfig, root_path: String, is_mock: bool) {
        self.last_config = Some(config.clone());
        self.last_root_path = Some(root_path.clone());
        self.is_mock = is_mock;

        let rx = run_installation_async(config, root_path, is_mock);
        self.receiver = Some(rx);
        self.is_running = true;
        self.is_complete = false;
        self.has_error = false;
        self.error_message = None;
        self.log_lines.clear();
        self.log_lines.push("Starting installation...".to_string());
    }

    /// Retry the installation after a failure
    pub fn retry_installation(&mut self) {
        if let (Some(config), Some(root_path)) =
            (self.last_config.clone(), self.last_root_path.clone())
        {
            self.log_lines.push(String::new());
            self.log_lines
                .push("=== Retrying installation ===".to_string());

            let rx = run_installation_async(config, root_path, self.is_mock);
            self.receiver = Some(rx);
            self.is_running = true;
            self.is_complete = false;
            self.has_error = false;
            self.error_message = None;
            self.tip_frame_counter = 0;
        }
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
                        self.log_lines.push(format!(
                            "{} Installation completed successfully!",
                            icons::SUCCESS
                        ));
                    }
                    InstallMessage::Error(err) => {
                        self.is_running = false;
                        self.has_error = true;
                        self.error_message = Some(err.clone());
                        self.log_lines.push(format!(
                            "{} Installation failed: {}",
                            icons::ERROR,
                            err
                        ));
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
                InstallStage::Installing => {
                    "Installing NixOS (this may take a while)...".to_string()
                }
                InstallStage::Verifying => "Verifying installation...".to_string(),
                InstallStage::Complete => "Installation complete!".to_string(),
                InstallStage::Failed(ref err) => {
                    format!("Installation failed: {}", err)
                }
            }
        } else {
            "Initializing...".to_string()
        }
    }

    /// Whether this screen can proceed
    pub fn can_proceed(&self) -> bool {
        self.is_complete
    }

    /// Whether this screen can go back
    pub fn can_go_back(&self) -> bool {
        false // Cannot go back during/after installation
    }

    /// Title of this screen
    pub fn title(&self) -> &str {
        "Installing NixOS"
    }

    /// Help content for this screen
    pub fn help_content(&self) -> Vec<String> {
        vec![
            "# Installation Progress Screen".to_string(),
            "".to_string(),
            "NixOS is being installed to your system. This process may take".to_string(),
            "20-60 minutes depending on your internet connection and hardware.".to_string(),
            "".to_string(),
            "## What's Happening".to_string(),
            "".to_string(),
            "1. Generating Configuration: Creating your system configuration \
             files"
                .to_string(),
            "2. Hardware Detection: Detecting and configuring hardware".to_string(),
            "3. Installing Packages: Downloading and building system packages".to_string(),
            "4. Verification: Ensuring installation completed successfully".to_string(),
            "".to_string(),
            "## Important Notes".to_string(),
            "".to_string(),
            "- Do not interrupt the installation once it has started".to_string(),
            "- Keep your internet connection stable".to_string(),
            "- If installation fails, check the log for error details".to_string(),
        ]
    }

    /// Render this screen as a gpui element tree.
    pub fn view(&mut self) -> impl IntoElement {
        let theme = AppTheme::new();

        // Update installation state
        self.update();

        // Rotate tips every ~150 frames (~15 seconds at 100ms poll)
        if self.is_running {
            self.tip_frame_counter += 1;
            if self.tip_frame_counter >= 150 {
                self.tip_frame_counter = 0;
                self.tip_index = (self.tip_index + 1) % INSTALL_TIPS.len();
            }
        }

        let mut col = div()
            .flex()
            .flex_col()
            .size_full()
            .gap(px(spacing::MEDIUM))
            .p(px(spacing::LARGE));

        // === Title ===
        let title = if self.has_error {
            "Installation Failed"
        } else if self.is_complete {
            "Installation Complete"
        } else {
            "Installing NixOS"
        };

        let title_color = if self.has_error {
            theme.error
        } else if self.is_complete {
            theme.success
        } else {
            theme.primary
        };

        col = col.child(
            div()
                .w_full()
                .flex()
                .justify_center()
                .pt(px(spacing::MEDIUM))
                .child(
                    div()
                        .text_color(title_color)
                        .font_weight(FontWeight::BOLD)
                        .text_xl()
                        .child(title),
                ),
        );

        // === Progress bar ===
        let percent = self.progress.as_ref().map(|p| p.percent).unwrap_or(0);
        let mut progress_bar =
            ProgressBar::new(percent as u16).with_label(format!("Progress: {}%", percent));

        if self.has_error {
            progress_bar = progress_bar.with_color(theme.error);
        } else if self.is_complete {
            progress_bar = progress_bar.with_color(theme.success);
        }

        col = col.child(div().w_full().px(px(spacing::MEDIUM)).child(progress_bar));

        // === Current operation ===
        let operation_text: SharedString = self.stage_description().into();
        col = col.child(
            div().w_full().flex().justify_center().child(
                div()
                    .text_color(theme.foreground)
                    .text_sm()
                    .child(operation_text),
            ),
        );

        // === Tip display ===
        if self.is_running {
            let tip: SharedString = INSTALL_TIPS[self.tip_index].to_string().into();
            col = col.child(
                div()
                    .w_full()
                    .flex()
                    .justify_center()
                    .child(div().text_color(theme.info).text_sm().child(tip)),
            );
        }

        // === Log output ===
        let log_title: SharedString = " Installation Log ".into();

        // Calculate visible window (show ~20 lines centered on
        // scroll_position)
        let visible_lines = 20_usize;
        let start_idx = self.scroll_position.saturating_sub(visible_lines / 2);
        let end_idx = (start_idx + visible_lines).min(self.log_lines.len());

        let mut log_content = div()
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
                    .child(log_title),
            );

        for line in &self.log_lines[start_idx..end_idx] {
            let line_color = if line.starts_with(icons::SUCCESS) {
                theme.success
            } else if line.starts_with(icons::ERROR) {
                theme.error
            } else {
                theme.foreground
            };

            let line_text: SharedString = line.clone().into();
            log_content = log_content.child(
                div()
                    .px(px(spacing::MEDIUM))
                    .py(px(1.0))
                    .text_color(line_color)
                    .text_sm()
                    .child(line_text),
            );
        }

        col = col.child(log_content);

        // === Navigation hints ===
        if self.is_complete {
            col = col.child(NavigationHints::new(vec![(
                "Enter",
                "Continue to completion screen",
            )]));
        } else if self.has_error {
            col = col.child(NavigationHints::new(vec![
                ("q", "Exit"),
                ("r", "Retry"),
                ("Up/Down", "Scroll log"),
            ]));
        } else {
            col = col.child(NavigationHints::new(vec![("Up/Down", "Scroll log")]));
        }

        col
    }
}

impl Default for InstallationScreen {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn test_stage_description_initializing() {
        let screen = InstallationScreen::new();
        assert_eq!(screen.stage_description(), "Initializing...");
    }

    #[test]
    fn test_stage_description_with_progress() {
        let mut screen = InstallationScreen::new();
        screen.progress = Some(InstallProgress {
            stage: InstallStage::Installing,
            percent: 50,
            operation: "Installing...".to_string(),
            log_line: None,
        });
        assert_eq!(
            screen.stage_description(),
            "Installing NixOS (this may take a while)..."
        );
    }

    #[test]
    fn test_stage_description_complete() {
        let mut screen = InstallationScreen::new();
        screen.progress = Some(InstallProgress {
            stage: InstallStage::Complete,
            percent: 100,
            operation: "Done".to_string(),
            log_line: None,
        });
        assert_eq!(screen.stage_description(), "Installation complete!");
    }

    #[test]
    fn test_stage_description_failed() {
        let mut screen = InstallationScreen::new();
        screen.progress = Some(InstallProgress {
            stage: InstallStage::Failed("disk error".to_string()),
            percent: 30,
            operation: "Failed".to_string(),
            log_line: None,
        });
        assert!(screen.stage_description().contains("disk error"));
    }

    #[test]
    fn test_default_impl() {
        let screen = InstallationScreen::default();
        assert!(!screen.is_running);
        assert!(!screen.is_complete);
        assert!(!screen.has_error);
        assert!(screen.progress.is_none());
        assert!(screen.log_lines.is_empty());
    }

    #[test]
    fn test_scroll_down_at_end() {
        let mut screen = InstallationScreen::new();
        screen.log_lines = vec!["Line 1".to_string()];
        screen.scroll_position = 0;

        // Can't scroll past end
        screen.scroll_down();
        assert_eq!(screen.scroll_position, 0);
    }

    #[test]
    fn test_scroll_empty_log() {
        let mut screen = InstallationScreen::new();
        // Empty log, scroll should be safe
        screen.scroll_up();
        assert_eq!(screen.scroll_position, 0);
        screen.scroll_down();
        assert_eq!(screen.scroll_position, 0);
    }

    #[test]
    fn test_install_tips_count() {
        assert_eq!(INSTALL_TIPS.len(), 10);
        for tip in INSTALL_TIPS {
            assert!(!tip.is_empty());
            assert!(tip.starts_with("Tip:"));
        }
    }

    #[test]
    fn test_error_state() {
        let mut screen = InstallationScreen::new();
        screen.has_error = true;
        screen.error_message = Some("Disk write failed".to_string());

        assert!(!screen.can_proceed());
        assert!(!screen.can_go_back());
        assert!(screen.error_message.is_some());
    }

    #[test]
    fn test_retry_without_config() {
        let mut screen = InstallationScreen::new();
        // Retry without a previous config should be a no-op
        screen.retry_installation();
        assert!(!screen.is_running);
    }

    #[test]
    fn test_update_no_receiver() {
        let mut screen = InstallationScreen::new();
        // Update without receiver should be safe
        screen.update();
        assert!(screen.progress.is_none());
    }
}
