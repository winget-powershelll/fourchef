# 4chef

4chef is a desktop app for keeping kitchen operations data clean, searchable, and cost-ready.
It uses a Tauri shell, a Rust backend, a Leptos frontend, and a local SQLite database.

## Quick Start

### Linux/macOS

```bash
cd /path/to/4chef

# one-time setup
rustup target add wasm32-unknown-unknown
cargo install tauri-cli trunk --locked

# run dev
NO_COLOR=false cargo tauri dev
```

### Windows (PowerShell)

```powershell
cd C:\path\to\4chef

# one-time setup
rustup target add wasm32-unknown-unknown
cargo install tauri-cli trunk --locked

# run dev
cargo tauri dev
```

For prerequisites and clean build steps, see `docs/BUILD.md`.
You can also use `scripts/build-windows.ps1` on Windows and `scripts/build-linux.sh` on Linux.

## Current Modules

- Imports
- Inventory
- Recipes
- Vendors
- Conversions
- Purchasing
- Reports

## Import Inputs

4chef imports CSV exports from your chosen folder and stores normalized data in SQLite.

Supported inputs include:
- `Units.csv`
- `Inv.csv`
- `ConvUnit.csv`
- `RecpItems.csv`
- `InvUnits.csv`
- `InvPrices.csv`
- `Vendor.csv`
- `Recipe.csv`
- `Invoice.csv`
- `Trans.csv`

## In-App Data Fixes

You can now correct missing or messy data directly in the UI:
- Assign purchase units (including default flag)
- Add or override manual vendor pricing
- Merge duplicate vendor IDs into a single vendor

## Notes

- The local database is stored under your app data directory as `4chef.db`.
- Import notes and warnings are available in the Imports module details view.
