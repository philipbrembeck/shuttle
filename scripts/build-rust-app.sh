#!/usr/bin/env bash
set -euo pipefail

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
if [ -f shuttle.icns ]; then
  cp shuttle.icns "$RESOURCES/shuttle.icns"
fi

printf 'Built %s\n' "$APP_DIR"
