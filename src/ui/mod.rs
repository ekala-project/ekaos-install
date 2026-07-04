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
    Button, CheckboxList, ConfirmDialog, FilterableSelectList, HelpPanel, InputField, MessageType,
    ProgressBar, SelectList, StatusMessage,
};
pub use layout::{Layout, render_footer, render_header};
pub use screens::{
    ConfigurationScreen, ConfirmationScreen, DiskSelectionScreen, InstallationScreen,
    PartitionPlanningScreen, Screen, ScreenAction, SuccessScreen, WelcomeScreen,
};
pub use theme::{AppTheme, icons, spacing};
