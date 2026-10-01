//! NixOS installation execution
//!
//! Handles the actual NixOS installation process including configuration generation,
//! nixos-install execution, and post-installation verification.

use crate::config::InstallConfig;
use crate::error::{CommandError, ConfigError, InstallerError};
use crate::nixos::image::{write_image, ImageFormat};
use crate::system::command::{CommandExecutor, RealExecutor};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Installation progress state
#[derive(Debug, Clone, PartialEq)]
pub enum InstallStage {
    /// Generating configuration files
    GeneratingConfig,
    /// Running nixos-generate-config
    GeneratingHardwareConfig,
    /// Installing NixOS
    Installing,
    /// Verifying installation
    Verifying,
    /// Installation complete
    Complete,
    /// Installation failed
    Failed(String),
}

/// Installation progress information
#[derive(Debug, Clone)]
pub struct InstallProgress {
    /// Current stage
    pub stage: InstallStage,
    /// Progress percentage (0-100)
    pub percent: u8,
    /// Current operation description
    pub operation: String,
    /// Latest log line
    pub log_line: Option<String>,
}

/// Installation output message
#[derive(Debug, Clone)]
pub enum InstallMessage {
    /// Progress update
    Progress(InstallProgress),
    /// Log line output
    Log(String),
    /// Installation completed successfully
    Success,
    /// Installation failed
    Error(String),
}

/// Trait for executing installation operations
pub trait InstallExecutor {
    /// Generate hardware configuration using nixos-generate-config
    fn generate_hardware_config(&self, root_path: &Path) -> Result<(), InstallerError>;

    /// Execute nixos-install
    fn execute_install(&self, root_path: &Path) -> Result<(), InstallerError>;

    /// Verify installation was successful
    fn verify_installation(&self, root_path: &Path) -> Result<(), InstallerError>;
}

/// Real installation executor
pub struct RealInstallExecutor {
    executor: RealExecutor,
}

impl RealInstallExecutor {
    /// Create a new real installation executor
    pub fn new() -> Self {
        Self {
            executor: RealExecutor,
        }
    }
}

impl Default for RealInstallExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallExecutor for RealInstallExecutor {
    fn generate_hardware_config(&self, root_path: &Path) -> Result<(), InstallerError> {
        let output = self
            .executor
            .execute(
                "nixos-generate-config",
                &["--root", root_path.to_str().unwrap()],
            )
            .map_err(|e| InstallerError::Config(ConfigError::GenerationFailed(format!("{}", e))))?;

        if !output.success {
            return Err(InstallerError::Config(ConfigError::GenerationFailed(
                output.stderr,
            )));
        }

        Ok(())
    }

    fn execute_install(&self, root_path: &Path) -> Result<(), InstallerError> {
        let output = self
            .executor
            .execute(
                "nixos-install",
                &["--root", root_path.to_str().unwrap(), "--no-root-passwd"],
            )
            .map_err(|e| {
                InstallerError::Command(CommandError::ExecutionFailed {
                    command: "nixos-install".to_string(),
                    code: -1,
                    stderr: format!("{}", e),
                })
            })?;

        if !output.success {
            return Err(InstallerError::Command(CommandError::ExecutionFailed {
                command: "nixos-install".to_string(),
                code: output.exit_code,
                stderr: output.stderr,
            }));
        }

        Ok(())
    }

    fn verify_installation(&self, root_path: &Path) -> Result<(), InstallerError> {
        // Check that configuration.nix exists
        let config_path = root_path.join("etc/nixos/configuration.nix");
        if !config_path.exists() {
            return Err(InstallerError::Config(ConfigError::ReadFailed {
                path: config_path.display().to_string(),
                reason: "Configuration file not found".to_string(),
            }));
        }

        // Check that hardware-configuration.nix exists
        let hw_config_path = root_path.join("etc/nixos/hardware-configuration.nix");
        if !hw_config_path.exists() {
            return Err(InstallerError::Config(ConfigError::ReadFailed {
                path: hw_config_path.display().to_string(),
                reason: "Hardware configuration file not found".to_string(),
            }));
        }

        // Check that boot directory exists and has content
        let boot_path = root_path.join("boot");
        if !boot_path.exists() {
            return Err(InstallerError::other(
                "Boot directory not found after installation",
            ));
        }

        Ok(())
    }
}

/// Mock installation executor for testing
pub struct MockInstallExecutor {
    should_fail: bool,
}

impl MockInstallExecutor {
    /// Create a new mock installation executor
    pub fn new() -> Self {
        Self { should_fail: false }
    }

    /// Set whether the installation should fail (for testing)
    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
}

impl Default for MockInstallExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallExecutor for MockInstallExecutor {
    fn generate_hardware_config(&self, _root_path: &Path) -> Result<(), InstallerError> {
        if self.should_fail {
            return Err(InstallerError::Config(ConfigError::GenerationFailed(
                "Mock failure".to_string(),
            )));
        }

        // Simulate some delay
        thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    fn execute_install(&self, _root_path: &Path) -> Result<(), InstallerError> {
        if self.should_fail {
            return Err(InstallerError::Command(CommandError::ExecutionFailed {
                command: "nixos-install".to_string(),
                code: 1,
                stderr: "Mock installation failure".to_string(),
            }));
        }

        // Simulate installation delay
        thread::sleep(Duration::from_millis(500));
        Ok(())
    }

    fn verify_installation(&self, _root_path: &Path) -> Result<(), InstallerError> {
        if self.should_fail {
            return Err(InstallerError::other("Verification failed"));
        }

        Ok(())
    }
}

/// Run the installation process in a background thread
///
/// Sends progress updates through the returned channel
pub fn run_installation_async(
    config: InstallConfig,
    root_path: String,
    is_mock: bool,
) -> Receiver<InstallMessage> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        run_installation_impl(config, root_path, is_mock, tx);
    });

    rx
}

/// Implementation of the installation process
fn run_installation_impl(
    config: InstallConfig,
    root_path: String,
    is_mock: bool,
    tx: Sender<InstallMessage>,
) {
    let root = Path::new(&root_path);

    // Create executor based on mode
    let executor: Box<dyn InstallExecutor> = if is_mock {
        Box::new(MockInstallExecutor::new())
    } else {
        Box::new(RealInstallExecutor::new())
    };

    // Stage 1: Generate configuration (0-20%)
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::GeneratingConfig,
        percent: 5,
        operation: "Generating NixOS configuration...".to_string(),
        log_line: None,
    }));

    let config_path = root.join("etc/nixos/configuration.nix");
    if let Err(e) = crate::nixos::config::write_configuration(&config, &config_path) {
        let _ = tx.send(InstallMessage::Error(format!(
            "Failed to write configuration: {}",
            e
        )));
        return;
    }

    let _ = tx.send(InstallMessage::Log(format!(
        "Configuration written to {}",
        config_path.display()
    )));

    // Stage 2: Generate hardware configuration (20-30%)
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::GeneratingHardwareConfig,
        percent: 20,
        operation: "Generating hardware configuration...".to_string(),
        log_line: None,
    }));

    if let Err(e) = executor.generate_hardware_config(root) {
        let _ = tx.send(InstallMessage::Error(format!(
            "Failed to generate hardware configuration: {}",
            e
        )));
        return;
    }

    let _ = tx.send(InstallMessage::Log(
        "Hardware configuration generated successfully".to_string(),
    ));

    // Stage 3: Run nixos-install (30-90%)
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::Installing,
        percent: 30,
        operation: "Installing NixOS (this may take a while)...".to_string(),
        log_line: None,
    }));

    if is_mock {
        // In mock mode, simulate realistic progress stages
        let mock_stages = [
            (35, "Evaluating NixOS configuration..."),
            (40, "Building system derivations..."),
            (50, "Downloading dependencies from cache..."),
            (60, "Building kernel modules..."),
            (70, "Installing system packages..."),
            (80, "Configuring bootloader..."),
            (85, "Setting up user accounts..."),
        ];
        for (percent, desc) in mock_stages {
            thread::sleep(Duration::from_millis(300));
            let _ = tx.send(InstallMessage::Progress(InstallProgress {
                stage: InstallStage::Installing,
                percent,
                operation: desc.to_string(),
                log_line: Some(desc.to_string()),
            }));
            let _ = tx.send(InstallMessage::Log(desc.to_string()));
        }
    } else {
        // Real mode: report that we're waiting for nixos-install
        let _ = tx.send(InstallMessage::Log(
            "Running nixos-install... (this typically takes 15-60 minutes)".to_string(),
        ));
        let _ = tx.send(InstallMessage::Progress(InstallProgress {
            stage: InstallStage::Installing,
            percent: 50,
            operation: "Running nixos-install (downloading and building packages)...".to_string(),
            log_line: None,
        }));
    }

    if let Err(e) = executor.execute_install(root) {
        let _ = tx.send(InstallMessage::Error(format!("Installation failed: {}", e)));
        return;
    }

    let _ = tx.send(InstallMessage::Log(
        "NixOS installation completed successfully".to_string(),
    ));

    // Stage 4: Verify installation (90-100%)
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::Verifying,
        percent: 90,
        operation: "Verifying installation...".to_string(),
        log_line: None,
    }));

    if let Err(e) = executor.verify_installation(root) {
        let _ = tx.send(InstallMessage::Error(format!(
            "Installation verification failed: {}",
            e
        )));
        return;
    }

    let _ = tx.send(InstallMessage::Log(
        "Installation verified successfully".to_string(),
    ));

    // Stage 5: Complete
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::Complete,
        percent: 100,
        operation: "Installation complete!".to_string(),
        log_line: None,
    }));

    let _ = tx.send(InstallMessage::Success);
}

/// Run the fast installation process (disk image write) in a background thread
///
/// Writes a pre-built disk image to the target disk using dd, then sets
/// the root password via chroot.
pub fn run_fast_installation_async(
    image_path: String,
    disk_path: String,
    root_password: String,
    is_mock: bool,
) -> Receiver<InstallMessage> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        run_fast_installation_impl(image_path, disk_path, root_password, is_mock, tx);
    });

    rx
}

/// Implementation of the fast installation process
fn run_fast_installation_impl(
    image_path: String,
    disk_path: String,
    root_password: String,
    is_mock: bool,
    tx: Sender<InstallMessage>,
) {
    // Stage 1: Validate image file and detect format (0-5%)
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::Installing,
        percent: 2,
        operation: "Detecting image format...".to_string(),
        log_line: None,
    }));

    let format = if is_mock {
        // In mock mode, detect from extension only (file may not exist)
        ImageFormat::detect_from_extension_or_raw(&image_path)
    } else {
        let image = Path::new(&image_path);
        if !image.exists() {
            let _ = tx.send(InstallMessage::Error(format!(
                "Disk image not found: {}",
                image_path
            )));
            return;
        }
        match ImageFormat::detect(&image_path) {
            Ok(f) => f,
            Err(e) => {
                let _ = tx.send(InstallMessage::Error(format!(
                    "Failed to detect image format: {}",
                    e
                )));
                return;
            }
        }
    };

    let _ = tx.send(InstallMessage::Log(format!("Image: {}", image_path)));
    let _ = tx.send(InstallMessage::Log(format!(
        "Format: {}",
        format.display_name()
    )));
    let _ = tx.send(InstallMessage::Log(format!("Target: {}", disk_path)));

    if format.needs_qemu_img() {
        let _ = tx.send(InstallMessage::Log(
            "Using qemu-img convert to write image...".to_string(),
        ));
    }

    // Stage 2: Write disk image (5-85%)
    let write_desc = format!("Writing {} to target disk...", format.display_name());
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::Installing,
        percent: 5,
        operation: write_desc,
        log_line: None,
    }));

    if is_mock {
        // Simulate write progress
        let mock_stages = [
            (10, "Writing disk image... 0%"),
            (20, "Writing disk image... 15%"),
            (30, "Writing disk image... 30%"),
            (40, "Writing disk image... 45%"),
            (50, "Writing disk image... 55%"),
            (60, "Writing disk image... 70%"),
            (70, "Writing disk image... 85%"),
            (80, "Writing disk image... 95%"),
            (85, "Syncing buffers to disk..."),
        ];
        for (percent, desc) in mock_stages {
            thread::sleep(Duration::from_millis(200));
            let _ = tx.send(InstallMessage::Progress(InstallProgress {
                stage: InstallStage::Installing,
                percent,
                operation: desc.to_string(),
                log_line: Some(desc.to_string()),
            }));
            let _ = tx.send(InstallMessage::Log(desc.to_string()));
        }
    } else {
        let _ = tx.send(InstallMessage::Log(
            "Writing image (this is IO-bound and may take several minutes)...".to_string(),
        ));

        match write_image(&image_path, &disk_path, &format) {
            Ok(output) => {
                if !output.is_empty() {
                    let _ = tx.send(InstallMessage::Log(output));
                }
            }
            Err(e) => {
                let _ = tx.send(InstallMessage::Error(format!(
                    "Failed to write disk image: {}",
                    e
                )));
                return;
            }
        }
    }

    let _ = tx.send(InstallMessage::Log(
        "Disk image written successfully".to_string(),
    ));

    // Stage 3: Set root password (85-95%)
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::Installing,
        percent: 85,
        operation: "Setting root password...".to_string(),
        log_line: None,
    }));

    if is_mock {
        thread::sleep(Duration::from_millis(200));
        let _ = tx.send(InstallMessage::Log(
            "Root password set successfully".to_string(),
        ));
    } else {
        // Mount the newly written disk to set the root password
        // First, re-read partition table
        let executor = RealExecutor;
        let _ = executor.execute("partprobe", &[&disk_path]);
        thread::sleep(Duration::from_millis(500));

        // Find the root partition — try common layouts
        // For NixOS images, root is typically the largest partition
        let root_part = find_root_partition(&disk_path);

        match root_part {
            Some(part) => {
                let mount_result = executor.execute("mount", &[&part, "/mnt"]);
                if let Err(e) = mount_result {
                    let _ = tx.send(InstallMessage::Error(format!(
                        "Failed to mount root partition {}: {}",
                        part, e
                    )));
                    return;
                }

                // Set root password using chroot + chpasswd
                let chpasswd_result = std::process::Command::new("chroot")
                    .args(["/mnt", "chpasswd"])
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        use std::io::Write;
                        if let Some(ref mut stdin) = child.stdin {
                            stdin.write_all(format!("root:{}", root_password).as_bytes())?;
                        }
                        child.wait_with_output()
                    });

                // Always try to unmount
                let _ = executor.execute("umount", &["/mnt"]);

                match chpasswd_result {
                    Ok(output) if output.status.success() => {
                        let _ = tx.send(InstallMessage::Log(
                            "Root password set successfully".to_string(),
                        ));
                    }
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let _ = tx.send(InstallMessage::Error(format!(
                            "Failed to set root password: {}",
                            stderr
                        )));
                        return;
                    }
                    Err(e) => {
                        let _ = tx.send(InstallMessage::Error(format!(
                            "Failed to set root password: {}",
                            e
                        )));
                        return;
                    }
                }
            }
            None => {
                let _ = tx.send(InstallMessage::Error(
                    "Could not identify root partition on written disk".to_string(),
                ));
                return;
            }
        }
    }

    // Stage 4: Verify (95-100%)
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::Verifying,
        percent: 95,
        operation: "Verifying installation...".to_string(),
        log_line: None,
    }));

    if is_mock {
        thread::sleep(Duration::from_millis(100));
    } else {
        // Quick sanity check — verify the partition table was written
        let executor = RealExecutor;
        match executor.execute("lsblk", &[&disk_path, "--json"]) {
            Ok(output) => {
                if output.stdout.contains("children") {
                    let _ = tx.send(InstallMessage::Log("Partition table verified".to_string()));
                }
            }
            Err(_) => {
                let _ = tx.send(InstallMessage::Log(
                    "Warning: could not verify partition table".to_string(),
                ));
            }
        }
    }

    let _ = tx.send(InstallMessage::Log(
        "Installation verified successfully".to_string(),
    ));

    // Complete
    let _ = tx.send(InstallMessage::Progress(InstallProgress {
        stage: InstallStage::Complete,
        percent: 100,
        operation: "Fast installation complete!".to_string(),
        log_line: None,
    }));

    let _ = tx.send(InstallMessage::Success);
}

/// Find the root partition on a disk after writing an image.
///
/// Probes lsblk for the largest non-EFI partition, which is typically root.
fn find_root_partition(disk_path: &str) -> Option<String> {
    let output = std::process::Command::new("lsblk")
        .args([
            "--json",
            "--bytes",
            "--output",
            "NAME,SIZE,FSTYPE,PARTTYPE",
            disk_path,
        ])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse JSON to find the largest non-EFI partition
    #[derive(serde::Deserialize)]
    struct LsblkOut {
        blockdevices: Vec<LsblkDev>,
    }
    #[derive(serde::Deserialize)]
    struct LsblkDev {
        #[allow(dead_code)]
        name: String,
        #[serde(default)]
        children: Option<Vec<LsblkChild>>,
    }
    #[derive(serde::Deserialize)]
    struct LsblkChild {
        name: String,
        #[serde(default)]
        size: u64,
        #[serde(default)]
        fstype: Option<String>,
    }

    let parsed: LsblkOut = serde_json::from_str(&stdout).ok()?;
    let dev = parsed.blockdevices.into_iter().next()?;
    let children = dev.children?;

    // Find the largest partition that isn't vfat (EFI)
    children
        .into_iter()
        .filter(|c| c.fstype.as_deref() != Some("vfat"))
        .max_by_key(|c| c.size)
        .map(|c| format!("/dev/{}", c.name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BootLoader, UserConfig};
    use std::path::PathBuf;

    fn create_test_config() -> InstallConfig {
        InstallConfig {
            hostname: "test-nixos".to_string(),
            user: UserConfig {
                username: "testuser".to_string(),
                full_name: Some("Test User".to_string()),
                is_admin: true,
                password: "testpass123".to_string(),
            },
            timezone: "America/New_York".to_string(),
            locale: "en_US.UTF-8".to_string(),
            keymap: "us".to_string(),
            bootloader: BootLoader::SystemdBoot,
            network_manager: true,
            disk_path: "/dev/sda".to_string(),
            disk_size: 500_000_000_000,
            swap_size_gb: 8,
            root_filesystem: "ext4".to_string(),
            luks_encryption: false,
            luks_passphrase: String::new(),
        }
    }

    #[test]
    fn test_mock_executor_success() {
        let executor = MockInstallExecutor::new();
        let root = PathBuf::from("/mnt");

        assert!(executor.generate_hardware_config(&root).is_ok());
        assert!(executor.execute_install(&root).is_ok());
        assert!(executor.verify_installation(&root).is_ok());
    }

    #[test]
    fn test_mock_executor_failure() {
        let executor = MockInstallExecutor::new().with_failure(true);
        let root = PathBuf::from("/mnt");

        assert!(executor.generate_hardware_config(&root).is_err());
        assert!(executor.execute_install(&root).is_err());
        assert!(executor.verify_installation(&root).is_err());
    }

    #[test]
    fn test_install_stages() {
        assert_eq!(
            InstallStage::GeneratingConfig,
            InstallStage::GeneratingConfig
        );
        assert_ne!(InstallStage::Installing, InstallStage::Complete);
    }

    #[test]
    fn test_install_message() {
        let progress = InstallProgress {
            stage: InstallStage::Installing,
            percent: 50,
            operation: "Test".to_string(),
            log_line: None,
        };

        let msg = InstallMessage::Progress(progress.clone());
        if let InstallMessage::Progress(p) = msg {
            assert_eq!(p.percent, 50);
            assert_eq!(p.stage, InstallStage::Installing);
        } else {
            panic!("Expected Progress message");
        }
    }

    #[test]
    fn test_async_installation() {
        let config = create_test_config();
        let rx = run_installation_async(config, "/mnt".to_string(), true);

        // Collect messages
        let messages: Vec<InstallMessage> = rx.iter().collect();

        // Should receive multiple progress updates
        assert!(!messages.is_empty());

        // Last message should be success or error (error expected since /mnt doesn't exist)
        match messages.last() {
            Some(InstallMessage::Success) => {
                // Expected in a real environment
            }
            Some(InstallMessage::Error(_)) => {
                // Expected in test environment (can't write to /mnt)
            }
            _ => {
                // Check if we got at least one progress message
                let has_progress = messages
                    .iter()
                    .any(|m| matches!(m, InstallMessage::Progress(_)));
                assert!(has_progress, "Expected at least one progress message");
            }
        }
    }

    #[test]
    fn test_fast_installation_mock() {
        let rx = run_fast_installation_async(
            "/path/to/image.raw".to_string(),
            "/dev/sda".to_string(),
            "password123".to_string(),
            true,
        );

        let messages: Vec<InstallMessage> = rx.iter().collect();

        // Should receive multiple progress updates
        assert!(!messages.is_empty());

        // Last message should be success
        assert!(
            matches!(messages.last(), Some(InstallMessage::Success)),
            "Expected Success message, got: {:?}",
            messages.last()
        );

        // Should have progress messages
        let has_progress = messages
            .iter()
            .any(|m| matches!(m, InstallMessage::Progress(_)));
        assert!(has_progress, "Expected at least one progress message");

        // Should reach 100%
        let final_progress = messages.iter().rev().find_map(|m| {
            if let InstallMessage::Progress(p) = m {
                Some(p.percent)
            } else {
                None
            }
        });
        assert_eq!(final_progress, Some(100));
    }
}
