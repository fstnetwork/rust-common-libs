{ pkgs, stdenv, mkShell }:

let
  llvmPackages = pkgs.llvmPackages_13;
  clang-tools = pkgs.clang-tools.override { inherit llvmPackages; };
  nodejs = pkgs.nodejs-16_x;
  yarn = pkgs.yarn.override { inherit nodejs; };
in
mkShell {
  buildInputs = with pkgs;
    [
      llvmPackages.clang
      clang-tools
      codespell
      nixpkgs-fmt

      yarn
      nodePackages."@commitlint/cli"
      nodePackages.prettier
      nodePackages.sql-formatter

      convco

      gitAndTools.git-extras
      gitAndTools.pre-commit
      tokei

      rustup
      rust-bindgen
      sccache
      cargo-deny
      cargo-edit
      cargo-tarpaulin
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

      # Helm chart testing
      chart-testing
      kubernetes-helm
      yamale
      yamllint

      # TODO: figure out who use libiconv
      libiconv
    ] ++ lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.SystemConfiguration
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
