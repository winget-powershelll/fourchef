# Build Guide

This repo is a clean source build for the 4chef Tauri app. Use the steps below for dev or release bundles.

## Workflow

You can keep editing source code at all times. Build scripts only package the current code; they do not "lock" or freeze the repo.

## Prerequisites

Common:
- Rust toolchain via `rustup`
- `cargo` in PATH

Linux:
- WebKit GTK 4.1 dev libs
  - Fedora: `sudo dnf install webkit2gtk4.1-devel`
  - Ubuntu/Debian: `sudo apt install libwebkit2gtk-4.1-dev`

Windows:
- Rust toolchain via rustup
- Build tools for MSVC (Visual Studio Build Tools or full VS)

## Dev Run (All Platforms)

```bash
rustup target add wasm32-unknown-unknown
cargo install tauri-cli trunk --locked
cargo tauri dev
```

`cargo tauri dev` uses `scripts/trunk-dev` (or `scripts/trunk-dev.cmd` on Windows), which clears incompatible `NO_COLOR` values automatically.

## Linux Release Build

```bash
./scripts/build-linux.sh
```

Bundles (example):
```bash
./scripts/build-linux.sh --bundles appimage,deb,rpm
```

Clean/reproducible container build (recommended for release artifacts):
```bash
./scripts/build-linux-container.sh --bundles appimage,deb,rpm
```

## Windows Release Build

```powershell
.\scripts\build-windows.ps1
```

If you are in bash and need to invoke PowerShell:
```bash
pwsh -File ./scripts/build-windows.ps1
```

No-bundle build:
```powershell
.\scripts\build-windows.ps1 -NoBundle
```

Checklist:
```powershell
.\scripts\windows-release-checklist.ps1
```
