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
        components::Component, render_footer, render_header, HelpPanel, Layout, Screen,
        ScreenAction, SystemInfoScreen, WelcomeScreen,
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

/// Run the TUI application
fn run_app(mode: AppMode, dry_run: bool) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(mode, dry_run);

    // Create screen instances
    let mut welcome_screen = WelcomeScreen::new(app.is_mock(), dry_run);
    let mut system_info_screen = SystemInfoScreen::new();

    // Call on_enter for initial screen
    welcome_screen.on_enter();

    // Run the event loop
    let result = run_event_loop(
        &mut terminal,
        &mut app,
        &mut welcome_screen,
        &mut system_info_screen,
    );

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Main event loop
fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    welcome_screen: &mut WelcomeScreen,
    system_info_screen: &mut SystemInfoScreen,
) -> Result<()> {
    let mut previous_screen = app.current_screen;

    loop {
        // Check if we changed screens
        if previous_screen != app.current_screen {
            // Call lifecycle methods
            match previous_screen {
                AppScreen::Welcome => welcome_screen.on_exit(),
                AppScreen::SystemInfo => system_info_screen.on_exit(),
                _ => {}
            }

            match app.current_screen {
                AppScreen::Welcome => welcome_screen.on_enter(),
                AppScreen::SystemInfo => system_info_screen.on_enter(),
                _ => {}
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
                AppScreen::SystemInfo => system_info_screen.render(frame, content_area),
                _ => {
                    // Placeholder for unimplemented screens
                    use ratatui::{
                        style::{Color, Modifier, Style},
                        text::{Line, Span},
                        widgets::{Block, Borders, Paragraph},
                    };

                    let lines = vec![
                        Line::from(""),
                        Line::from(Span::styled(
                            "[Screen Not Yet Implemented]",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(""),
                        Line::from("This screen will be implemented in a future phase."),
                    ];

                    let paragraph = Paragraph::new(lines)
                        .block(Block::default().borders(Borders::ALL))
                        .alignment(ratatui::layout::Alignment::Center);

                    frame.render_widget(paragraph, content_area);
                }
            }

            // Render help panel if visible
            if app.help_visible {
                // Get help content from current screen
                let help_content = match app.current_screen {
                    AppScreen::Welcome => welcome_screen.help_content(),
                    AppScreen::SystemInfo => system_info_screen.help_content(),
                    _ => vec!["Help not available for this screen".to_string()],
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
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
                {
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
                    AppScreen::SystemInfo => system_info_screen.handle_input(key.code),
                    _ => {
                        // For unimplemented screens, allow basic navigation
                        match key.code {
                            KeyCode::Enter => ScreenAction::Next,
                            KeyCode::Left | KeyCode::Backspace => ScreenAction::Back,
                            KeyCode::Char('q') | KeyCode::Esc => ScreenAction::Exit,
                            _ => ScreenAction::None,
                        }
                    }
                };

                // Handle screen action
                match action {
                    ScreenAction::Next => {
                        // Check if screen allows proceeding
                        let can_proceed = match app.current_screen {
                            AppScreen::Welcome => welcome_screen.can_proceed(),
                            AppScreen::SystemInfo => system_info_screen.can_proceed(),
                            _ => true,
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
