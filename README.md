# Ekaos Install

A terminal user interface (TUI) application that guides users through installing NixOS, following the [official installation guide](https://nixos.org/manual/nixos/stable/#sec-installation).

> **Status**: Phase 0 (Foundation) - In Development
>
> This project is in early development. Core infrastructure is in place, but many features are not yet implemented. See [roadmap.md](roadmap.md) for the complete development plan.

## Features

- **Beginner-Friendly**: Safe defaults and clear guidance at every step
- **Interactive TUI**: Modern terminal interface with keyboard navigation
- **Safety-First**: Multiple confirmations before destructive operations
- **Mock Mode**: Test the full UI without system privileges
- **Flakes Support**: Generate both traditional and flakes-based configurations
- **Progress Tracking**: Visual progress indicators throughout installation

## Current Status (Phase 0)

✅ **Completed**:
- Basic TUI skeleton with ratatui
- CLI argument parsing with clap
- Error handling framework
- Mock mode for testing
- VM testing infrastructure
- GitHub Actions CI

🚧 **In Progress**:
- Screen implementations (Welcome screen only)
- System detection (planned for Phase 2)
- Disk operations (planned for Phase 4)
- Installation execution (planned for Phase 5)

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

Run the actual installer (when fully implemented):

```bash
sudo cargo run
```

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

- **Enter**: Proceed to next screen / Confirm
- **←** / **Backspace**: Go back to previous screen
- **→**: Go to next screen
- **?**: Toggle help panel (coming soon)
- **q** / **Esc**: Quit application
- **Ctrl+C**: Force exit

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run only integration tests
cargo test --test '*'
```

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

**Current Phase**: Phase 0 - Foundation & Infrastructure (Weeks 1-2)

**Next Phase**: Phase 1 - Core UI Framework (Weeks 3-4)

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

---

**Note**: This project is in active development. The installer is not yet functional for actual NixOS installation. Use at your own risk and always test in VMs first.
