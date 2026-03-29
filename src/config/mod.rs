//! NixOS configuration management
//!
//! This module handles the configuration data model and generation.

use serde::{Deserialize, Serialize};

/// NixOS installation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// System hostname
    pub hostname: String,
    /// User account configuration
    pub user: UserConfig,
    /// Timezone (e.g., "America/New_York")
    pub timezone: String,
    /// Locale (e.g., "en_US.UTF-8")
    pub locale: String,
    /// Keyboard layout (e.g., "us")
    pub keymap: String,
    /// Boot loader type
    pub bootloader: BootLoader,
    /// Whether to enable NetworkManager
    pub network_manager: bool,
    /// Disk path (e.g., "/dev/sda")
    pub disk_path: String,
    /// Disk size in bytes
    pub disk_size: u64,
    /// Swap partition size in GB
    pub swap_size_gb: u64,
    /// Root filesystem type (e.g., "ext4", "btrfs", "xfs", "zfs")
    pub root_filesystem: String,
}

/// User account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// Username (lowercase, alphanumeric)
    pub username: String,
    /// Full name (optional)
    pub full_name: Option<String>,
    /// Whether user should have sudo access
    pub is_admin: bool,
    /// Initial password (will be hashed)
    pub password: String,
}

/// Boot loader type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootLoader {
    /// systemd-boot (UEFI only)
    SystemdBoot,
    /// GRUB (works with both UEFI and BIOS)
    Grub,
}

impl BootLoader {
    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            BootLoader::SystemdBoot => "systemd-boot",
            BootLoader::Grub => "GRUB",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            BootLoader::SystemdBoot => "Modern, simple boot loader for UEFI systems",
            BootLoader::Grub => "Traditional boot loader, supports UEFI and BIOS",
        }
    }
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            hostname: "nixos".to_string(),
            user: UserConfig {
                username: String::new(),
                full_name: None,
                is_admin: true,
                password: String::new(),
            },
            timezone: "America/New_York".to_string(),
            locale: "en_US.UTF-8".to_string(),
            keymap: "us".to_string(),
            bootloader: BootLoader::SystemdBoot,
            network_manager: true,
            disk_path: String::new(),
            disk_size: 0,
            swap_size_gb: 8,
            root_filesystem: "ext4".to_string(),
        }
    }
}

impl InstallConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate hostname
        if self.hostname.is_empty() {
            return Err("Hostname cannot be empty".to_string());
        }

        if !self
            .hostname
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err("Hostname must contain only letters, numbers, and hyphens".to_string());
        }

        // Validate username
        if self.user.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }

        if !self
            .user
            .username
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(
                "Username must contain only lowercase letters, numbers, and underscores"
                    .to_string(),
            );
        }

        if self.user.username.len() > 32 {
            return Err("Username must be 32 characters or less".to_string());
        }

        // Validate password
        if self.user.password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        if self.user.password.len() < 6 {
            return Err("Password must be at least 6 characters".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = InstallConfig::default();
        assert_eq!(config.hostname, "nixos");
        assert_eq!(config.timezone, "America/New_York");
        assert_eq!(config.locale, "en_US.UTF-8");
        assert!(config.network_manager);
        // Test new disk/partition fields
        assert_eq!(config.disk_path, "");
        assert_eq!(config.disk_size, 0);
        assert_eq!(config.swap_size_gb, 8);
        assert_eq!(config.root_filesystem, "ext4");
    }

    #[test]
    fn test_bootloader_display() {
        assert_eq!(BootLoader::SystemdBoot.as_str(), "systemd-boot");
        assert_eq!(BootLoader::Grub.as_str(), "GRUB");
    }

    #[test]
    fn test_valid_config() {
        let mut config = InstallConfig::default();
        config.user.username = "testuser".to_string();
        config.user.password = "password123".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_hostname() {
        let mut config = InstallConfig::default();
        config.user.username = "test".to_string();
        config.user.password = "password".to_string();

        config.hostname = "".to_string();
        assert!(config.validate().is_err());

        config.hostname = "invalid name".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_username() {
        let mut config = InstallConfig::default();
        config.user.password = "password".to_string();

        config.user.username = "".to_string();
        assert!(config.validate().is_err());

        config.user.username = "InvalidUser".to_string();
        assert!(config.validate().is_err());

        config.user.username = "user@name".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_password() {
        let mut config = InstallConfig::default();
        config.user.username = "testuser".to_string();

        config.user.password = "".to_string();
        assert!(config.validate().is_err());

        config.user.password = "short".to_string();
        assert!(config.validate().is_err());
    }
}
