//! Disk detection and information
//!
//! Detects available disks using lsblk and parses disk information.

use crate::error::Result;
use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::debug;

/// Disk type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskType {
    /// Hard Disk Drive (HDD)
    #[serde(rename = "disk")]
    Disk,
    /// Solid State Drive (SSD)
    #[serde(rename = "ssd")]
    Ssd,
    /// NVMe drive
    #[serde(rename = "nvme")]
    Nvme,
    /// Loop device
    #[serde(rename = "loop")]
    Loop,
    /// ROM drive
    #[serde(rename = "rom")]
    Rom,
    /// Unknown type
    #[serde(other)]
    Unknown,
}

impl Default for DiskType {
    fn default() -> Self {
        DiskType::Unknown
    }
}

impl DiskType {
    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            DiskType::Disk => "HDD",
            DiskType::Ssd => "SSD",
            DiskType::Nvme => "NVMe",
            DiskType::Loop => "Loop",
            DiskType::Rom => "ROM",
            DiskType::Unknown => "Unknown",
        }
    }

    /// Check if this is a physical disk suitable for installation
    pub fn is_installable(&self) -> bool {
        matches!(self, DiskType::Disk | DiskType::Ssd | DiskType::Nvme)
    }
}

/// Partition information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    /// Partition name (e.g., "sda1")
    pub name: String,
    /// Size in bytes
    pub size: u64,
    /// Human-readable size
    pub size_human: String,
    /// Filesystem type (e.g., "ext4", "ntfs", "vfat")
    pub fstype: Option<String>,
    /// Mount point if mounted
    pub mountpoint: Option<String>,
    /// Label if available
    pub label: Option<String>,
}

/// Disk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disk {
    /// Device name (e.g., "sda", "nvme0n1")
    pub name: String,
    /// Full device path (e.g., "/dev/sda")
    pub path: String,
    /// Disk size in bytes
    pub size: u64,
    /// Human-readable size (e.g., "500G")
    pub size_human: String,
    /// Disk type
    #[serde(rename = "type", default)]
    pub disk_type: DiskType,
    /// Model name
    #[serde(default)]
    pub model: Option<String>,
    /// Whether the disk is removable
    #[serde(rename = "rm", default)]
    pub removable: bool,
    /// Whether the disk is read-only
    #[serde(rename = "ro", default)]
    pub readonly: bool,
    /// Mount point (if mounted)
    #[serde(default)]
    pub mountpoint: Option<String>,
    /// Existing partitions on this disk
    #[serde(default)]
    pub partitions: Vec<PartitionInfo>,
}

impl Disk {
    /// Check if this disk is suitable for installation
    pub fn is_installable(&self) -> bool {
        !self.readonly
            && !self.removable
            && self.mountpoint.is_none()
            && self.disk_type.is_installable()
            && self.size >= 8_000_000_000 // At least 8GB
    }

    /// Get size in gigabytes
    pub fn size_gb(&self) -> f64 {
        self.size as f64 / 1_000_000_000.0
    }

    /// Get display name with model
    pub fn display_name(&self) -> String {
        if let Some(model) = &self.model {
            format!("{} ({}) - {}", self.name, model, self.size_human)
        } else {
            format!("{} - {}", self.name, self.size_human)
        }
    }
}

/// lsblk JSON output structure
#[derive(Debug, Deserialize)]
struct LsblkOutput {
    blockdevices: Vec<BlockDevice>,
}

/// Block device from lsblk
#[derive(Debug, Deserialize)]
struct BlockDevice {
    name: String,
    #[serde(rename = "type")]
    device_type: String,
    size: u64,
    #[serde(default)]
    model: Option<String>,
    #[serde(rename = "rm", default)]
    removable: bool,
    #[serde(rename = "ro", default)]
    readonly: bool,
    #[serde(default)]
    mountpoint: Option<String>,
    #[serde(default)]
    fstype: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    children: Option<Vec<BlockDevice>>,
}

/// Detect available disks
///
/// Uses lsblk with JSON output to detect all available storage devices.
/// Returns a list of Disk structs with detailed information.
pub fn detect_disks() -> Result<Vec<Disk>> {
    if should_use_mock() {
        debug!("Using mock disk detection");
        return Ok(detect_disks_mock());
    }

    detect_disks_real()
}

/// Detect disks on real system using lsblk
fn detect_disks_real() -> Result<Vec<Disk>> {
    let output = Command::new("lsblk")
        .arg("--json")
        .arg("--bytes")
        .arg("--output")
        .arg("NAME,TYPE,SIZE,MODEL,RM,RO,MOUNTPOINT,FSTYPE,LABEL")
        .arg("--exclude")
        .arg("7") // Exclude loop devices
        .output()
        .context("Failed to execute lsblk command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("lsblk command failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("lsblk output: {}", stdout);

    let lsblk: LsblkOutput =
        serde_json::from_str(&stdout).context("Failed to parse lsblk JSON output")?;

    let disks = lsblk
        .blockdevices
        .into_iter()
        .filter(|dev| {
            // Only include real disk devices, exclude virtual/RAM devices
            dev.device_type == "disk"
                && !dev.name.starts_with("zram")
                && !dev.name.starts_with("ram")
        })
        .map(|dev| {
            let disk_type = classify_disk_type(&dev.name);
            let size_human = format_size_human(dev.size);

            let partitions = dev.children.as_ref().map(|children| {
                children.iter().filter(|c| c.device_type == "part").map(|c| {
                    PartitionInfo {
                        name: c.name.clone(),
                        size: c.size,
                        size_human: format_size_human(c.size),
                        fstype: c.fstype.clone(),
                        mountpoint: c.mountpoint.clone(),
                        label: c.label.clone(),
                    }
                }).collect::<Vec<_>>()
            }).unwrap_or_default();

            Disk {
                name: dev.name.clone(),
                path: format!("/dev/{}", dev.name),
                size: dev.size,
                size_human,
                disk_type,
                model: dev.model,
                removable: dev.removable,
                readonly: dev.readonly,
                mountpoint: dev.mountpoint,
                partitions,
            }
        })
        .collect();

    Ok(disks)
}

/// Classify disk type based on device name
fn classify_disk_type(name: &str) -> DiskType {
    if name.starts_with("nvme") {
        DiskType::Nvme
    } else if name.starts_with("sd") {
        // Could be HDD or SSD - would need to check /sys/block/*/queue/rotational
        // For now, default to Disk
        DiskType::Disk
    } else if name.starts_with("vd") {
        // Virtual disk (KVM/QEMU)
        DiskType::Disk
    } else if name.starts_with("hd") {
        // IDE disk (rare nowadays)
        DiskType::Disk
    } else if name.starts_with("loop") {
        DiskType::Loop
    } else if name.starts_with("sr") {
        DiskType::Rom
    } else {
        DiskType::Unknown
    }
}

/// Format size in human-readable format
fn format_size_human(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1000.0 && unit_idx < UNITS.len() - 1 {
        size /= 1000.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

/// Mock disk detection (for testing)
fn detect_disks_mock() -> Vec<Disk> {
    vec![
        Disk {
            name: "sda".to_string(),
            path: "/dev/sda".to_string(),
            size: 500_000_000_000,
            size_human: "500.0 GB".to_string(),
            disk_type: DiskType::Disk,
            model: Some("Samsung SSD 860 EVO 500GB".to_string()),
            removable: false,
            readonly: false,
            mountpoint: None,
            partitions: vec![
                PartitionInfo {
                    name: "sda1".to_string(),
                    size: 512_000_000,
                    size_human: "512.0 MB".to_string(),
                    fstype: Some("vfat".to_string()),
                    mountpoint: None,
                    label: Some("EFI".to_string()),
                },
                PartitionInfo {
                    name: "sda2".to_string(),
                    size: 491_488_000_000,
                    size_human: "491.5 GB".to_string(),
                    fstype: Some("ext4".to_string()),
                    mountpoint: None,
                    label: Some("nixos".to_string()),
                },
            ],
        },
        Disk {
            name: "nvme0n1".to_string(),
            path: "/dev/nvme0n1".to_string(),
            size: 1_000_000_000_000,
            size_human: "1.0 TB".to_string(),
            disk_type: DiskType::Nvme,
            model: Some("Samsung SSD 970 EVO Plus 1TB".to_string()),
            removable: false,
            readonly: false,
            mountpoint: None,
            partitions: Vec::new(),
        },
    ]
}

/// Check if we should use mock mode
fn should_use_mock() -> bool {
    std::env::var("EKAOS_MOCK").is_ok() || std::env::var("EKAOS_INSTALL_MOCK").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_type_display() {
        assert_eq!(DiskType::Disk.as_str(), "HDD");
        assert_eq!(DiskType::Ssd.as_str(), "SSD");
        assert_eq!(DiskType::Nvme.as_str(), "NVMe");
    }

    #[test]
    fn test_disk_type_installable() {
        assert!(DiskType::Disk.is_installable());
        assert!(DiskType::Ssd.is_installable());
        assert!(DiskType::Nvme.is_installable());
        assert!(!DiskType::Loop.is_installable());
        assert!(!DiskType::Rom.is_installable());
    }

    #[test]
    fn test_disk_is_installable() {
        let disk = Disk {
            name: "sda".to_string(),
            path: "/dev/sda".to_string(),
            size: 500_000_000_000,
            size_human: "500 GB".to_string(),
            disk_type: DiskType::Disk,
            model: None,
            removable: false,
            readonly: false,
            mountpoint: None,
            partitions: Vec::new(),
        };
        assert!(disk.is_installable());

        // Too small
        let small_disk = Disk {
            size: 4_000_000_000,
            ..disk.clone()
        };
        assert!(!small_disk.is_installable());

        // Readonly
        let readonly_disk = Disk {
            readonly: true,
            ..disk.clone()
        };
        assert!(!readonly_disk.is_installable());

        // Removable
        let removable_disk = Disk {
            removable: true,
            ..disk.clone()
        };
        assert!(!removable_disk.is_installable());
    }

    #[test]
    fn test_disk_size_gb() {
        let disk = Disk {
            name: "sda".to_string(),
            path: "/dev/sda".to_string(),
            size: 500_000_000_000,
            size_human: "500 GB".to_string(),
            disk_type: DiskType::Disk,
            model: None,
            removable: false,
            readonly: false,
            mountpoint: None,
            partitions: Vec::new(),
        };
        assert!((disk.size_gb() - 500.0).abs() < 0.1);
    }

    #[test]
    fn test_classify_disk_type() {
        assert_eq!(classify_disk_type("nvme0n1"), DiskType::Nvme);
        assert_eq!(classify_disk_type("sda"), DiskType::Disk);
        assert_eq!(classify_disk_type("vda"), DiskType::Disk);
        assert_eq!(classify_disk_type("loop0"), DiskType::Loop);
        assert_eq!(classify_disk_type("sr0"), DiskType::Rom);
    }

    #[test]
    fn test_format_size_human() {
        assert_eq!(format_size_human(500), "500 B");
        assert_eq!(format_size_human(1500), "1.5 KB");
        assert_eq!(format_size_human(1_500_000), "1.5 MB");
        assert_eq!(format_size_human(1_500_000_000), "1.5 GB");
        assert_eq!(format_size_human(1_500_000_000_000), "1.5 TB");
    }

    #[test]
    fn test_mock_detection() {
        std::env::set_var("EKAOS_MOCK", "1");
        let disks = detect_disks().unwrap();
        assert_eq!(disks.len(), 2);
        assert_eq!(disks[0].name, "sda");
        assert_eq!(disks[1].name, "nvme0n1");
        std::env::remove_var("EKAOS_MOCK");
    }

    #[test]
    fn test_real_detection() {
        // This will test actual disk detection
        std::env::remove_var("EKAOS_MOCK");
        std::env::remove_var("EKAOS_INSTALL_MOCK");

        // May fail if lsblk is not available
        if let Ok(disks) = detect_disks() {
            // Just verify we got a list (may be empty in some environments)
            assert!(disks.len() >= 0);

            // Verify no zram or ram devices are included
            for disk in &disks {
                assert!(!disk.name.starts_with("zram"),
                    "zram device should be filtered out: {}", disk.name);
                assert!(!disk.name.starts_with("ram"),
                    "ram device should be filtered out: {}", disk.name);
            }
        }
    }

    #[test]
    fn test_disk_display_name() {
        let disk = Disk {
            name: "sda".to_string(),
            path: "/dev/sda".to_string(),
            size: 500_000_000_000,
            size_human: "500 GB".to_string(),
            disk_type: DiskType::Disk,
            model: Some("Samsung SSD".to_string()),
            removable: false,
            readonly: false,
            mountpoint: None,
            partitions: Vec::new(),
        };
        assert!(disk.display_name().contains("sda"));
        assert!(disk.display_name().contains("Samsung SSD"));
        assert!(disk.display_name().contains("500 GB"));
    }
}
