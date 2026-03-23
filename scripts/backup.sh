#!/usr/bin/env bash
# backup.sh — Back up the Meridian SQLite database to a timestamped file
# Usage: ./scripts/backup.sh [--dest /path/to/backup/dir]
set -euo pipefail

DEST=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dest) DEST="$2"; shift 2 ;;
    *) echo "Unknown argument: $1"; exit 1 ;;
  esac
done

# Detect data directory per platform
if [[ "$OSTYPE" == "darwin"* ]]; then
  DATA_DIR="$HOME/Library/Application Support/com.meridian.app"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
  DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/com.meridian.app"
else
  echo "Unsupported platform: $OSTYPE" >&2
  exit 1
fi

DB="$DATA_DIR/meridian.db"

if [[ ! -f "$DB" ]]; then
  echo "Database not found at: $DB" >&2
  echo "Is Meridian installed and has been run at least once?" >&2
  exit 1
fi

if [[ -z "$DEST" ]]; then
  DEST="$DATA_DIR/backups"
fi

mkdir -p "$DEST"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$DEST/meridian_backup_$TIMESTAMP.db"

echo "→ Backing up database"
echo "  Source: $DB"
echo "  Destination: $BACKUP_FILE"

# Use SQLite's .backup command for a consistent snapshot (safe even while app is running)
if command -v sqlite3 &>/dev/null; then
  sqlite3 "$DB" ".backup '$BACKUP_FILE'"
else
  # Fallback: checkpoint WAL first, then copy
  cp "$DB" "$BACKUP_FILE"
  [[ -f "$DB-wal" ]] && cp "$DB-wal" "$BACKUP_FILE-wal" || true
  [[ -f "$DB-shm" ]] && cp "$DB-shm" "$BACKUP_FILE-shm" || true
fi

SIZE=$(du -sh "$BACKUP_FILE" | cut -f1)
echo "✓ Backup complete ($SIZE): $BACKUP_FILE"

# Rotate: keep only the 10 most recent backups
BACKUP_COUNT=$(ls "$DEST"/meridian_backup_*.db 2>/dev/null | wc -l)
if [[ $BACKUP_COUNT -gt 10 ]]; then
  echo "→ Rotating old backups (keeping 10 most recent)"
  ls -t "$DEST"/meridian_backup_*.db | tail -n +11 | xargs rm -f
fi
