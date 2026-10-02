//! Disk image format detection and writing
//!
//! Supports auto-detecting image formats from magic bytes and dispatching
//! to the appropriate write strategy (dd, qemu-img convert, or decompression pipe).

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::Result;
use anyhow::{bail, Context};
use tracing::debug;

/// Compression format for compressed raw images
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    /// gzip (.gz)
    Gzip,
    /// xz (.xz)
    Xz,
    /// Zstandard (.zst)
    Zstd,
}

impl Compression {
    /// Get the decompression command name
    pub fn decompress_command(&self) -> &'static str {
        match self {
            Compression::Gzip => "gzip",
            Compression::Xz => "xz",
            Compression::Zstd => "zstd",
        }
    }

    /// Get the decompression arguments
    pub fn decompress_args(&self) -> &'static [&'static str] {
        match self {
            Compression::Gzip => &["-dc"],
            Compression::Xz => &["-dc"],
            Compression::Zstd => &["-dc"],
        }
    }

    /// File extension
    pub fn extension(&self) -> &'static str {
        match self {
            Compression::Gzip => ".gz",
            Compression::Xz => ".xz",
            Compression::Zstd => ".zst",
        }
    }
}

impl fmt::Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Compression::Gzip => write!(f, "gzip"),
            Compression::Xz => write!(f, "xz"),
            Compression::Zstd => write!(f, "zstd"),
        }
    }
}

/// Disk image format
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageFormat {
    /// Raw disk image (.raw, .img) — written with dd
    Raw,
    /// QCOW2 image (.qcow2) — converted with qemu-img
    Qcow2,
    /// ISO 9660 image (.iso) — written with dd (hybrid ISOs)
    Iso,
    /// VMDK image (.vmdk) — converted with qemu-img
    Vmdk,
    /// VDI image (.vdi) — converted with qemu-img
    Vdi,
    /// VHD image (.vhd) — converted with qemu-img (format name: vpc)
    Vhd,
    /// VHDX image (.vhdx) — converted with qemu-img
    Vhdx,
    /// Compressed raw image — decompressed and piped to dd
    CompressedRaw(Compression),
}

impl ImageFormat {
    /// Detect the image format from a file by reading its magic bytes,
    /// with extension-based fallback.
    pub fn detect(path: &str) -> Result<Self> {
        // Try magic-byte detection first
        if let Some(format) = Self::detect_from_magic(path)? {
            return Ok(format);
        }

        // Fall back to extension-based detection
        if let Some(format) = Self::detect_from_extension(path) {
            return Ok(format);
        }

        // Default to raw if nothing else matches
        Ok(ImageFormat::Raw)
    }

    /// Detect format from file magic bytes
    fn detect_from_magic(path: &str) -> Result<Option<Self>> {
        let file_path = Path::new(path);
        if !file_path.exists() {
            bail!("Image file not found: {}", path);
        }

        let mut file = File::open(file_path)
            .with_context(|| format!("Failed to open image file: {}", path))?;
        let mut header = [0u8; 65];
        let bytes_read = file
            .read(&mut header)
            .with_context(|| format!("Failed to read image header: {}", path))?;

        if bytes_read < 4 {
            return Ok(None);
        }

        // Check compressed formats first (these wrap another format)
        // gzip: 1F 8B
        if header[0] == 0x1F && header[1] == 0x8B {
            return Ok(Some(ImageFormat::CompressedRaw(Compression::Gzip)));
        }

        // xz: FD 37 7A 58 5A 00
        if bytes_read >= 6
            && header[0] == 0xFD
            && header[1] == 0x37
            && header[2] == 0x7A
            && header[3] == 0x58
            && header[4] == 0x5A
            && header[5] == 0x00
        {
            return Ok(Some(ImageFormat::CompressedRaw(Compression::Xz)));
        }

        // zstd: 28 B5 2F FD
        if header[0] == 0x28 && header[1] == 0xB5 && header[2] == 0x2F && header[3] == 0xFD {
            return Ok(Some(ImageFormat::CompressedRaw(Compression::Zstd)));
        }

        // QCOW2: QFI\xfb (51 46 49 FB)
        if header[0] == 0x51 && header[1] == 0x46 && header[2] == 0x49 && header[3] == 0xFB {
            return Ok(Some(ImageFormat::Qcow2));
        }

        // VMDK sparse: KDMV (4B 44 4D 56)
        if header[0] == 0x4B && header[1] == 0x44 && header[2] == 0x4D && header[3] == 0x56 {
            return Ok(Some(ImageFormat::Vmdk));
        }

        // VHD: "conectix" at offset 0
        if bytes_read >= 8 && &header[0..8] == b"conectix" {
            return Ok(Some(ImageFormat::Vhd));
        }

        // VHDX: "vhdxfile" at offset 0
        if bytes_read >= 8 && &header[0..8] == b"vhdxfile" {
            return Ok(Some(ImageFormat::Vhdx));
        }

        // VDI: "<<< Oracle VM VirtualBox Disk Image >>>" near offset 0
        if bytes_read >= 40 {
            let header_str = String::from_utf8_lossy(&header[0..40]);
            if header_str.contains("Oracle VM VirtualBox Disk Image") {
                return Ok(Some(ImageFormat::Vdi));
            }
        }

        // VMDK descriptor file: starts with "# Disk DescriptorFile"
        if bytes_read >= 21 {
            let header_str = String::from_utf8_lossy(&header[0..21]);
            if header_str.starts_with("# Disk DescriptorFile") {
                return Ok(Some(ImageFormat::Vmdk));
            }
        }

        Ok(None)
    }

    /// Detect format from file extension
    fn detect_from_extension(path: &str) -> Option<Self> {
        let lower = path.to_lowercase();

        // Check compressed extensions first (double extensions)
        if lower.ends_with(".raw.gz") || lower.ends_with(".img.gz") {
            return Some(ImageFormat::CompressedRaw(Compression::Gzip));
        }
        if lower.ends_with(".raw.xz") || lower.ends_with(".img.xz") {
            return Some(ImageFormat::CompressedRaw(Compression::Xz));
        }
        if lower.ends_with(".raw.zst") || lower.ends_with(".img.zst") {
            return Some(ImageFormat::CompressedRaw(Compression::Zstd));
        }

        // Single extensions
        if lower.ends_with(".qcow2") || lower.ends_with(".qcow") {
            return Some(ImageFormat::Qcow2);
        }
        if lower.ends_with(".vmdk") {
            return Some(ImageFormat::Vmdk);
        }
        if lower.ends_with(".vdi") {
            return Some(ImageFormat::Vdi);
        }
        if lower.ends_with(".vhd") || lower.ends_with(".vpc") {
            return Some(ImageFormat::Vhd);
        }
        if lower.ends_with(".vhdx") {
            return Some(ImageFormat::Vhdx);
        }
        if lower.ends_with(".iso") {
            return Some(ImageFormat::Iso);
        }
        if lower.ends_with(".raw") || lower.ends_with(".img") || lower.ends_with(".bin") {
            return Some(ImageFormat::Raw);
        }

        None
    }

    /// Detect format from extension only, defaulting to Raw.
    ///
    /// Useful when the file may not exist (e.g., mock mode).
    pub fn detect_from_extension_or_raw(path: &str) -> Self {
        Self::detect_from_extension(path).unwrap_or(ImageFormat::Raw)
    }

    /// Get the qemu-img format name (for formats that use qemu-img)
    pub fn qemu_format_name(&self) -> Option<&'static str> {
        match self {
            ImageFormat::Qcow2 => Some("qcow2"),
            ImageFormat::Vmdk => Some("vmdk"),
            ImageFormat::Vdi => Some("vdi"),
            ImageFormat::Vhd => Some("vpc"), // Note: VHD uses "vpc" in qemu-img
            ImageFormat::Vhdx => Some("vhdx"),
            _ => None,
        }
    }

    /// Whether this format requires qemu-img to write
    pub fn needs_qemu_img(&self) -> bool {
        self.qemu_format_name().is_some()
    }

    /// Human-readable format name
    pub fn display_name(&self) -> &str {
        match self {
            ImageFormat::Raw => "Raw disk image",
            ImageFormat::Qcow2 => "QCOW2 (QEMU)",
            ImageFormat::Iso => "ISO 9660",
            ImageFormat::Vmdk => "VMDK (VMware)",
            ImageFormat::Vdi => "VDI (VirtualBox)",
            ImageFormat::Vhd => "VHD (Hyper-V)",
            ImageFormat::Vhdx => "VHDX (Hyper-V)",
            ImageFormat::CompressedRaw(c) => match c {
                Compression::Gzip => "Compressed raw (gzip)",
                Compression::Xz => "Compressed raw (xz)",
                Compression::Zstd => "Compressed raw (zstd)",
            },
        }
    }
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Write a disk image to a target block device.
///
/// Dispatches to the appropriate write strategy based on image format.
/// Returns Ok(()) on success, or an error with details.
pub fn write_image(
    image_path: &str,
    disk_path: &str,
    format: &ImageFormat,
) -> std::result::Result<String, String> {
    debug!(
        "Writing image {} ({}) to {}",
        image_path,
        format.display_name(),
        disk_path
    );

    match format {
        ImageFormat::Raw | ImageFormat::Iso => write_with_dd(image_path, disk_path),
        ImageFormat::Qcow2
        | ImageFormat::Vmdk
        | ImageFormat::Vdi
        | ImageFormat::Vhd
        | ImageFormat::Vhdx => {
            let qemu_fmt = format.qemu_format_name().unwrap();
            write_with_qemu_img(image_path, disk_path, qemu_fmt)
        }
        ImageFormat::CompressedRaw(compression) => {
            write_compressed_raw(image_path, disk_path, *compression)
        }
    }
}

/// Write a raw/ISO image with dd
fn write_with_dd(image_path: &str, disk_path: &str) -> std::result::Result<String, String> {
    let output = Command::new("dd")
        .args([
            &format!("if={}", image_path),
            &format!("of={}", disk_path),
            "bs=4M",
            "conv=fsync",
            "status=progress",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute dd: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("dd failed: {}", stderr));
    }

    // dd prints stats to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(stderr.to_string())
}

/// Write an image using qemu-img convert
fn write_with_qemu_img(
    image_path: &str,
    disk_path: &str,
    input_format: &str,
) -> std::result::Result<String, String> {
    let output = Command::new("qemu-img")
        .args([
            "convert",
            "-f",
            input_format,
            "-O",
            "raw",
            "-p",
            image_path,
            disk_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute qemu-img: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("qemu-img convert failed: {}", stderr));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(stderr.to_string())
}

/// Write a compressed raw image by piping decompression into dd
fn write_compressed_raw(
    image_path: &str,
    disk_path: &str,
    compression: Compression,
) -> std::result::Result<String, String> {
    // Spawn the decompressor
    let decompress = Command::new(compression.decompress_command())
        .args(compression.decompress_args())
        .arg(image_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!(
                "Failed to execute {}: {}",
                compression.decompress_command(),
                e
            )
        })?;

    let decompress_stdout = decompress
        .stdout
        .ok_or("Failed to capture decompressor stdout")?;

    // Pipe into dd
    let dd_output = Command::new("dd")
        .args([
            &format!("of={}", disk_path),
            "bs=4M",
            "iflag=fullblock",
            "conv=fsync",
            "status=progress",
        ])
        .stdin(decompress_stdout)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute dd: {}", e))?;

    if !dd_output.status.success() {
        let stderr = String::from_utf8_lossy(&dd_output.stderr);
        return Err(format!(
            "Decompression pipeline failed ({}→dd): {}",
            compression, stderr
        ));
    }

    let stderr = String::from_utf8_lossy(&dd_output.stderr);
    Ok(stderr.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Helper: write magic bytes to a temp file and detect format
    fn detect_magic(magic: &[u8], extension: &str) -> ImageFormat {
        let dir = std::env::temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("test_image{}", extension));
        let mut file = File::create(&path).unwrap();
        // Write magic bytes padded to at least 65 bytes
        file.write_all(magic).unwrap();
        if magic.len() < 65 {
            file.write_all(&vec![0u8; 65 - magic.len()]).unwrap();
        }
        file.flush().unwrap();

        let result = ImageFormat::detect(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();
        result
    }

    #[test]
    fn test_detect_qcow2() {
        let format = detect_magic(&[0x51, 0x46, 0x49, 0xFB], ".qcow2");
        assert_eq!(format, ImageFormat::Qcow2);
    }

    #[test]
    fn test_detect_gzip() {
        let format = detect_magic(&[0x1F, 0x8B, 0x08, 0x00], ".raw.gz");
        assert_eq!(format, ImageFormat::CompressedRaw(Compression::Gzip));
    }

    #[test]
    fn test_detect_xz() {
        let format = detect_magic(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00], ".raw.xz");
        assert_eq!(format, ImageFormat::CompressedRaw(Compression::Xz));
    }

    #[test]
    fn test_detect_zstd() {
        let format = detect_magic(&[0x28, 0xB5, 0x2F, 0xFD], ".raw.zst");
        assert_eq!(format, ImageFormat::CompressedRaw(Compression::Zstd));
    }

    #[test]
    fn test_detect_vmdk() {
        let format = detect_magic(&[0x4B, 0x44, 0x4D, 0x56], ".vmdk");
        assert_eq!(format, ImageFormat::Vmdk);
    }

    #[test]
    fn test_detect_vhd() {
        let format = detect_magic(b"conectix", ".vhd");
        assert_eq!(format, ImageFormat::Vhd);
    }

    #[test]
    fn test_detect_vhdx() {
        let format = detect_magic(b"vhdxfile", ".vhdx");
        assert_eq!(format, ImageFormat::Vhdx);
    }

    #[test]
    fn test_detect_vdi() {
        let magic = b"<<< Oracle VM VirtualBox Disk Image >>>\n";
        let format = detect_magic(magic, ".vdi");
        assert_eq!(format, ImageFormat::Vdi);
    }

    #[test]
    fn test_detect_vmdk_descriptor() {
        let magic = b"# Disk DescriptorFile\n";
        let format = detect_magic(magic, ".vmdk");
        assert_eq!(format, ImageFormat::Vmdk);
    }

    #[test]
    fn test_detect_raw_by_extension() {
        // Create a file with no recognizable magic
        let format = detect_magic(&[0x00; 65], ".raw");
        assert_eq!(format, ImageFormat::Raw);
    }

    #[test]
    fn test_detect_img_by_extension() {
        let format = detect_magic(&[0x00; 65], ".img");
        assert_eq!(format, ImageFormat::Raw);
    }

    #[test]
    fn test_detect_iso_by_extension() {
        let format = detect_magic(&[0x00; 65], ".iso");
        assert_eq!(format, ImageFormat::Iso);
    }

    #[test]
    fn test_detect_unknown_defaults_to_raw() {
        let format = detect_magic(&[0x00; 65], ".mystery");
        assert_eq!(format, ImageFormat::Raw);
    }

    #[test]
    fn test_extension_detection_only() {
        // Test the extension-only path
        assert_eq!(
            ImageFormat::detect_from_extension("image.qcow2"),
            Some(ImageFormat::Qcow2)
        );
        assert_eq!(
            ImageFormat::detect_from_extension("image.vmdk"),
            Some(ImageFormat::Vmdk)
        );
        assert_eq!(
            ImageFormat::detect_from_extension("image.vdi"),
            Some(ImageFormat::Vdi)
        );
        assert_eq!(
            ImageFormat::detect_from_extension("image.vhd"),
            Some(ImageFormat::Vhd)
        );
        assert_eq!(
            ImageFormat::detect_from_extension("image.vhdx"),
            Some(ImageFormat::Vhdx)
        );
        assert_eq!(
            ImageFormat::detect_from_extension("image.raw.gz"),
            Some(ImageFormat::CompressedRaw(Compression::Gzip))
        );
        assert_eq!(
            ImageFormat::detect_from_extension("image.raw.xz"),
            Some(ImageFormat::CompressedRaw(Compression::Xz))
        );
        assert_eq!(
            ImageFormat::detect_from_extension("image.raw.zst"),
            Some(ImageFormat::CompressedRaw(Compression::Zstd))
        );
        assert_eq!(
            ImageFormat::detect_from_extension("image.img.xz"),
            Some(ImageFormat::CompressedRaw(Compression::Xz))
        );
    }

    #[test]
    fn test_qemu_format_name() {
        assert_eq!(ImageFormat::Qcow2.qemu_format_name(), Some("qcow2"));
        assert_eq!(ImageFormat::Vmdk.qemu_format_name(), Some("vmdk"));
        assert_eq!(ImageFormat::Vdi.qemu_format_name(), Some("vdi"));
        assert_eq!(ImageFormat::Vhd.qemu_format_name(), Some("vpc"));
        assert_eq!(ImageFormat::Vhdx.qemu_format_name(), Some("vhdx"));
        assert_eq!(ImageFormat::Raw.qemu_format_name(), None);
        assert_eq!(ImageFormat::Iso.qemu_format_name(), None);
        assert_eq!(
            ImageFormat::CompressedRaw(Compression::Gzip).qemu_format_name(),
            None
        );
    }

    #[test]
    fn test_needs_qemu_img() {
        assert!(ImageFormat::Qcow2.needs_qemu_img());
        assert!(ImageFormat::Vmdk.needs_qemu_img());
        assert!(ImageFormat::Vdi.needs_qemu_img());
        assert!(ImageFormat::Vhd.needs_qemu_img());
        assert!(ImageFormat::Vhdx.needs_qemu_img());
        assert!(!ImageFormat::Raw.needs_qemu_img());
        assert!(!ImageFormat::Iso.needs_qemu_img());
        assert!(!ImageFormat::CompressedRaw(Compression::Xz).needs_qemu_img());
    }

    #[test]
    fn test_display_names() {
        assert_eq!(ImageFormat::Raw.display_name(), "Raw disk image");
        assert_eq!(ImageFormat::Qcow2.display_name(), "QCOW2 (QEMU)");
        assert_eq!(ImageFormat::Iso.display_name(), "ISO 9660");
        assert_eq!(ImageFormat::Vmdk.display_name(), "VMDK (VMware)");
        assert_eq!(ImageFormat::Vdi.display_name(), "VDI (VirtualBox)");
        assert_eq!(ImageFormat::Vhd.display_name(), "VHD (Hyper-V)");
        assert_eq!(ImageFormat::Vhdx.display_name(), "VHDX (Hyper-V)");
        assert_eq!(
            ImageFormat::CompressedRaw(Compression::Gzip).display_name(),
            "Compressed raw (gzip)"
        );
        assert_eq!(
            ImageFormat::CompressedRaw(Compression::Xz).display_name(),
            "Compressed raw (xz)"
        );
        assert_eq!(
            ImageFormat::CompressedRaw(Compression::Zstd).display_name(),
            "Compressed raw (zstd)"
        );
    }

    #[test]
    fn test_display_trait() {
        assert_eq!(format!("{}", ImageFormat::Qcow2), "QCOW2 (QEMU)");
        assert_eq!(
            format!("{}", ImageFormat::CompressedRaw(Compression::Zstd)),
            "Compressed raw (zstd)"
        );
    }

    #[test]
    fn test_compression_display() {
        assert_eq!(format!("{}", Compression::Gzip), "gzip");
        assert_eq!(format!("{}", Compression::Xz), "xz");
        assert_eq!(format!("{}", Compression::Zstd), "zstd");
    }

    #[test]
    fn test_compression_commands() {
        assert_eq!(Compression::Gzip.decompress_command(), "gzip");
        assert_eq!(Compression::Xz.decompress_command(), "xz");
        assert_eq!(Compression::Zstd.decompress_command(), "zstd");

        for c in [Compression::Gzip, Compression::Xz, Compression::Zstd] {
            assert_eq!(c.decompress_args(), &["-dc"]);
        }
    }

    #[test]
    fn test_detect_nonexistent_file() {
        let result = ImageFormat::detect("/nonexistent/path/image.qcow2");
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive_extension() {
        assert_eq!(
            ImageFormat::detect_from_extension("image.QCOW2"),
            Some(ImageFormat::Qcow2)
        );
        assert_eq!(
            ImageFormat::detect_from_extension("image.RAW.GZ"),
            Some(ImageFormat::CompressedRaw(Compression::Gzip))
        );
        assert_eq!(
            ImageFormat::detect_from_extension("image.Vmdk"),
            Some(ImageFormat::Vmdk)
        );
    }
}
