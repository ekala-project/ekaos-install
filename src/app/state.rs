//! Application state management

use crate::error::Result;

/// Application mode (mock vs real)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Mock mode - no actual system operations
    Mock,
    /// Real mode - perform actual system operations
    Real,
}

/// Screens in the installation wizard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Welcome screen with pre-flight checks
    Welcome,
    /// System information display
    SystemInfo,
    /// Disk selection
    DiskSelection,
    /// Partition planning
    PartitionPlanning,
    /// Configuration setup
    Configuration,
    /// Installation progress
    Installation,
    /// Success/completion screen
    Complete,
}

impl Screen {
    /// Get the next screen in the wizard flow
    pub fn next(&self) -> Option<Self> {
        match self {
            Screen::Welcome => Some(Screen::SystemInfo),
            Screen::SystemInfo => Some(Screen::DiskSelection),
            Screen::DiskSelection => Some(Screen::PartitionPlanning),
            Screen::PartitionPlanning => Some(Screen::Configuration),
            Screen::Configuration => Some(Screen::Installation),
            Screen::Installation => Some(Screen::Complete),
            Screen::Complete => None,
        }
    }

    /// Get the previous screen (if navigation is allowed)
    pub fn previous(&self) -> Option<Self> {
        match self {
            Screen::Welcome => None,
            Screen::SystemInfo => Some(Screen::Welcome),
            Screen::DiskSelection => Some(Screen::SystemInfo),
            Screen::PartitionPlanning => Some(Screen::DiskSelection),
            Screen::Configuration => Some(Screen::PartitionPlanning),
            // Cannot go back after starting installation
            Screen::Installation => None,
            Screen::Complete => None,
        }
    }

    /// Get the title for this screen
    pub fn title(&self) -> &'static str {
        match self {
            Screen::Welcome => "Welcome to NixOS Installation",
            Screen::SystemInfo => "System Information",
            Screen::DiskSelection => "Select Installation Disk",
            Screen::PartitionPlanning => "Partition Layout",
            Screen::Configuration => "System Configuration",
            Screen::Installation => "Installing NixOS",
            Screen::Complete => "Installation Complete",
        }
    }

    /// Get the step number for progress indication
    pub fn step_number(&self) -> (usize, usize) {
        match self {
            Screen::Welcome => (1, 7),
            Screen::SystemInfo => (2, 7),
            Screen::DiskSelection => (3, 7),
            Screen::PartitionPlanning => (4, 7),
            Screen::Configuration => (5, 7),
            Screen::Installation => (6, 7),
            Screen::Complete => (7, 7),
        }
    }

    /// Check if this screen allows going back
    pub fn can_go_back(&self) -> bool {
        self.previous().is_some()
    }
}

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Current mode (mock or real)
    pub mode: AppMode,
    /// Current screen
    pub current_screen: Screen,
    /// Whether the application should exit
    pub should_exit: bool,
    /// Dry-run mode (show actions without executing)
    pub dry_run: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(mode: AppMode, dry_run: bool) -> Self {
        Self {
            mode,
            current_screen: Screen::Welcome,
            should_exit: false,
            dry_run,
        }
    }

    /// Navigate to the next screen
    pub fn next_screen(&mut self) -> Result<()> {
        if let Some(next) = self.current_screen.next() {
            self.current_screen = next;
        }
        Ok(())
    }

    /// Navigate to the previous screen
    pub fn previous_screen(&mut self) -> Result<()> {
        if let Some(prev) = self.current_screen.previous() {
            self.current_screen = prev;
        }
        Ok(())
    }

    /// Request application exit
    pub fn exit(&mut self) {
        self.should_exit = true;
    }

    /// Check if app is in mock mode
    pub fn is_mock(&self) -> bool {
        self.mode == AppMode::Mock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_progression() {
        assert_eq!(Screen::Welcome.next(), Some(Screen::SystemInfo));
        assert_eq!(Screen::Complete.next(), None);
    }

    #[test]
    fn test_screen_back_navigation() {
        assert_eq!(Screen::SystemInfo.previous(), Some(Screen::Welcome));
        assert_eq!(Screen::Welcome.previous(), None);
        assert_eq!(Screen::Installation.previous(), None);
    }

    #[test]
    fn test_app_navigation() {
        let mut app = App::new(AppMode::Mock, false);
        assert_eq!(app.current_screen, Screen::Welcome);

        app.next_screen().unwrap();
        assert_eq!(app.current_screen, Screen::SystemInfo);

        app.previous_screen().unwrap();
        assert_eq!(app.current_screen, Screen::Welcome);
    }
}
