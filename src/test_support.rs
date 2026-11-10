//! Helpers shared by unit tests.

use std::sync::{Mutex, MutexGuard};

/// Serializes every test that reads or writes the mock-mode env vars.
///
/// Environment variables are process-global, while `cargo test` runs tests
/// on parallel threads in a single process. Without this lock one test's
/// `remove_var("EKAOS_MOCK")` can land between another test's `set_var` and
/// its assertion.
static ENV_LOCK: Mutex<()> = Mutex::new(());

const MOCK_VARS: [&str; 2] = ["EKAOS_MOCK", "EKAOS_INSTALL_MOCK"];

/// Holds [`ENV_LOCK`] with the mock-mode env vars in a known state, and
/// clears them again on drop.
pub(crate) struct MockEnv {
    _guard: MutexGuard<'static, ()>,
}

impl MockEnv {
    /// Run with mock mode enabled (`EKAOS_MOCK=1`).
    pub(crate) fn enabled() -> Self {
        let env = Self::lock();
        std::env::set_var("EKAOS_MOCK", "1");
        env
    }

    /// Run with mock mode disabled (neither var set).
    pub(crate) fn disabled() -> Self {
        Self::lock()
    }

    fn lock() -> Self {
        // A failed assertion in another test poisons the lock; the env is
        // reset below either way, so the poison carries no information.
        let guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_mock_vars();
        Self { _guard: guard }
    }
}

impl Drop for MockEnv {
    fn drop(&mut self) {
        clear_mock_vars();
    }
}

fn clear_mock_vars() {
    for var in MOCK_VARS {
        std::env::remove_var(var);
    }
}
