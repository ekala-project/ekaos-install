//! Screen implementations for the installation wizard

pub mod configuration;
pub mod confirmation;
pub mod disk_selection;
pub mod fast_confirmation;
pub mod installation;
pub mod partition_planning;
pub mod success;
pub mod welcome;

pub use configuration::ConfigurationScreen;
pub use confirmation::ConfirmationScreen;
pub use disk_selection::DiskSelectionScreen;
pub use fast_confirmation::FastConfirmationScreen;
pub use installation::InstallationScreen;
pub use partition_planning::PartitionPlanningScreen;
pub use success::SuccessScreen;
pub use welcome::WelcomeScreen;
