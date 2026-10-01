# AGENTS.md

## Project Overview

ekaos-install is a terminal user interface (TUI) for guided NixOS installation. Built in Rust with ratatui, it walks users through a 7-screen wizard: pre-flight checks, boot mode selection, disk selection, partition planning, system configuration, installation, and completion.

## Rules

- **No commit attribution** — Do not add `Co-Authored-By:` or similar attribution lines to commits.
- **Tests and lint before committing** — After each edit, run `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` and ensure all pass before committing.
- **Format before committing** — Run `cargo fmt` and `nix fmt .` before committing to ensure consistent formatting across Rust and Nix files.
- **Keep this file current** — If you alter the project structure (add/remove/rename modules or significant files), update this file to reflect the change.

## Hard Constraints

- **MSRV 1.74** (see `Cargo.toml` `rust-version`).
- **No external mocking frameworks.** Tests use the built-in `CommandExecutor` trait with `Real`/`Mock` implementations.
- **CI runs `clippy` with `-D warnings`** — all warnings are errors.
- **rustfmt config** (`rustfmt.toml`): edition 2021, max_width 100, 4-space indentation, Unix line endings.

## Repository Layout

```
Cargo.toml                          # Package manifest (single crate)
flake.nix                           # Nix flake: dev shell, packages, formatting
flake.lock                          # Nix flake lock
shell.nix                           # Legacy Nix shell (compatibility)
rustfmt.toml                        # Rust formatting configuration
LICENSE                             # GPLv3

nix/
  dev-shell.nix                     # Dev environment (Rust toolchain, qemu, etc.)
  package.nix                       # Nix package definition
  overlay.nix                       # Nix overlay for local packages

scripts/
  run-vm.sh                         # VM testing script
  setup-vm.sh                       # VM setup script

src/
  main.rs                           # Entry point: terminal setup, event loop, cleanup
  lib.rs                            # Library root, module exports
  error.rs                          # Error types (InstallerError, domain-specific errors)

  app/                              # Application state & wizard flow
    mod.rs
    state.rs                        # Screen enum, App struct, navigation logic

  ui/                               # TUI rendering
    mod.rs
    layout.rs                       # Layout system
    theme.rs                        # Color/styling configuration
    utils/                          # UI utilities (input handling, navigation)
    screens/                        # 7 wizard screens
      mod.rs                        #   Screen trait definition
      welcome.rs                    #   Pre-flight checks (root, network, ISO, disk space)
      disk_selection.rs             #   Target disk selection
      partition_planning.rs         #   Swap size, partition layout
      configuration.rs              #   Hostname, user, timezone, locale, desktop
      confirmation.rs               #   Review before install
      installation.rs               #   Real-time progress tracking
      success.rs                    #   Completion summary
    components/                     # Reusable UI widgets
      mod.rs                        #   Component/Focusable/Validatable/Interactive traits
      button.rs
      checkbox.rs
      dialog.rs
      filterable_select.rs
      help.rs
      input.rs
      message.rs
      progress.rs
      select.rs

  system/                           # System detection & command execution
    bootmode.rs                     # UEFI/BIOS detection
    command.rs                      # CommandExecutor trait (Real/Mock implementations)
    detection.rs                    # CPU, RAM, architecture detection
    network.rs                      # Network connectivity checks

  nixos/                            # NixOS-specific operations
    config.rs                       # NixOS configuration generation
    disk.rs                         # Disk detection & parsing
    install.rs                      # Installation execution (async)

  config/                           # Configuration data model
    mod.rs                          # InstallConfig, UserConfig, BootLoader enums

  data/                             # Static data
    keymaps.rs                      # Keyboard layout data
    locales.rs                      # Locale data
    timezones.rs                    # Timezone data

tests/
  integration/                      # Integration tests
    mod.rs

.github/workflows/
  ci.yml                            # CI: test, fmt, clippy, build (ubuntu+macos), security audit
```

## Architecture

### Wizard Flow

The application is a state machine driven by `App` in `app/state.rs`. The `Screen` enum defines the 7 installation steps, and `App` manages transitions with validation and lifecycle hooks (`on_enter`/`on_exit`).

```
Welcome → DiskSelection → PartitionPlanning → Configuration → Confirmation → Installation → Complete
```

### Key Traits

| Trait | Location | Purpose |
|-------|----------|---------|
| `Screen` | `ui/screens/mod.rs` | Main wizard screen interface |
| `Component` | `ui/components/mod.rs` | Base rendering interface |
| `Focusable` | `ui/components/mod.rs` | Components that can receive focus |
| `Validatable` | `ui/components/mod.rs` | Components with validation logic |
| `Interactive` | `ui/components/mod.rs` | Keyboard input handling |
| `CommandExecutor` | `system/command.rs` | Abstracted command execution (Real/Mock) |

### Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `App` | `app/state.rs` | Application state machine |
| `Screen` (enum) | `app/state.rs` | 7 wizard screens |
| `InstallConfig` | `config/mod.rs` | Full installation configuration |
| `UserConfig` | `config/mod.rs` | Username, password, admin flag |
| `BootLoader` | `config/mod.rs` | SystemdBoot or Grub |
| `Disk` | `nixos/disk.rs` | Disk info (name, path, size, type) |
| `DiskType` | `nixos/disk.rs` | Disk/Ssd/Nvme/Loop/Rom/Unknown |
| `InstallerError` | `error.rs` | Top-level error wrapper |
| `TerminalGuard` | `main.rs` | RAII terminal state cleanup |

### Mock Mode

The `CommandExecutor` trait abstracts all system commands behind `Real` and `Mock` implementations, allowing full UI testing without root access or real disks. The `App` struct tracks `AppMode::Mock` vs `AppMode::Real`.

## Testing

- **Unit tests**: `#[test]` with inline assertions, co-located in source files across `system/`, `ui/`, `nixos/`, `config/`, `app/`.
- **Integration tests**: `tests/integration/mod.rs`.
- **Run**: `cargo test` (CI also runs `cargo test --doc --verbose`).
- **Sequential**: Use `--test-threads=1` if tests interfere via environment variables.

## External Dependencies

Runtime tools (provided by Nix dev shell):

- `nix` — NixOS installation commands
- `nixfmt` — Nix file formatting
- `git` — Version control
- `qemu` — VM testing (development only)

Rust crate dependencies:

- **TUI**: ratatui 0.26, crossterm 0.27
- **CLI**: clap 4.5 (derive)
- **Error handling**: anyhow 1.0, thiserror 1.0
- **Logging**: tracing 0.1, tracing-subscriber 0.3
- **Utilities**: regex 1.10, serde 1.0, libc 0.2

## Extension Recipes

### Adding a New Screen

1. Create `src/ui/screens/<name>.rs` implementing the `Screen` trait
2. Add the variant to `Screen` enum in `app/state.rs`
3. Add navigation transitions in `App`
4. Register the module in `ui/screens/mod.rs`

### Adding a New UI Component

1. Create `src/ui/components/<name>.rs`
2. Implement `Component` and optionally `Focusable`, `Validatable`, `Interactive`
3. Register the module in `ui/components/mod.rs`

### Adding a New System Detection

1. Add detection function in appropriate `system/` module
2. If it needs command execution, use `CommandExecutor` trait
3. Add mock implementation for testing
