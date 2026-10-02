{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  perl,
  cmake,
  vulkan-loader,
  wayland,
  libxkbcommon,
  fontconfig,
  freetype,
  xorg,
}:

rustPlatform.buildRustPackage {
  pname = "ekaos-install";
  version =
    let
      cargo_toml = builtins.readFile ../Cargo.toml;
      cargo_info = builtins.fromTOML cargo_toml;
    in
    cargo_info.package.version;

  cargoLock.lockFile = ../Cargo.lock;
  src = ../.;

  nativeBuildInputs = [
    perl
    pkg-config
    cmake
  ];

  buildInputs = [
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
  ];

  # This causes the build to occur again, but in debug mode
  doCheck = false;
}
