#!/usr/bin/env bash
set -euo pipefail

APP_DIR="target/release/Shuttle.app"
OUT_DIR="target/release"
ARCHIVE="$OUT_DIR/Shuttle.zip"

if [ ! -d "$APP_DIR" ]; then
  printf 'error: %s does not exist; run ./scripts/build-rust-app.sh first\n' "$APP_DIR" >&2
  exit 1
fi

python3 scripts/sync-version.py --check
rm -f "$ARCHIVE"
(
  cd "$OUT_DIR"
  ditto -c -k --sequesterRsrc --keepParent Shuttle.app Shuttle.zip
)
printf 'Packaged %s\n' "$ARCHIVE"
