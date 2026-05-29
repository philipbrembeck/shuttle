#!/usr/bin/env bash
set -euo pipefail

python3 scripts/sync-version.py --check
cargo build --release
APP_DIR="target/release/Shuttle.app"
CONTENTS="$APP_DIR/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"

rm -rf "$APP_DIR"
mkdir -p "$MACOS" "$RESOURCES"
cp resources/Shuttle-Info.plist "$CONTENTS/Info.plist"
cp target/release/shuttle-rs "$MACOS/shuttle-rs"
cp resources/shuttle.default.json "$RESOURCES/shuttle.default.json"
if [ -d resources/apple-scpt ]; then
  cp resources/apple-scpt/*.scpt "$RESOURCES/"
fi
# Prefer the generated icns in resources/, fall back to root-level legacy file
if [ -f resources/shuttle.icns ]; then
  cp resources/shuttle.icns "$RESOURCES/shuttle.icns"
elif [ -f shuttle.icns ]; then
  cp shuttle.icns "$RESOURCES/shuttle.icns"
fi

printf 'Built %s\n' "$APP_DIR"
