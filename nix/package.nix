{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  perl,
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
  ];

  buildInputs = [
  ];

  # This causes the build to occur again, but in debug mode
  doCheck = false;
}
