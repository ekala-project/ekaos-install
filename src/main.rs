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
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use tracing::{info, warn};

use ekaos_install::{
    app::{App, AppMode, Screen},
    ui::{render_footer, render_header, Layout},
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

    // Run the event loop
    let result = run_event_loop(&mut terminal, &mut app);

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
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| {
            let layout = Layout::new();
            let (header_area, content_area, footer_area) = layout.split(frame.area());

            // Render header
            render_header(
                frame,
                header_area,
                app.current_screen,
                app.current_screen.title(),
            );

            // Render content based on current screen
            render_screen_content(frame, content_area, app);

            // Render footer
            render_footer(frame, footer_area, app.current_screen, app.is_mock());
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(app, key.code, key.modifiers)?;
            }
        }

        // Check if we should exit
        if app.should_exit {
            break;
        }
    }

    Ok(())
}

/// Render content for the current screen
fn render_screen_content(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    app: &App,
) {
    match app.current_screen {
        Screen::Welcome => render_welcome_screen(frame, area, app),
        Screen::SystemInfo => render_placeholder_screen(frame, area, "System Information"),
        Screen::DiskSelection => render_placeholder_screen(frame, area, "Disk Selection"),
        Screen::PartitionPlanning => render_placeholder_screen(frame, area, "Partition Planning"),
        Screen::Configuration => render_placeholder_screen(frame, area, "Configuration"),
        Screen::Installation => render_placeholder_screen(frame, area, "Installation"),
        Screen::Complete => render_complete_screen(frame, area),
    }
}

/// Render the welcome screen
fn render_welcome_screen(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    app: &App,
) {
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Welcome to Ekaos Install",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("A guided installer for NixOS"),
        Line::from(""),
        Line::from(""),
    ];

    // Add mode indicator
    if app.is_mock() {
        lines.push(Line::from(vec![Span::styled(
            "Running in MOCK MODE",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from("(No actual system changes will be made)"));
    }

    if app.dry_run {
        lines.push(Line::from(vec![Span::styled(
            "DRY-RUN MODE ENABLED",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("Press "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to continue"),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render a placeholder screen (for screens not yet implemented)
fn render_placeholder_screen(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    title: &str,
) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "[Coming Soon]",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(""),
        Line::from(""),
        Line::from("This screen will be implemented in a future phase."),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render the completion screen
fn render_complete_screen(frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "✓ Installation Complete!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(""),
        Line::from("NixOS has been successfully installed."),
        Line::from(""),
        Line::from("Next steps:"),
        Line::from("  1. Remove the installation media"),
        Line::from("  2. Reboot your system"),
        Line::from("  3. Login with your created user account"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::raw("Press "),
            Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" to exit"),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Handle keyboard input
fn handle_key_event(app: &mut App, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    // Handle Ctrl+C
    if key == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
        app.exit();
        return Ok(());
    }

    match key {
        KeyCode::Char('q') | KeyCode::Esc => {
            // Ask for confirmation on certain screens
            if matches!(app.current_screen, Screen::Welcome | Screen::Complete) {
                app.exit();
            } else {
                // For other screens, go back or exit
                if app.current_screen.can_go_back() {
                    app.previous_screen()?;
                } else {
                    app.exit();
                }
            }
        }
        KeyCode::Enter => {
            // Progress to next screen
            if app.current_screen.next().is_some() {
                app.next_screen()?;
            } else {
                // On final screen, exit
                app.exit();
            }
        }
        KeyCode::Left | KeyCode::Backspace => {
            // Go to previous screen
            app.previous_screen()?;
        }
        KeyCode::Right => {
            // Go to next screen
            if app.current_screen.next().is_some() {
                app.next_screen()?;
            }
        }
        _ => {}
    }

    Ok(())
}
