#!/usr/bin/env bash
set -euo pipefail

show_help() {
  cat <<'EOF'
Usage: ./scripts/build-linux-container.sh [--bundles <list>] [--image <name>] [--no-bundle]

Examples:
  ./scripts/build-linux-container.sh
  ./scripts/build-linux-container.sh --bundles appimage,deb,rpm
  ./scripts/build-linux-container.sh --no-bundle
EOF
}

bundles="appimage,deb,rpm"
no_bundle="false"
image_name="fourchef-linux-builder:local"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundles)
      bundles="${2:-}"
      shift 2
      ;;
    --image)
      image_name="${2:-}"
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

if ! command -v podman >/dev/null 2>&1; then
  echo "Missing command: podman" >&2
  echo "Install podman first." >&2
  exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
containerfile="$repo_root/tools/container/Containerfile.linux-build"

echo "Building container image: $image_name"
podman build -t "$image_name" -f "$containerfile" "$repo_root"

build_cmd=("./scripts/build-linux.sh")
if [[ "$no_bundle" == "true" ]]; then
  build_cmd+=("--no-bundle")
else
  build_cmd+=("--bundles" "$bundles")
fi

echo "Running clean build in container"
podman run --rm \
  -v "$repo_root":/workspace:Z \
  -w /workspace \
  "$image_name" \
  bash -lc "${build_cmd[*]}"

echo "Container build completed"
