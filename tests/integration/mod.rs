//! Integration tests for ekaos-install
//!
//! These tests verify the interaction between multiple components.

#[cfg(test)]
mod tests {
    use ekaos_install::{
        app::{App, AppMode},
        system::{CommandExecutor, MockExecutor},
    };

    #[test]
    fn test_app_initialization() {
        let app = App::new(AppMode::Mock, false);
        assert!(app.is_mock());
        assert!(!app.should_exit);
    }

    #[test]
    fn test_mock_executor() {
        let executor = MockExecutor::new();
        assert!(executor.command_exists("lsblk"));
        let result = executor.execute("lsblk", &["-J"]).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_screen_progression() {
        use ekaos_install::app::Screen;
        let mut app = App::new(AppMode::Mock, false);

        assert_eq!(app.current_screen, Screen::Welcome);
        app.next_screen().unwrap();
        assert_eq!(app.current_screen, Screen::DiskSelection);
        app.next_screen().unwrap();
        assert_eq!(app.current_screen, Screen::PartitionPlanning);
        app.next_screen().unwrap();
        assert_eq!(app.current_screen, Screen::Configuration);
        app.next_screen().unwrap();
        assert_eq!(app.current_screen, Screen::Confirmation);
        app.next_screen().unwrap();
        assert_eq!(app.current_screen, Screen::Installation);
        app.next_screen().unwrap();
        assert_eq!(app.current_screen, Screen::Complete);
        // Cannot go past Complete
        app.next_screen().unwrap();
        assert_eq!(app.current_screen, Screen::Complete);
    }

    #[test]
    fn test_config_validation() {
        use ekaos_install::config::InstallConfig;

        let mut config = InstallConfig::default();
        config.user.username = "testuser".to_string();
        config.user.password = "password123".to_string();
        assert!(config.validate().is_ok());
    }
}
