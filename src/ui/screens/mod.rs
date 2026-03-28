//! Screen implementations for the installation wizard
//!
//! Each screen represents a step in the installation process.

use crossterm::event::KeyCode;
use ratatui::{layout::Rect, Frame};

pub mod configuration;
pub mod disk_selection;
pub mod installation;
pub mod partition_planning;
pub mod success;
pub mod system_info;
pub mod welcome;

pub use configuration::ConfigurationScreen;
pub use disk_selection::DiskSelectionScreen;
pub use installation::InstallationScreen;
pub use partition_planning::PartitionPlanningScreen;
pub use success::SuccessScreen;
pub use system_info::SystemInfoScreen;
pub use welcome::WelcomeScreen;

/// Action to take after handling input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenAction {
    /// Do nothing, stay on current screen
    None,
    /// Move to next screen
    Next,
    /// Go back to previous screen
    Back,
    /// Exit the application
    Exit,
    /// Show help
    ToggleHelp,
}

/// Trait for screens in the installation wizard
pub trait Screen {
    /// Render the screen
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect);

    /// Handle keyboard input
    /// Returns an action to take (None, Next, Back, Exit)
    fn handle_input(&mut self, key: KeyCode) -> ScreenAction;

    /// Called when entering this screen
    fn on_enter(&mut self) {}

    /// Called when exiting this screen
    fn on_exit(&mut self) {}

    /// Get help content for this screen
    fn help_content(&self) -> Vec<String> {
        vec![]
    }

    /// Get screen title
    fn title(&self) -> &str;

    /// Check if this screen can be navigated away from
    /// (allows validation before proceeding)
    fn can_proceed(&self) -> bool {
        true
    }

    /// Check if this screen allows going back
    fn can_go_back(&self) -> bool {
        true
    }

    /// Handle standard keyboard input (quit, help)
    ///
    /// This method provides default handling for common keys:
    /// - `q` or `Esc`: Exit the application
    /// - `?`: Toggle help panel
    ///
    /// Returns `Some(ScreenAction)` if the key was handled, or `None` if the
    /// screen should handle it with custom logic.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
    ///     // Try standard handlers first
    ///     if let Some(action) = self.handle_standard_input(key) {
    ///         return action;
    ///     }
    ///
    ///     // Handle screen-specific keys
    ///     match key {
    ///         KeyCode::Enter => ScreenAction::Next,
    ///         _ => ScreenAction::None,
    ///     }
    /// }
    /// ```
    fn handle_standard_input(&self, key: KeyCode) -> Option<ScreenAction> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => Some(ScreenAction::Exit),
            KeyCode::Char('?') => Some(ScreenAction::ToggleHelp),
            _ => None,
        }
    }

    /// Handle back navigation input
    ///
    /// This method provides default handling for back navigation keys:
    /// - `←` (Left arrow) or `Backspace`: Go back to previous screen
    ///
    /// The back action is only returned if `can_go_back()` returns true.
    ///
    /// Returns `Some(ScreenAction)` if the key was handled, or `None` if the
    /// screen should handle it with custom logic.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// fn handle_input(&mut self, key: KeyCode) -> ScreenAction {
    ///     if let Some(action) = self.handle_standard_input(key) {
    ///         return action;
    ///     }
    ///     if let Some(action) = self.handle_back_input(key) {
    ///         return action;
    ///     }
    ///
    ///     // Handle other keys...
    ///     ScreenAction::None
    /// }
    /// ```
    fn handle_back_input(&self, key: KeyCode) -> Option<ScreenAction> {
        if self.can_go_back() {
            match key {
                KeyCode::Left | KeyCode::Backspace => Some(ScreenAction::Back),
                _ => None,
            }
        } else {
            None
        }
    }
}
