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
}
