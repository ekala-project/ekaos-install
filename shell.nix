{
  pkgs ? import <nixpkgs> { },
}:

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
    cmake

    # GPU/Wayland/X11 dependencies (for gpui)
    vulkan-loader
    wayland
    libxkbcommon
    fontconfig
    freetype
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    xorg.libxcb

    # Development tools
    cargo-watch

    # VM testing
    qemu
  ];

  shellHook = ''
    echo "Ekaos Install Development Environment"
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
