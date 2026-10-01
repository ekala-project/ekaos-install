{
  stdenv,
  fenix,
  pkg-config,
  openssl,
  cmake,
  vulkan-loader,
  wayland,
  libxkbcommon,
  fontconfig,
  freetype,
  xorg,
}:

stdenv.mkDerivation {
  name = "dev";

  nativeBuildInputs = [
    (fenix.default.withComponents [
      "cargo"
      "clippy"
      "rust-std"
      "rustc"
      "rustfmt-preview"
    ])
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
}
