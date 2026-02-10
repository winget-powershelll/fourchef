#!/usr/bin/env bash
set -euo pipefail

show_help() {
  cat <<'EOF'
Usage: ./scripts/build-linux.sh [--bundles <list>] [--no-bundle]

Examples:
  ./scripts/build-linux.sh
  ./scripts/build-linux.sh --bundles appimage,deb,rpm
  ./scripts/build-linux.sh --no-bundle
EOF
}

bundles=""
no_bundle="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundles)
      bundles="${2:-}"
      shift 2
      ;;
    --no-bundle)
      no_bundle="true"
      shift
      ;;
    -h|--help)
      show_help
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      show_help
      exit 1
      ;;
  esac
done

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing command: $1" >&2
    echo "Hint: $2" >&2
    exit 1
  fi
}

require_cmd cargo "Install Rust with rustup first."
require_cmd rustup "Install Rust with rustup first."
require_cmd trunk "Run: cargo install trunk --locked"

if ! cargo tauri --version >/dev/null 2>&1; then
  echo "tauri-cli is missing. Run: cargo install tauri-cli --locked" >&2
  exit 1
fi

rustup target add wasm32-unknown-unknown

# linuxdeploy strip can choke on RELR sections; disable strip via env.
export NO_STRIP=1

if [[ "$no_bundle" == "true" ]]; then
  cargo tauri build --no-bundle
elif [[ -n "$bundles" ]]; then
  cargo tauri build --bundles "$bundles"
else
  cargo tauri build
fi
