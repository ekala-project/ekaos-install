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

/// Installation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMode {
    /// Normal installation - full NixOS install with partitioning and configuration
    Normal,
    /// Fast installation - write a pre-built disk image directly to disk
    Fast,
}

/// Screens in the installation wizard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Welcome screen with pre-flight checks and system info
    Welcome,
    /// Disk selection
    DiskSelection,
    /// Partition planning (normal mode only)
    PartitionPlanning,
    /// Configuration setup (normal mode only)
    Configuration,
    /// Confirmation/review screen (normal mode only)
    Confirmation,
    /// Fast install confirmation: root password + DELETE gate (fast mode only)
    FastConfirmation,
    /// Installation progress
    Installation,
    /// Success/completion screen
    Complete,
}

impl Screen {
    /// Get the next screen in the wizard flow for normal mode
    pub fn next(&self) -> Option<Self> {
        self.next_for_mode(InstallMode::Normal)
    }

    /// Get the next screen for the given install mode
    pub fn next_for_mode(&self, mode: InstallMode) -> Option<Self> {
        match mode {
            InstallMode::Normal => match self {
                Screen::Welcome => Some(Screen::DiskSelection),
                Screen::DiskSelection => Some(Screen::PartitionPlanning),
                Screen::PartitionPlanning => Some(Screen::Configuration),
                Screen::Configuration => Some(Screen::Confirmation),
                Screen::Confirmation => Some(Screen::Installation),
                Screen::Installation => Some(Screen::Complete),
                Screen::Complete | Screen::FastConfirmation => None,
            },
            InstallMode::Fast => match self {
                Screen::Welcome => Some(Screen::DiskSelection),
                Screen::DiskSelection => Some(Screen::FastConfirmation),
                Screen::FastConfirmation => Some(Screen::Installation),
                Screen::Installation => Some(Screen::Complete),
                Screen::Complete
                | Screen::PartitionPlanning
                | Screen::Configuration
                | Screen::Confirmation => None,
            },
        }
    }

    /// Get the previous screen (if navigation is allowed)
    pub fn previous(&self) -> Option<Self> {
        self.previous_for_mode(InstallMode::Normal)
    }

    /// Get the previous screen for the given install mode
    pub fn previous_for_mode(&self, mode: InstallMode) -> Option<Self> {
        match mode {
            InstallMode::Normal => match self {
                Screen::Welcome => None,
                Screen::DiskSelection => Some(Screen::Welcome),
                Screen::PartitionPlanning => Some(Screen::DiskSelection),
                Screen::Configuration => Some(Screen::PartitionPlanning),
                Screen::Confirmation => Some(Screen::Configuration),
                Screen::Installation | Screen::Complete | Screen::FastConfirmation => None,
            },
            InstallMode::Fast => match self {
                Screen::Welcome => None,
                Screen::DiskSelection => Some(Screen::Welcome),
                Screen::FastConfirmation => Some(Screen::DiskSelection),
                Screen::Installation | Screen::Complete => None,
                Screen::PartitionPlanning | Screen::Configuration | Screen::Confirmation => None,
            },
        }
    }

    /// Get the title for this screen
    pub fn title(&self) -> &'static str {
        match self {
            Screen::Welcome => "Welcome to NixOS Installation",
            Screen::DiskSelection => "Select Installation Disk",
            Screen::PartitionPlanning => "Partition Layout",
            Screen::Configuration => "System Configuration",
            Screen::Confirmation => "Review Configuration",
            Screen::FastConfirmation => "Confirm Fast Installation",
            Screen::Installation => "Installing NixOS",
            Screen::Complete => "Installation Complete",
        }
    }

    /// Get the step number for progress indication
    pub fn step_number(&self) -> (usize, usize) {
        self.step_number_for_mode(InstallMode::Normal)
    }

    /// Get the step number for the given install mode
    pub fn step_number_for_mode(&self, mode: InstallMode) -> (usize, usize) {
        match mode {
            InstallMode::Normal => match self {
                Screen::Welcome => (1, 7),
                Screen::DiskSelection => (2, 7),
                Screen::PartitionPlanning => (3, 7),
                Screen::Configuration => (4, 7),
                Screen::Confirmation => (5, 7),
                Screen::FastConfirmation => (5, 7),
                Screen::Installation => (6, 7),
                Screen::Complete => (7, 7),
            },
            InstallMode::Fast => match self {
                Screen::Welcome => (1, 4),
                Screen::DiskSelection => (2, 4),
                Screen::FastConfirmation => (3, 4),
                Screen::Installation => (4, 4),
                Screen::Complete => (4, 4),
                Screen::PartitionPlanning | Screen::Configuration | Screen::Confirmation => (1, 4),
            },
        }
    }

    /// Check if this screen allows going back
    pub fn can_go_back(&self) -> bool {
        self.previous().is_some()
    }

    /// Check if this screen allows going back in the given mode
    pub fn can_go_back_for_mode(&self, mode: InstallMode) -> bool {
        self.previous_for_mode(mode).is_some()
    }
}

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Current mode (mock or real)
    pub mode: AppMode,
    /// Installation mode (normal or fast)
    pub install_mode: InstallMode,
    /// Current screen
    pub current_screen: Screen,
    /// Whether the application should exit
    pub should_exit: bool,
    /// Dry-run mode (show actions without executing)
    pub dry_run: bool,
    /// Whether help panel is visible
    pub help_visible: bool,
    /// Path to disk image for fast install
    pub image_path: Option<String>,
}

impl App {
    /// Create a new application instance
    pub fn new(mode: AppMode, dry_run: bool) -> Self {
        Self {
            mode,
            install_mode: InstallMode::Normal,
            current_screen: Screen::Welcome,
            should_exit: false,
            dry_run,
            help_visible: false,
            image_path: None,
        }
    }

    /// Create a new application instance in fast install mode
    pub fn new_fast(mode: AppMode, dry_run: bool, image_path: String) -> Self {
        Self {
            mode,
            install_mode: InstallMode::Fast,
            current_screen: Screen::Welcome,
            should_exit: false,
            dry_run,
            help_visible: false,
            image_path: Some(image_path),
        }
    }

    /// Toggle help panel visibility
    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }

    /// Navigate to the next screen
    pub fn next_screen(&mut self) -> Result<()> {
        if let Some(next) = self.current_screen.next_for_mode(self.install_mode) {
            self.current_screen = next;
        }
        Ok(())
    }

    /// Navigate to the previous screen
    pub fn previous_screen(&mut self) -> Result<()> {
        if let Some(prev) = self.current_screen.previous_for_mode(self.install_mode) {
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

    /// Check if app is in fast install mode
    pub fn is_fast(&self) -> bool {
        self.install_mode == InstallMode::Fast
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_progression() {
        assert_eq!(Screen::Welcome.next(), Some(Screen::DiskSelection));
        assert_eq!(Screen::Complete.next(), None);
    }

    #[test]
    fn test_screen_back_navigation() {
        assert_eq!(Screen::DiskSelection.previous(), Some(Screen::Welcome));
        assert_eq!(Screen::Welcome.previous(), None);
        assert_eq!(Screen::Installation.previous(), None);
    }

    #[test]
    fn test_app_navigation() {
        let mut app = App::new(AppMode::Mock, false);
        assert_eq!(app.current_screen, Screen::Welcome);

        app.next_screen().unwrap();
        assert_eq!(app.current_screen, Screen::DiskSelection);

        app.previous_screen().unwrap();
        assert_eq!(app.current_screen, Screen::Welcome);
    }

    #[test]
    fn test_fast_mode_screen_progression() {
        let mode = InstallMode::Fast;
        assert_eq!(
            Screen::Welcome.next_for_mode(mode),
            Some(Screen::DiskSelection)
        );
        assert_eq!(
            Screen::DiskSelection.next_for_mode(mode),
            Some(Screen::FastConfirmation)
        );
        assert_eq!(
            Screen::FastConfirmation.next_for_mode(mode),
            Some(Screen::Installation)
        );
        assert_eq!(
            Screen::Installation.next_for_mode(mode),
            Some(Screen::Complete)
        );
        assert_eq!(Screen::Complete.next_for_mode(mode), None);
    }

    #[test]
    fn test_fast_mode_back_navigation() {
        let mode = InstallMode::Fast;
        assert_eq!(Screen::Welcome.previous_for_mode(mode), None);
        assert_eq!(
            Screen::DiskSelection.previous_for_mode(mode),
            Some(Screen::Welcome)
        );
        assert_eq!(
            Screen::FastConfirmation.previous_for_mode(mode),
            Some(Screen::DiskSelection)
        );
        assert_eq!(Screen::Installation.previous_for_mode(mode), None);
    }

    #[test]
    fn test_fast_mode_step_numbers() {
        let mode = InstallMode::Fast;
        assert_eq!(Screen::Welcome.step_number_for_mode(mode), (1, 4));
        assert_eq!(Screen::DiskSelection.step_number_for_mode(mode), (2, 4));
        assert_eq!(Screen::FastConfirmation.step_number_for_mode(mode), (3, 4));
        assert_eq!(Screen::Installation.step_number_for_mode(mode), (4, 4));
    }

    #[test]
    fn test_fast_app_navigation() {
        let mut app = App::new_fast(AppMode::Mock, false, "/path/to/image.raw".to_string());
        assert_eq!(app.install_mode, InstallMode::Fast);
        assert_eq!(app.current_screen, Screen::Welcome);

        app.next_screen().unwrap();
        assert_eq!(app.current_screen, Screen::DiskSelection);

        app.next_screen().unwrap();
        assert_eq!(app.current_screen, Screen::FastConfirmation);

        app.previous_screen().unwrap();
        assert_eq!(app.current_screen, Screen::DiskSelection);
    }

    #[test]
    fn test_fast_confirmation_title() {
        assert_eq!(
            Screen::FastConfirmation.title(),
            "Confirm Fast Installation"
        );
    }
}
