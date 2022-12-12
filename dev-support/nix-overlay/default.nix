final: prev: {
  devShell = final.callPackage ./dev-shell.nix { };
  libpulsar = final.callPackage ./libpulsar { };
}
