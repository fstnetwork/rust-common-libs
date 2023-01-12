{ pkgs, stdenv, mkShell }:

let
  llvmPackages = pkgs.llvmPackages_13;
  clang-tools = pkgs.clang-tools.override { inherit llvmPackages; };
in
mkShell {
  buildInputs = with pkgs;
    [
      llvmPackages.clang
      clang-tools
      codespell
      nixpkgs-fmt

      nodePackages."@commitlint/cli"
      nodePackages.prettier

      convco

      gitAndTools.git-extras
      gitAndTools.pre-commit
      tokei

      rustup
      rust-bindgen
      sccache
      cargo-deny
      cargo-edit
      cargo-udeps

      cmake
      pkg-config

      libpulsar
      openssl.dev
      protobuf

      # shell
      checkbashisms
      shellcheck
      shfmt

      # TODO: figure out who use libiconv
      libiconv
    ] ++ lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.SystemConfiguration
    ] ++ lib.optionals (stdenv.isx86_64 && stdenv.isLinux) [
      # Officially cargo-tarpaulin only supports x86_64-linux (ref: https://github.com/NixOS/nixpkgs/pull/173049)
      cargo-tarpaulin
    ];

  shellHook = ''
    export PATH=$PWD/dev-support/bin:$PWD/target/release:$PWD/target/debug:$PATH
  '';

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

  RUST_BACKTRACE = 1;

  COMMITLINT_PRESET = "${
  pkgs.nodePackages."@commitlint/config-conventional"
  }/lib/node_modules/@commitlint/config-conventional";
}
