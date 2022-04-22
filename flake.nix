{
  description = "Saffron";

  inputs =
    {
      nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;
      # this is not used by flake, but intend to add an entry in flake.lock for shell.nix
      flake-compat = {
        url = github:edolstra/flake-compat;
        flake = false;
      };
      flake-utils.url = github:numtide/flake-utils;
    };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      rec {
        inherit (legacyPackages) devShell;

        legacyPackages = import nixpkgs {
          inherit system;
          config = { };
          overlays = [ (import ./dev-support/nix-overlay) ];
        };
      }
    );
}
