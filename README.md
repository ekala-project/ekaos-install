# Ekaos Install - NixOS Installation TUI

A modern, beginner-friendly terminal user interface (TUI) for installing NixOS. Built with Rust and [ratatui](https://github.com/ratatui-org/ratatui).

> **Status**: v0.1.0 (MVP) - Feature Complete
>
> The installer now has all core functionality for basic NixOS installations! Phases 0-6 complete. See [roadmap.md](roadmap.md) for the complete development plan.

## Features

- **Beginner-Friendly**: Safe defaults and clear guidance at every step
- **Interactive TUI**: Modern terminal interface with keyboard navigation
- **Safety-First**: Multiple confirmations before destructive operations
- **Mock Mode**: Test the full UI without system privileges
- **Flakes Support**: Generate both traditional and flakes-based configurations
- **Progress Tracking**: Visual progress indicators throughout installation

## Installation Workflow

The installer guides you through 7 comprehensive screens:

1. **Welcome Screen** - Pre-flight checks (root access, network, NixOS ISO, disk space)
2. **Boot Mode Screen** - Select UEFI (systemd-boot) or BIOS (GRUB) bootloader
3. **Disk Selection Screen** - Choose target disk with size and model information
4. **Partition Planning Screen** - Configure swap size and review partition layout
5. **Configuration Screen** - Set hostname, username, password, timezone, locale, and desktop
6. **Installation Screen** - Real-time progress tracking with detailed logs
7. **Success Screen** - Installation summary and next steps

## Completed Features (v0.1.0)

✅ TUI skeleton with ratatui
✅ CLI argument parsing with clap
✅ Error handling framework
✅ Mock mode for testing
✅ VM testing infrastructure
✅ Screen trait system
✅ Component architecture
✅ Input handling
✅ Theme system
✅ CPU detection
✅ RAM detection
✅ Architecture detection
✅ Root privilege checking
✅ NixOS environment validation
✅ Disk detection and selection
✅ Partition layout planning
✅ Swap configuration
✅ Disk information display
✅ Hostname configuration
✅ User account setup
✅ Password validation
✅ Timezone selection
✅ Locale configuration
✅ Desktop environment options
✅ NixOS configuration generation
✅ Hardware configuration generation
✅ Asynchronous installation
✅ Real-time progress tracking
✅ Installation verification
✅ Success screen with next steps

## Installation

### Prerequisites

- Rust 1.74 or later
- For VM testing: QEMU

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/ekaos-install.git
cd ekaos-install

# Build the project
cargo build --release

# The binary will be in target/release/ekaos-install
```

## Usage

### Mock Mode (No Root Required)

Test the UI without making any system changes:

```bash
cargo run -- --mock
```

### Real Mode (Requires Root)

Run the actual installer:

```bash
sudo cargo run
```

**Warning**: Real mode will make actual changes to your system. Always test in a VM or use mock mode first!

### Command-Line Options

```
Options:
  -m, --mock              Run in mock mode (no actual system operations)
      --dry-run           Show what would be done without executing
  -v, --verbose           Increase logging verbosity
  -c, --config <FILE>     Load configuration from file
  -h, --help              Print help
  -V, --version           Print version
```

### Keyboard Navigation

- **Enter**: Proceed to next screen / Confirm selection
- **←** / **Backspace**: Go back to previous screen
- **→**: Go to next screen (when validation passes)
- **↑** / **↓**: Navigate lists and menus
- **Tab**: Cycle through input fields and buttons
- **?**: Toggle help panel
- **q** / **Esc**: Quit application
- **Ctrl+C**: Force exit

### Screen-Specific Controls

**Disk Selection Screen**:
- ↑/↓: Navigate disk list
- Enter: Select disk and proceed

**Partition Planning Screen**:
- Tab: Switch between swap size input and proceed button
- Type numbers for swap size in GB

**Configuration Screen**:
- Tab: Cycle through all input fields (hostname, username, password, etc.)
- Type to enter values
- Select from dropdowns for timezone, locale, and desktop

## Development

### Running Tests

```bash
# Run all tests (118 tests total)
cargo test -- --test-threads=1

# Run tests with output
cargo test -- --nocapture --test-threads=1

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run specific test
cargo test test_name
```

**Note**: Use `--test-threads=1` to avoid test interference with environment variables in mock mode.

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check formatting without modifying files
cargo fmt -- --check
```

### VM Testing

Set up the VM testing environment:

```bash
./scripts/setup-vm.sh
```

Run a test VM:

```bash
# Set the NixOS ISO path
export NIXOS_ISO=/path/to/nixos.iso

# Launch VM
./scripts/run-vm.sh

# Clean up VM disk image
./scripts/run-vm.sh clean
```

## Project Structure

```
ekaos-install/
├── src/
│   ├── main.rs          # Entry point and TUI event loop
│   ├── lib.rs           # Library root
│   ├── error.rs         # Error types
│   ├── app/             # Application state management
│   ├── ui/              # TUI components and screens
│   ├── system/          # System command execution
│   └── nixos/           # NixOS-specific operations
├── tests/
│   └── integration/     # Integration tests
├── scripts/
│   ├── run-vm.sh        # VM testing script
│   └── setup-vm.sh      # VM setup script
├── .github/
│   └── workflows/       # CI/CD configuration
└── roadmap.md           # Development roadmap
```

## Architecture

The application follows a clean architecture with separation of concerns:

- **UI Layer** (`ui/`): Ratatui-based interface, handles rendering and input
- **Application Layer** (`app/`): State machine and wizard flow control
- **Business Logic** (`nixos/`, `system/`): NixOS operations and system commands
- **Error Handling** (`error.rs`): Comprehensive error types with user-friendly messages

### Key Design Principles

1. **Separation of Concerns**: UI never directly executes system commands
2. **Mock Support**: All system operations can be mocked for testing
3. **Safety**: Multiple validations and confirmations before destructive operations
4. **Testability**: Modular design enables comprehensive testing

## Development Roadmap

See [roadmap.md](roadmap.md) for the complete multi-phase development plan.

**Status**: ✅ Phases 0-6 Complete (v0.1.0 MVP)

**Next Phase**: Phase 7 - Polish & Release (Documentation, error handling improvements, final testing)

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) (coming soon) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting (`cargo test && cargo clippy`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Acknowledgments

- [NixOS](https://nixos.org) for the amazing operating system
- [ratatui](https://github.com/ratatui-org/ratatui) for the TUI framework
- The Rust community for excellent tools and libraries

## Related Projects

- [NixOS Installation Manual](https://nixos.org/manual/nixos/stable/#sec-installation)
- [nixos-anywhere](https://github.com/nix-community/nixos-anywhere) - Remote NixOS installation
- [nixos-infect](https://github.com/elitak/nixos-infect) - Convert existing Linux to NixOS

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/ekaos-install/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/ekaos-install/discussions)
- **Documentation**: See [roadmap.md](roadmap.md) for detailed project plan

## Troubleshooting

### Pre-flight Check Failures

**Root Access Failed**:
- Run with `sudo` or use `--mock` flag for testing
- In mock mode: `cargo run -- --mock`

**Network Check Failed**:
- Verify network connectivity: `ping nixos.org`
- Check DNS resolution: `nslookup nixos.org`
- The installer requires internet to download packages

**NixOS ISO Check Failed**:
- This installer must run from NixOS installation media
- Download the latest ISO from https://nixos.org/download
- Use mock mode for testing on non-NixOS systems

**Disk Space Insufficient**:
- Ensure target disk has at least 10GB free space
- Consider reducing swap size in partition planning

### Installation Issues

**Installation Fails**:
- Check the installation logs in the UI
- Verify disk is not mounted: `umount -R /mnt`
- Ensure network is stable during installation
- Try running in mock mode first to verify configuration

**Configuration Errors**:
- Ensure hostname contains only valid characters (alphanumeric, hyphens)
- Username must be lowercase and start with a letter
- Password must be at least 8 characters

---

**Note**: This installer performs real system operations. Always test in VMs first. The mock mode allows safe testing of the complete UI flow without making any system changes.
