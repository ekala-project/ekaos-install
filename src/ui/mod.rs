//! User interface components and screens
//!
//! This module contains all TUI-related code including layouts,
//! reusable components, and screen implementations.

pub mod components;
pub mod layout;
pub mod screens;
pub mod theme;
pub mod utils;

pub use components::{
    Button, CheckboxList, ConfirmDialog, HelpPanel, InputField, MessageType, ProgressBar,
    SelectList, StatusMessage,
};
pub use layout::{render_footer, render_header, Layout};
pub use screens::{
    ConfigurationScreen, DiskSelectionScreen, InstallationScreen, PartitionPlanningScreen, Screen,
    ScreenAction, SuccessScreen, SystemInfoScreen, WelcomeScreen,
};
pub use theme::{icons, spacing, AppTheme};
