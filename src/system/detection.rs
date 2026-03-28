//! System information detection
//!
//! Detects CPU, RAM, architecture, and other system properties.

use crate::error::Result;
use std::fs;
use std::process::Command;
use tracing::debug;

/// System information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// CPU model name
    pub cpu_model: String,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Total RAM in bytes
    pub ram_bytes: u64,
    /// Total RAM in gigabytes
    pub ram_gb: f64,
    /// System architecture (x86_64, aarch64, etc.)
    pub architecture: String,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            cpu_model: "Unknown CPU".to_string(),
            cpu_cores: 1,
            ram_bytes: 0,
            ram_gb: 0.0,
            architecture: "unknown".to_string(),
        }
    }
}

/// Detect complete system information
pub fn detect_system() -> Result<SystemInfo> {
    if should_use_mock() {
        debug!("Using mock system detection");
        return Ok(detect_system_mock());
    }

    let cpu_model = get_cpu_model().unwrap_or_else(|_| "Unknown CPU".to_string());
    let cpu_cores = get_cpu_cores().unwrap_or(1);
    let ram_bytes = get_ram_bytes().unwrap_or(0);
    let ram_gb = ram_bytes as f64 / 1_073_741_824.0; // Convert to GB
    let architecture = get_architecture().unwrap_or_else(|_| "unknown".to_string());

    Ok(SystemInfo {
        cpu_model,
        cpu_cores,
        ram_bytes,
        ram_gb,
        architecture,
    })
}

/// Get CPU model name from /proc/cpuinfo
fn get_cpu_model() -> Result<String> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;

    for line in cpuinfo.lines() {
        if line.starts_with("model name") {
            if let Some(model) = line.split(':').nth(1) {
                return Ok(model.trim().to_string());
            }
        }
    }

    Ok("Unknown CPU".to_string())
}

/// Get number of CPU cores from /proc/cpuinfo
fn get_cpu_cores() -> Result<usize> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;
    let count = cpuinfo
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count();

    Ok(count.max(1))
}

/// Get total RAM in bytes from /proc/meminfo
fn get_ram_bytes() -> Result<u64> {
    let meminfo = fs::read_to_string("/proc/meminfo")?;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            // Format: "MemTotal:       16384000 kB"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(kb) = parts[1].parse::<u64>() {
                    // Convert KB to bytes
                    return Ok(kb * 1024);
                }
            }
        }
    }

    Ok(0)
}

/// Get system architecture
fn get_architecture() -> Result<String> {
    let output = Command::new("uname").arg("-m").output()?;

    if output.status.success() {
        let arch = String::from_utf8_lossy(&output.stdout);
        Ok(arch.trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

/// Check if running as root
pub fn is_root() -> bool {
    if should_use_mock() {
        return true; // Mock mode always "root"
    }

    // Check UID
    #[cfg(unix)]
    {
        unsafe { libc::getuid() == 0 }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, check EUID environment variable
        std::env::var("EUID").map(|v| v == "0").unwrap_or(false)
    }
}

/// Check if running on NixOS
pub fn is_nixos() -> Result<bool> {
    if should_use_mock() {
        return Ok(true); // Mock mode assumes NixOS
    }

    // Check /etc/os-release
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("ID=") && line.contains("nixos") {
                return Ok(true);
            }
            if line.starts_with("ID_LIKE=") && line.contains("nixos") {
                return Ok(true);
            }
        }
    }

    // Also check for /nix/store as fallback
    Ok(std::path::Path::new("/nix/store").exists())
}

/// Mock system detection
fn detect_system_mock() -> SystemInfo {
    SystemInfo {
        cpu_model: "Intel Core i7-8700K (Simulated)".to_string(),
        cpu_cores: 8,
        ram_bytes: 17_179_869_184, // 16 GB
        ram_gb: 16.0,
        architecture: "x86_64".to_string(),
    }
}

/// Check if we should use mock mode
fn should_use_mock() -> bool {
    std::env::var("EKAOS_MOCK").is_ok() || std::env::var("EKAOS_INSTALL_MOCK").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_default() {
        let info = SystemInfo::default();
        assert_eq!(info.cpu_model, "Unknown CPU");
        assert_eq!(info.cpu_cores, 1);
    }

    #[test]
    fn test_mock_detection() {
        // Clean up first in case other tests left vars set
        std::env::remove_var("EKAOS_INSTALL_MOCK");
        std::env::set_var("EKAOS_MOCK", "1");

        let info = detect_system().unwrap();
        assert!(info.cpu_model.contains("Simulated"));
        assert_eq!(info.cpu_cores, 8);
        assert_eq!(info.ram_gb, 16.0);

        std::env::remove_var("EKAOS_MOCK");
    }

    #[test]
    fn test_real_detection() {
        // Clean up environment
        std::env::remove_var("EKAOS_MOCK");
        std::env::remove_var("EKAOS_INSTALL_MOCK");

        let info = detect_system().unwrap();
        // Should have detected something
        assert!(info.cpu_cores > 0);
        assert!(!info.architecture.is_empty());
    }

    #[test]
    fn test_cpu_model_parsing() {
        // This will test on the actual system
        if let Ok(model) = get_cpu_model() {
            assert!(!model.is_empty());
        }
    }

    #[test]
    fn test_cpu_cores_parsing() {
        if let Ok(cores) = get_cpu_cores() {
            assert!(cores > 0);
            assert!(cores <= 256); // Sanity check
        }
    }

    #[test]
    fn test_ram_detection() {
        if let Ok(ram) = get_ram_bytes() {
            assert!(ram > 0);
            // Should be at least 512MB (sanity check)
            assert!(ram >= 512 * 1024 * 1024);
        }
    }

    #[test]
    fn test_architecture_detection() {
        if let Ok(arch) = get_architecture() {
            assert!(!arch.is_empty());
            // Common architectures
            assert!(
                arch.contains("x86_64")
                    || arch.contains("aarch64")
                    || arch.contains("armv")
                    || arch.contains("i686")
            );
        }
    }

    #[test]
    fn test_is_root_mock() {
        // Clean up first
        std::env::remove_var("EKAOS_INSTALL_MOCK");
        std::env::set_var("EKAOS_MOCK", "1");

        assert!(is_root());

        std::env::remove_var("EKAOS_MOCK");
    }

    #[test]
    fn test_is_nixos() {
        // Will return true in NixOS environment or mock mode
        if let Ok(result) = is_nixos() {
            // Just check it doesn't panic
            let _ = result;
        }
    }
}
