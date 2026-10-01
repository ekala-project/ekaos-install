//! User interface components and screens
//!
//! This module contains all GUI-related code including layouts,
//! reusable components, screen implementations, and gpui actions.

pub mod components;
pub mod layout;
pub mod screens;
pub mod theme;
pub mod utils;

pub use layout::{Footer, Header};
pub use screens::{
    ConfigurationScreen, ConfirmationScreen, DiskSelectionScreen, InstallationScreen,
    PartitionPlanningScreen, SuccessScreen, WelcomeScreen,
};
pub use theme::{icons, spacing, AppTheme};
pub use utils::navigation::NavigationHints;

/// gpui actions for the installer wizard.
#[allow(missing_docs)]
pub mod actions {
    use gpui::actions;
    actions!(
        installer,
        [
            NextScreen,
            PreviousScreen,
            Quit,
            ToggleHelp,
            Confirm,
            SelectNext,
            SelectPrevious,
            ToggleItem,
            FocusNext,
            FocusPrevious,
            CycleForward,
            CycleBackward,
            ScrollUp,
            ScrollDown,
            RetryInstall,
            ToggleEncryption,
        ]
    );
}
pub use actions::*;
