param(
  [string]$Bundles = "",
  [switch]$NoBundle
)

$ErrorActionPreference = "Stop"

function Assert-Command {
  param(
    [string]$Name,
    [string]$Hint
  )

  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "$Name is not installed. $Hint"
  }
}

Assert-Command -Name "cargo" -Hint "Install Rust with rustup first."
Assert-Command -Name "rustup" -Hint "Install Rust with rustup first."
Assert-Command -Name "trunk" -Hint "Run: cargo install trunk --locked"

cargo tauri --version *> $null
if ($LASTEXITCODE -ne 0) {
  throw "tauri-cli is missing. Run: cargo install tauri-cli --locked"
}

rustup target add wasm32-unknown-unknown

if ($NoBundle -and $Bundles) {
  throw "Use either -NoBundle or -Bundles, not both."
}

if (Test-Path Env:NO_COLOR) {
  Remove-Item Env:NO_COLOR
}

if ($NoBundle) {
  cargo tauri build --no-bundle
} elseif ($Bundles) {
  cargo tauri build --bundles $Bundles
} else {
  cargo tauri build
}
