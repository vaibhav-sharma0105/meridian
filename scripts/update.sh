#!/usr/bin/env bash
# update.sh — Build and package Meridian for all platforms
# Usage: ./scripts/update.sh [--version x.y.z] [--skip-frontend]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

VERSION=""
SKIP_FRONTEND=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --skip-frontend) SKIP_FRONTEND=true; shift ;;
    *) echo "Unknown argument: $1"; exit 1 ;;
  esac
done

if [[ -n "$VERSION" ]]; then
  echo "→ Bumping version to $VERSION"
  # Update Cargo.toml
  sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" "$ROOT/src-tauri/Cargo.toml"
  # Update tauri.conf.json
  node -e "
    const fs = require('fs');
    const conf = JSON.parse(fs.readFileSync('$ROOT/src-tauri/tauri.conf.json', 'utf8'));
    conf.version = '$VERSION';
    fs.writeFileSync('$ROOT/src-tauri/tauri.conf.json', JSON.stringify(conf, null, 2));
  "
  # Update package.json
  node -e "
    const fs = require('fs');
    const pkg = JSON.parse(fs.readFileSync('$ROOT/package.json', 'utf8'));
    pkg.version = '$VERSION';
    fs.writeFileSync('$ROOT/package.json', JSON.stringify(pkg, null, 2));
  "
  echo "  Version updated to $VERSION"
fi

cd "$ROOT"

if [[ "$SKIP_FRONTEND" == false ]]; then
  echo "→ Installing npm dependencies"
  npm ci

  echo "→ Type-checking TypeScript"
  npx tsc --noEmit

  echo "→ Building frontend"
  npm run build
fi

echo "→ Building Tauri (release)"
npm run tauri build

echo ""
echo "✓ Build complete. Artifacts are in src-tauri/target/release/bundle/"
ls src-tauri/target/release/bundle/ 2>/dev/null || true
