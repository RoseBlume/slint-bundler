# slint-bundler

A Rust CLI tool to build and bundle Rust projects for multiple platforms.
## Prerequisites
### Visual Studio Build tools
1. Download the Microsoft C++ Build Tools installer and open it to begin installation.
2. During installation check the “Desktop development with C++” option.

### Android Studio
1. Install Android studio
2. In the SDK manager install platform 33
3. Install ndk 27.0.12077973
4. Install platform tools if not automatically installed

## Environment variables
Append these directories to your path
```
C:\Users\{YourUsername}\AppData\Local\Android\Sdk\platform-tools
```
```
"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja"
```
## Features
- `build` subcommand: Compiles the project in release mode and bundles it for:
  - Windows: MSI, NSIS
  - Linux: .deb, .rpm, .tar.zst, .tar.xz (Arch), standalone
  - Use `--bundles` to specify which bundles to create (e.g. `--bundles deb rpm`).
  - Defaults to all supported bundles for the OS if not specified.
- `dev` subcommand: Runs the project in dev mode, recompiling and rerunning on file changes.

## Usage
First start up your android studio adb device and go to your projects root directory.

Now setup your icon. This must be a 1024x1024px png.
```sh
slint-bundler icon <path/to/your/icon>
```
Initialize and build your android project
```sh
slint-bundler android init
slint-bundler android dev
```

## Installation

```sh
cargo install --path .
```

# TODO
- Setup Paths for linux