final: prev: with final; {
  dev-shell = callPackage ./dev-shell.nix { };

  ekaos-install = callPackage ./package.nix { };
}
