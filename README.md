# FST Network Rust Common Libraries

[![Build Status](https://github.com/fstnetwork/rust-common-libs/actions/workflows/rust.yaml/badge.svg?branch=main)](https://github.com/fstnetwork/rust-common-libs/actions)

## Crates in this Repository

- [`k8s-structural-schema`](./k8s-structural-schema)
  Kubernetes structural schema utilities for [`schemars`](https://crates.io/crates/schemars)
- [`lifecycle-manager`](./lifecycle-manager)
  a utility for spawning background workers and handling UNIX signals
- [`pulsar-client`](./pulsar-client) Apache Pulsar client library for Rust based on libpulsar
- [`pulsar-client-sys`](./pulsar-client-sys) Native bindings to the libpulsar

## Development

- [`.github`](./.github) contains [GitHub Actions](https://github.com/features/actions) related definition.
- [`dev-support`](./dev-support) contains development utilities
  - [`dev-support/bin`](./dev-support/bin) contains tools which will be used through development process
  - [`dev-support/nix-overlay`](./dev-support/nix-overlay) contains overrides to Nix channel or custom derivations

### Git Hooks

It is suggested to install git hook for style linting before code committing. This project is configured with [pre-commit](https://pre-commit.com).

Installation steps:

```bash
pre-commit install --install-hooks -t commit-msg -t pre-commit
```

### Tools

There are some useful commands just like the one in `Rust` toolchain but with proper arguments:

- `cargo build-all`
- `cargo clippy-all`
- `cargo test-all`
- `cargo doc-all`
- `cargo miri-all`
- `cargo watch-all` (ex. `cargo watch-all clippy`)

Please perform the following steps before submitting Pull Request:

It is suggested to perform the following steps before submitting Pull Request:

- Run [codespell](https://github.com/codespell-project/codespell) to find out common misspellings
- Run [format-check](./dev-support/bin/format-check) to check format of `Rust`, `Shell`, `Nix`, `JavaScript`, `TypeScript`, `Markdown`, `JSON`, `YAML`
- Run [cargo clippy-all](./dev-support/bin/cargo-clippy-all) to lint common mistakes of `Rust`
- Run [cargo test-all](./dev-support/bin/cargo-test-all) to perform tests
- Run [udeps](./dev-support/bin/udeps) to check unused dependencies

Other tools:

- [install-deps](./dev-support/bin/install-deps): Install dependencies
- [format-all](./dev-support/bin/format-all): format all files
- [format-rust](./dev-support/bin/format-rust): format `*.rs` files
- [lines-of-code](./dev-support/bin/lines-of-code): Count lines of code in this project
