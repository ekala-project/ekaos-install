//! Ekaos Install - NixOS Installation TUI
//!
//! Main entry point for the application.

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tracing::{info, warn};

use ekaos_install::{
    app::{App, AppMode, Screen as AppScreen},
    ui::{
        components::Component, render_footer, render_header, ConfigurationScreen,
        ConfirmationScreen, DiskSelectionScreen, HelpPanel, InstallationScreen, Layout,
        PartitionPlanningScreen, Screen, ScreenAction, SuccessScreen, WelcomeScreen,
    },
    APP_NAME, VERSION,
};

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(name = APP_NAME, version = VERSION, about = "NixOS Installation TUI")]
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

fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize logging
    init_logging(args.verbose);

    info!("Starting {} v{}", APP_NAME, VERSION);

    // Determine app mode
    let mode = if args.mock {
        info!("Running in mock mode");
        AppMode::Mock
    } else {
        AppMode::Real
    };

    // Run the TUI application
    let result = run_app(mode, args.dry_run);

    // Log exit status
    match &result {
        Ok(_) => info!("Application exited successfully"),
        Err(e) => warn!("Application exited with error: {}", e),
    }

    result
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

/// RAII guard for terminal state - ensures cleanup on drop
///
/// This guard automatically restores the terminal to its original state
/// when dropped, even in case of panics or early returns. This prevents
/// leaving the terminal in a bad state (raw mode, alternate screen, etc.)
struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalGuard {
    /// Create a new terminal guard and setup the terminal
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    /// Get a mutable reference to the terminal
    fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    /// Restore terminal state on drop - always runs, even on panic
    fn drop(&mut self) {
        // Ignore errors during cleanup to ensure all cleanup steps run
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

/// Run the TUI application
fn run_app(mode: AppMode, dry_run: bool) -> Result<()> {
    // Setup terminal with RAII guard for automatic cleanup
    let mut terminal_guard = TerminalGuard::new()?;
    let terminal = terminal_guard.terminal_mut();

    // Create app state
    let mut app = App::new(mode, dry_run);

    // Create screen instances
    let mut welcome_screen = WelcomeScreen::new(app.is_mock(), dry_run);
    let mut disk_selection_screen = DiskSelectionScreen::new();
    let mut partition_planning_screen = PartitionPlanningScreen::new();
    let mut configuration_screen = ConfigurationScreen::new();
    let mut confirmation_screen = ConfirmationScreen::new();
    let mut installation_screen = InstallationScreen::new();
    let mut success_screen = SuccessScreen::new();

    // Call on_enter for initial screen
    welcome_screen.on_enter();

    // Run the event loop
    run_event_loop(
        terminal,
        &mut app,
        &mut welcome_screen,
        &mut disk_selection_screen,
        &mut partition_planning_screen,
        &mut configuration_screen,
        &mut confirmation_screen,
        &mut installation_screen,
        &mut success_screen,
    )
    // Terminal is automatically cleaned up when terminal_guard goes out of scope
    // This happens even if the event loop panics or returns an error
}

/// Main event loop
fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    welcome_screen: &mut WelcomeScreen,
    disk_selection_screen: &mut DiskSelectionScreen,
    partition_planning_screen: &mut PartitionPlanningScreen,
    configuration_screen: &mut ConfigurationScreen,
    confirmation_screen: &mut ConfirmationScreen,
    installation_screen: &mut InstallationScreen,
    success_screen: &mut SuccessScreen,
) -> Result<()> {
    let mut previous_screen = app.current_screen;

    loop {
        // Check if we changed screens
        if previous_screen != app.current_screen {
            // Call lifecycle methods
            match previous_screen {
                AppScreen::Welcome => welcome_screen.on_exit(),
                AppScreen::DiskSelection => disk_selection_screen.on_exit(),
                AppScreen::PartitionPlanning => partition_planning_screen.on_exit(),
                AppScreen::Configuration => configuration_screen.on_exit(),
                AppScreen::Confirmation => confirmation_screen.on_exit(),
                AppScreen::Installation => installation_screen.on_exit(),
                AppScreen::Complete => success_screen.on_exit(),
            }

            match app.current_screen {
                AppScreen::Welcome => welcome_screen.on_enter(),
                AppScreen::DiskSelection => disk_selection_screen.on_enter(),
                AppScreen::PartitionPlanning => {
                    // Pass disk info to partition planning screen
                    if let Some(disk) = disk_selection_screen.selected_disk() {
                        partition_planning_screen.set_disk(
                            welcome_screen.boot_mode,
                            disk.path.clone(),
                            disk.size,
                        );
                    }
                    partition_planning_screen.on_enter();
                }
                AppScreen::Configuration => {
                    // Pass boot mode to configuration screen
                    configuration_screen.set_boot_mode(welcome_screen.boot_mode);
                    // Pass disk and partition configuration
                    configuration_screen.set_disk_config(
                        partition_planning_screen.disk_path().to_string(),
                        partition_planning_screen.disk_size(),
                        partition_planning_screen.swap_size_gb(),
                        partition_planning_screen.root_filesystem().to_string(),
                        partition_planning_screen.luks_enabled(),
                        partition_planning_screen.luks_passphrase().to_string(),
                    );
                    configuration_screen.on_enter();
                }
                AppScreen::Confirmation => {
                    // Pass configuration to confirmation screen for review
                    let config = configuration_screen.get_config().clone();
                    confirmation_screen.set_config(config);
                    confirmation_screen.on_enter();
                }
                AppScreen::Installation => {
                    // Start installation with confirmed configuration
                    if let Some(config) = confirmation_screen.get_config() {
                        installation_screen.start_installation(config.clone(), "/mnt".to_string(), app.is_mock());
                    }
                    installation_screen.on_enter();
                }
                AppScreen::Complete => success_screen.on_enter(),
            }

            previous_screen = app.current_screen;
        }

        // Draw UI
        terminal.draw(|frame| {
            let layout = Layout::new();
            let (header_area, content_area, footer_area) = layout.split(frame.size());

            // Render header
            render_header(
                frame,
                header_area,
                app.current_screen,
                app.current_screen.title(),
            );

            // Render current screen
            match app.current_screen {
                AppScreen::Welcome => welcome_screen.render(frame, content_area),
                AppScreen::DiskSelection => disk_selection_screen.render(frame, content_area),
                AppScreen::PartitionPlanning => {
                    partition_planning_screen.render(frame, content_area)
                }
                AppScreen::Configuration => configuration_screen.render(frame, content_area),
                AppScreen::Confirmation => confirmation_screen.render(frame, content_area),
                AppScreen::Installation => installation_screen.render(frame, content_area),
                AppScreen::Complete => success_screen.render(frame, content_area),
            }

            // Render help panel if visible
            if app.help_visible {
                // Get help content from current screen
                let help_content = match app.current_screen {
                    AppScreen::Welcome => welcome_screen.help_content(),
                    AppScreen::DiskSelection => disk_selection_screen.help_content(),
                    AppScreen::PartitionPlanning => partition_planning_screen.help_content(),
                    AppScreen::Configuration => configuration_screen.help_content(),
                    AppScreen::Confirmation => confirmation_screen.help_content(),
                    AppScreen::Installation => installation_screen.help_content(),
                    AppScreen::Complete => success_screen.help_content(),
                };

                let mut temp_help = HelpPanel::new("Help").with_content(help_content);
                temp_help.set_visible(true);
                temp_help.render(frame, content_area);
            }

            // Render footer
            render_footer(frame, footer_area, app.current_screen, app.is_mock());
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle Ctrl+C globally
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.exit();
                    continue;
                }

                // Handle '?' globally for help
                if key.code == KeyCode::Char('?') {
                    app.toggle_help();
                    continue;
                }

                // If help is visible, close it on any key
                if app.help_visible {
                    app.toggle_help();
                    continue;
                }

                // Route input to current screen
                let action = match app.current_screen {
                    AppScreen::Welcome => welcome_screen.handle_input(key.code),
                    AppScreen::DiskSelection => disk_selection_screen.handle_input(key.code),
                    AppScreen::PartitionPlanning => {
                        partition_planning_screen.handle_input(key.code)
                    }
                    AppScreen::Configuration => configuration_screen.handle_input(key.code),
                    AppScreen::Confirmation => confirmation_screen.handle_input(key.code),
                    AppScreen::Installation => installation_screen.handle_input(key.code),
                    AppScreen::Complete => success_screen.handle_input(key.code),
                };

                // Handle screen action
                match action {
                    ScreenAction::Next => {
                        // Check if screen allows proceeding
                        let can_proceed = match app.current_screen {
                            AppScreen::Welcome => welcome_screen.can_proceed(),
                            AppScreen::DiskSelection => disk_selection_screen.can_proceed(),
                            AppScreen::PartitionPlanning => partition_planning_screen.can_proceed(),
                            AppScreen::Configuration => configuration_screen.can_proceed(),
                            AppScreen::Confirmation => confirmation_screen.can_proceed(),
                            AppScreen::Installation => installation_screen.can_proceed(),
                            AppScreen::Complete => success_screen.can_proceed(),
                        };

                        if can_proceed {
                            app.next_screen()?;
                        }
                    }
                    ScreenAction::Back => {
                        app.previous_screen()?;
                    }
                    ScreenAction::Exit => {
                        app.exit();
                    }
                    ScreenAction::ToggleHelp => {
                        app.toggle_help();
                    }
                    ScreenAction::None => {
                        // Do nothing
                    }
                }
            }
        }

        // Check if we should exit
        if app.should_exit {
            break;
        }
    }

    Ok(())
}
