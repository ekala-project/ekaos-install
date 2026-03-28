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
}
