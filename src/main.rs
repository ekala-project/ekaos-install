//! Ekaos Install - NixOS Installation GUI
//!
//! Main entry point for the application.

use clap::Parser;
use gpui::prelude::*;
use gpui::{div, px, AnyElement, Application, KeyBinding, WindowOptions};
use tracing::info;

use ekaos_install::{
    app::{App as InstallerApp, AppMode, Screen as AppScreen},
    ui::{
        spacing, AppTheme, ConfigurationScreen, ConfirmationScreen, DiskSelectionScreen, Footer,
        Header, InstallationScreen, NextScreen, PartitionPlanningScreen, PreviousScreen, Quit,
        SuccessScreen, ToggleHelp, WelcomeScreen,
    },
    APP_NAME, VERSION,
};

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(name = APP_NAME, version = VERSION, about = "NixOS Installation GUI")]
struct Args {
    /// Run in mock mode (no actual system operations)
    #[arg(long, short)]
    mock: bool,

    /// Dry-run mode (show what would be done without executing)
    #[arg(long)]
    dry_run: bool,

    /// Increase logging verbosity
    #[arg(long, short)]
    verbose: bool,

    /// Load configuration from file
    #[arg(long, short, value_name = "FILE")]
    config: Option<String>,
}

fn main() {
    let args = Args::parse();
    init_logging(args.verbose);
    detect_display();

    info!("Starting {} v{}", APP_NAME, VERSION);

    let mode = if args.mock {
        info!("Running in mock mode");
        AppMode::Mock
    } else {
        AppMode::Real
    };

    let dry_run = args.dry_run;

    Application::new().run(move |cx| {
        register_keybindings(cx);

        cx.open_window(
            WindowOptions {
                focus: true,
                show: true,
                ..Default::default()
            },
            |_window, cx| cx.new(|_cx| InstallerRoot::new(mode, dry_run)),
        )
        .expect("Failed to open window");
    });
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: bool) {
    use tracing_subscriber::filter::LevelFilter;

    let filter = if verbose {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(filter)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();
}

/// Auto-detect and set display server environment variables if missing.
///
/// On NixOS (and Linux generally), GUI applications need either `DISPLAY` (X11)
/// or `WAYLAND_DISPLAY` (Wayland) to be set. In some contexts (e.g. running from
/// a terminal multiplexer, SSH, or a sandboxed environment) these may not be
/// inherited. This function probes for running display servers and sets the
/// appropriate variable.
fn detect_display() {
    use std::env;
    use std::path::Path;

    // Ensure XDG_RUNTIME_DIR is set — nix develop often strips it
    if env::var("XDG_RUNTIME_DIR").is_err() {
        let uid = unsafe { libc::getuid() };
        let runtime_dir = format!("/run/user/{uid}");
        if Path::new(&runtime_dir).is_dir() {
            env::set_var("XDG_RUNTIME_DIR", &runtime_dir);
        }
    }

    let has_wayland = env::var("WAYLAND_DISPLAY")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let has_x11 = env::var("DISPLAY")
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    if has_wayland || has_x11 {
        return;
    }

    // Try Wayland first: look for wayland-* sockets in XDG_RUNTIME_DIR
    if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with("wayland-") && !name.ends_with(".lock") {
                    info!("Auto-detected Wayland display: {}", name);
                    env::set_var("WAYLAND_DISPLAY", name.as_ref());
                    return;
                }
            }
        }
    }

    // Fall back to X11: look for sockets in /tmp/.X11-unix/
    let x11_dir = Path::new("/tmp/.X11-unix");
    if x11_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(x11_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if let Some(num) = name.strip_prefix('X') {
                    let display_val = format!(":{num}");
                    info!("Auto-detected X11 display: {display_val}");
                    env::set_var("DISPLAY", &display_val);
                    return;
                }
            }
        }
    }

    info!("No display server detected; gpui will attempt default connection");
}

/// Register global key bindings for the installer
fn register_keybindings(cx: &mut gpui::App) {
    cx.bind_keys([
        KeyBinding::new("enter", NextScreen, Some("Installer")),
        KeyBinding::new("backspace", PreviousScreen, Some("Installer")),
        KeyBinding::new("ctrl-c", Quit, None),
        KeyBinding::new("escape", Quit, Some("Installer")),
        KeyBinding::new("?", ToggleHelp, Some("Installer")),
    ]);
}

/// Root view that owns all screen state and manages the wizard flow
struct InstallerRoot {
    /// Application state machine
    app: InstallerApp,
    /// Screen instances
    welcome: WelcomeScreen,
    disk_selection: DiskSelectionScreen,
    partition_planning: PartitionPlanningScreen,
    configuration: ConfigurationScreen,
    confirmation: ConfirmationScreen,
    installation: InstallationScreen,
    success: SuccessScreen,
    /// Help panel visibility
    help_visible: bool,
    /// Track the previous screen for lifecycle management
    previous_screen: AppScreen,
}

impl InstallerRoot {
    fn new(mode: AppMode, dry_run: bool) -> Self {
        let is_mock = mode == AppMode::Mock;
        let mut welcome = WelcomeScreen::new(is_mock, dry_run);
        welcome.run_checks();

        Self {
            app: InstallerApp::new(mode, dry_run),
            welcome,
            disk_selection: DiskSelectionScreen::new(),
            partition_planning: PartitionPlanningScreen::new(),
            configuration: ConfigurationScreen::new(),
            confirmation: ConfirmationScreen::new(),
            installation: InstallationScreen::new(),
            success: SuccessScreen::new(),
            help_visible: false,
            previous_screen: AppScreen::Welcome,
        }
    }

    /// Handle screen transition, passing data between screens
    fn handle_screen_transition(&mut self) {
        if self.previous_screen == self.app.current_screen {
            return;
        }

        match self.app.current_screen {
            AppScreen::DiskSelection => {
                self.disk_selection.detect();
            }
            AppScreen::PartitionPlanning => {
                if let Some(disk) = self.disk_selection.selected_disk() {
                    self.partition_planning.set_disk(
                        self.welcome.boot_mode,
                        disk.path.clone(),
                        disk.size,
                    );
                }
            }
            AppScreen::Configuration => {
                self.configuration.set_boot_mode(self.welcome.boot_mode);
                self.configuration.set_disk_config(
                    self.partition_planning.disk_path().to_string(),
                    self.partition_planning.disk_size(),
                    self.partition_planning.swap_size_gb(),
                    self.partition_planning.root_filesystem().to_string(),
                    self.partition_planning.luks_enabled(),
                    self.partition_planning.luks_passphrase().to_string(),
                );
            }
            AppScreen::Confirmation => {
                let config = self.configuration.get_config().clone();
                self.confirmation.set_config(config);
            }
            AppScreen::Installation => {
                if let Some(config) = self.confirmation.get_config() {
                    self.installation.start_installation(
                        config.clone(),
                        "/mnt".to_string(),
                        self.app.is_mock(),
                    );
                }
            }
            _ => {}
        }

        self.previous_screen = self.app.current_screen;
    }

    /// Navigate to next screen
    fn on_next_screen(
        &mut self,
        _: &NextScreen,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, Self>,
    ) {
        let can_proceed = match self.app.current_screen {
            AppScreen::Welcome => self.welcome.can_proceed(),
            AppScreen::DiskSelection => self.disk_selection.is_selection_installable(),
            AppScreen::PartitionPlanning => true,
            AppScreen::Configuration => self.configuration.can_proceed(),
            AppScreen::Confirmation => self.confirmation.can_proceed(),
            AppScreen::Installation => self.installation.can_proceed(),
            AppScreen::Complete => {
                cx.quit();
                return;
            }
        };

        if can_proceed {
            let _ = self.app.next_screen();
            self.handle_screen_transition();
            cx.notify();
        }
    }

    /// Navigate to previous screen
    fn on_previous_screen(
        &mut self,
        _: &PreviousScreen,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, Self>,
    ) {
        let _ = self.app.previous_screen();
        self.handle_screen_transition();
        cx.notify();
    }

    /// Quit the application
    fn on_quit(&mut self, _: &Quit, _window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self>) {
        cx.quit();
    }

    /// Toggle help panel
    fn on_toggle_help(
        &mut self,
        _: &ToggleHelp,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, Self>,
    ) {
        self.help_visible = !self.help_visible;
        cx.notify();
    }

    /// Render the current screen content
    fn render_current_screen(&mut self) -> AnyElement {
        match self.app.current_screen {
            AppScreen::Welcome => self.welcome.view().into_any_element(),
            AppScreen::DiskSelection => self.disk_selection.view().into_any_element(),
            AppScreen::PartitionPlanning => self.partition_planning.view().into_any_element(),
            AppScreen::Configuration => self.configuration.view().into_any_element(),
            AppScreen::Confirmation => self.confirmation.view().into_any_element(),
            AppScreen::Installation => {
                self.installation.update();
                self.installation.view().into_any_element()
            }
            AppScreen::Complete => self.success.view().into_any_element(),
        }
    }
}

impl Render for InstallerRoot {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, Self>,
    ) -> impl IntoElement {
        let theme = AppTheme::new();
        let screen_content = self.render_current_screen();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .text_color(theme.foreground)
            .key_context("Installer")
            .on_action(cx.listener(Self::on_next_screen))
            .on_action(cx.listener(Self::on_previous_screen))
            .on_action(cx.listener(Self::on_quit))
            .on_action(cx.listener(Self::on_toggle_help))
            // Header
            .child(Header::new(
                self.app.current_screen,
                self.app.current_screen.title(),
            ))
            // Main content area
            .child(
                div()
                    .id("content")
                    .flex_1()
                    .overflow_y_scroll()
                    .p(px(spacing::LARGE))
                    .child(screen_content),
            )
            // Footer
            .child(Footer::new(self.app.current_screen, self.app.is_mock()))
    }
}
