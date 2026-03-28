{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rustfmt
    clippy

    # Build dependencies
    pkg-config
    openssl
    gcc

    # Development tools
    cargo-watch

    # VM testing
    qemu

    # Terminal support
    ncurses
  ];

  shellHook = ''
    echo "🚀 Ekaos Install Development Environment"
    echo ""
    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    echo ""
    echo "Available commands:"
    echo "  cargo build        - Build the project"
    echo "  cargo run -- --mock - Run in mock mode"
    echo "  cargo test         - Run tests"
    echo "  cargo clippy       - Run linter"
    echo "  cargo fmt          - Format code"
    echo ""
  '';
}
