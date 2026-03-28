//! Boot mode detection (UEFI vs BIOS)
//!
//! Detects whether the system is booted in UEFI or legacy BIOS mode.

use crate::error::Result;
use std::path::Path;
use tracing::debug;

/// Boot mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootMode {
    /// UEFI boot mode (modern)
    Uefi,
    /// Legacy BIOS boot mode
    Bios,
    /// Unable to determine boot mode
    Unknown,
}

impl BootMode {
    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            BootMode::Uefi => "UEFI",
            BootMode::Bios => "Legacy BIOS",
            BootMode::Unknown => "Unknown",
        }
    }

    /// Get detailed description
    pub fn description(&self) -> &'static str {
        match self {
            BootMode::Uefi => "Modern boot mode with GPT partition table support and Secure Boot",
            BootMode::Bios => "Legacy boot mode with MBR partition tables (2TB disk limit)",
            BootMode::Unknown => "Unable to determine boot mode",
        }
    }

    /// Check if this is UEFI
    pub fn is_uefi(&self) -> bool {
        matches!(self, BootMode::Uefi)
    }

    /// Check if this is BIOS
    pub fn is_bios(&self) -> bool {
        matches!(self, BootMode::Bios)
    }
}

/// Detect boot mode
///
/// Checks for the presence of `/sys/firmware/efi` which indicates UEFI boot.
/// Returns BIOS mode if the directory doesn't exist.
pub fn detect_boot_mode() -> Result<BootMode> {
    // Check if we should use mock mode
    if should_use_mock() {
        debug!("Using mock boot mode detection");
        return Ok(detect_boot_mode_mock());
    }

    detect_boot_mode_real()
}

/// Detect boot mode on real system
fn detect_boot_mode_real() -> Result<BootMode> {
    let efi_path = Path::new("/sys/firmware/efi");

    if efi_path.exists() {
        debug!("Detected UEFI boot mode (/sys/firmware/efi exists)");
        Ok(BootMode::Uefi)
    } else {
        debug!("Detected BIOS boot mode (/sys/firmware/efi not found)");
        Ok(BootMode::Bios)
    }
}

/// Mock boot mode detection (for testing)
fn detect_boot_mode_mock() -> BootMode {
    // Default to UEFI in mock mode
    BootMode::Uefi
}

/// Check if we should use mock mode
fn should_use_mock() -> bool {
    std::env::var("EKAOS_MOCK").is_ok() ||
    std::env::var("EKAOS_INSTALL_MOCK").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_mode_display() {
        assert_eq!(BootMode::Uefi.as_str(), "UEFI");
        assert_eq!(BootMode::Bios.as_str(), "Legacy BIOS");
        assert_eq!(BootMode::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_boot_mode_is_checks() {
        assert!(BootMode::Uefi.is_uefi());
        assert!(!BootMode::Uefi.is_bios());

        assert!(BootMode::Bios.is_bios());
        assert!(!BootMode::Bios.is_uefi());
    }

    #[test]
    fn test_boot_mode_description() {
        assert!(BootMode::Uefi.description().contains("GPT"));
        assert!(BootMode::Bios.description().contains("MBR"));
    }

    #[test]
    fn test_mock_detection() {
        std::env::set_var("EKAOS_MOCK", "1");
        let mode = detect_boot_mode().unwrap();
        assert_eq!(mode, BootMode::Uefi);
        std::env::remove_var("EKAOS_MOCK");
    }

    #[test]
    fn test_real_detection() {
        // This test will detect the actual boot mode of the test system
        std::env::remove_var("EKAOS_MOCK");
        std::env::remove_var("EKAOS_INSTALL_MOCK");

        let mode = detect_boot_mode().unwrap();
        // Should be either UEFI or BIOS, not Unknown
        assert!(mode.is_uefi() || mode.is_bios());
    }
}
