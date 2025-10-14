# slint-bundler

A Rust CLI tool to build and bundle Rust projects for multiple platforms.

## Features
- `build` subcommand: Compiles the project in release mode and bundles it for:
  - Windows: MSI, NSIS
  - Linux: .deb, .rpm, .tar.zst, .tar.xz (Arch), standalone
  - Use `--bundles` to specify which bundles to create (e.g. `--bundles deb rpm`).
  - Defaults to all supported bundles for the OS if not specified.
- `dev` subcommand: Runs the project in dev mode, recompiling and rerunning on file changes.

## Usage

```sh
cargo run -- build [--bundles deb rpm]
cargo run -- dev
```

## Installation

```sh
cargo install --path .
```

## TODO
- Implement actual bundling logic for each format.
- Add cross-compilation support.
- Improve error handling and logging.
