# AGENTS.md

## Project Overview

ekaos-install is a native GUI application for guided NixOS installation. Built in Rust with gpui (GPU-accelerated UI framework), it walks users through a 7-screen wizard: pre-flight checks, disk selection, partition planning, system configuration, confirmation, installation, and completion.

## Rules

- **No commit attribution** — Do not add `Co-Authored-By:` or similar attribution lines to commits.
- **Tests and lint before committing** — After each edit, run `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` and ensure all pass before committing.
- **Format before committing** — Run `cargo fmt` and `nix fmt .` before committing to ensure consistent formatting across Rust and Nix files.
- **Keep this file current** — If you alter the project structure (add/remove/rename modules or significant files), update this file to reflect the change.

## Hard Constraints

- **MSRV 1.80** (see `Cargo.toml` `rust-version`).
- **No external mocking frameworks.** Tests use the built-in `CommandExecutor` trait with `Real`/`Mock` implementations.
- **CI runs `clippy` with `-D warnings`** — all warnings are errors.
- **rustfmt config** (`rustfmt.toml`): edition 2021, max_width 100, 4-space indentation, Unix line endings.
- **GPU required** — gpui requires a GPU and display server (Wayland/X11 on Linux, Metal on macOS).

## Repository Layout

```
Cargo.toml                          # Package manifest (single crate)
flake.nix                           # Nix flake: dev shell, packages, formatting
flake.lock                          # Nix flake lock
shell.nix                           # Legacy Nix shell (compatibility)
rustfmt.toml                        # Rust formatting configuration
LICENSE                             # GPLv3

nix/
  dev-shell.nix                     # Dev environment (Rust toolchain, GPU libs, etc.)
  package.nix                       # Nix package definition
  overlay.nix                       # Nix overlay for local packages

scripts/
  run-vm.sh                         # VM testing script
  setup-vm.sh                       # VM setup script

src/
  main.rs                           # Entry point: gpui Application, InstallerRoot view, keybindings
  lib.rs                            # Library root, module exports
  error.rs                          # Error types (InstallerError, domain-specific errors)

  app/                              # Application state & wizard flow
    mod.rs
    state.rs                        # Screen enum, App struct, navigation logic

  ui/                               # GUI rendering (gpui-based)
    mod.rs                          # Module exports, gpui actions definition
    layout.rs                       # Header and Footer RenderOnce components
    theme.rs                        # Color palette (Hsla), spacing, icons
    utils/                          # UI utilities
      mod.rs
      input.rs                      # (placeholder — input handled via gpui actions)
      navigation.rs                 # NavigationHints RenderOnce component
    screens/                        # 7 wizard screens (each implements gpui::Render)
      mod.rs                        #   Module declarations and re-exports
      welcome.rs                    #   Pre-flight checks (root, network, ISO, disk space)
      disk_selection.rs             #   Target disk selection
      partition_planning.rs         #   Swap size, partition layout, LUKS encryption
      configuration.rs              #   Hostname, user, timezone, locale, desktop
      confirmation.rs               #   Review before install (type DELETE gate)
      installation.rs               #   Real-time progress tracking
      success.rs                    #   Completion summary
    components/                     # Reusable UI widgets
      mod.rs                        #   Validatable trait, module declarations
      button.rs                     #   Button (RenderOnce)
      checkbox.rs                   #   CheckboxList with view() method
      dialog.rs                     #   ConfirmDialog with overlay
      filterable_select.rs          #   FilterableSelectList with search/filter
      help.rs                       #   HelpPanel overlay
      input.rs                      #   InputField (Render, with FocusHandle)
      message.rs                    #   StatusMessage (RenderOnce)
      progress.rs                   #   ProgressBar (RenderOnce), Spinner
      select.rs                     #   SelectList with view() method

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
  ci.yml                            # CI: test, fmt, clippy, build, security audit
```

## Architecture

### Wizard Flow

The application is a state machine driven by `App` in `app/state.rs`. The `Screen` enum defines the 7 installation steps, and `App` manages transitions with validation.

```
Welcome → DiskSelection → PartitionPlanning → Configuration → Confirmation → Installation → Complete
```

### GUI Framework (gpui)

The UI uses gpui, a GPU-accelerated hybrid immediate/retained mode framework. Key patterns:

- **Render trait**: Stateful views implement `gpui::Render` with `fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement`
- **RenderOnce trait**: Stateless components implement `gpui::RenderOnce` (consumed on render)
- **Element tree**: Built with `div()` and fluent styling methods (Tailwind-like)
- **Layout**: CSS Flexbox via Taffy engine
- **Actions**: Typed event structs dispatched through key bindings (`gpui::actions!` macro)
- **State**: Owned by the root `InstallerRoot` view; screens are plain structs with `view()` methods

### Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `InstallerRoot` | `main.rs` | Root gpui view, owns all screens |
| `App` | `app/state.rs` | Application state machine |
| `Screen` (enum) | `app/state.rs` | 7 wizard screens |
| `InstallConfig` | `config/mod.rs` | Full installation configuration |
| `UserConfig` | `config/mod.rs` | Username, password, admin flag |
| `BootLoader` | `config/mod.rs` | SystemdBoot or Grub |
| `Disk` | `nixos/disk.rs` | Disk info (name, path, size, type) |
| `InstallerError` | `error.rs` | Top-level error wrapper |
| `AppTheme` | `ui/theme.rs` | Color palette (Hsla values) |

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

System libraries (for gpui):

- `vulkan-loader` — GPU rendering
- `wayland` / `libX11` — Display server
- `libxkbcommon` — Keyboard handling
- `fontconfig` / `freetype` — Font rendering

Rust crate dependencies:

- **GUI**: gpui 0.2
- **CLI**: clap 4.5 (derive)
- **Error handling**: anyhow 1.0, thiserror 1.0
- **Logging**: tracing 0.1, tracing-subscriber 0.3
- **Utilities**: regex 1.10, serde 1.0, libc 0.2

## Extension Recipes

### Adding a New Screen

1. Create `src/ui/screens/<name>.rs` with a struct implementing `gpui::Render`
2. Add the variant to `Screen` enum in `app/state.rs`
3. Add navigation transitions in `App`
4. Register the module in `ui/screens/mod.rs`
5. Add the screen to `InstallerRoot` in `main.rs`

### Adding a New UI Component

1. Create `src/ui/components/<name>.rs`
2. Implement `gpui::Render` (stateful) or `gpui::RenderOnce` (stateless)
3. Register the module in `ui/components/mod.rs`

### Adding a New System Detection

1. Add detection function in appropriate `system/` module
2. If it needs command execution, use `CommandExecutor` trait
3. Add mock implementation for testing
