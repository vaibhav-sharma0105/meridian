#!/usr/bin/env bash
# gen-icons.sh — Generate all Tauri icon sizes from the source SVG
# Requires: npm (with tauri CLI available via package.json dev dependencies)
# Usage: ./scripts/gen-icons.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ICON_SRC="$ROOT/src-tauri/icons/icon.svg"

if [[ ! -f "$ICON_SRC" ]]; then
  echo "Error: Source icon not found at $ICON_SRC" >&2
  exit 1
fi

cd "$ROOT"

echo "→ Generating Tauri icons from $ICON_SRC"
npx tauri icon "$ICON_SRC"

echo "✓ Icons generated in src-tauri/icons/"
ls "$ROOT/src-tauri/icons/"
