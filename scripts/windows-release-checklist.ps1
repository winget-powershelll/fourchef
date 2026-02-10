param(
  [switch]$DryRun
)

$ErrorActionPreference = "Stop"

Write-Host "Windows Release Checklist"
Write-Host ""
Write-Host "1. Install Rust via rustup"
Write-Host "2. Install Visual Studio Build Tools (MSVC)"
Write-Host "3. Install trunk and tauri-cli:"
Write-Host "   cargo install trunk --locked"
Write-Host "   cargo install tauri-cli --locked"
Write-Host "4. Add wasm target:"
Write-Host "   rustup target add wasm32-unknown-unknown"
Write-Host "5. Build bundles:"
Write-Host "   .\\scripts\\build-windows.ps1"
Write-Host "   or explicitly: .\\scripts\\build-windows.ps1 -Bundles msi,nsis"
Write-Host ""
Write-Host "Output bundles will be under target\\release\\bundle"

if (-not $DryRun) {
  Write-Host ""
  Write-Host "Tip: Run with -DryRun to only print this checklist."
}
