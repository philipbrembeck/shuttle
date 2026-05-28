# Rust Port Packaging

Build and assemble a local `.app` bundle:

```sh
./scripts/check-rust.sh
./scripts/build-rust-app.sh
```

The bundle is written to:

```text
target/release/Shuttle.app
```

The packaging script copies:

- `resources/Shuttle-Info.plist` to `Contents/Info.plist`
- `target/release/shuttle-rs` to `Contents/MacOS/shuttle-rs`
- `resources/shuttle.default.json` to `Contents/Resources/shuttle.default.json`
- `shuttle.icns` to `Contents/Resources/shuttle.icns` when present

For release builds, codesign/notarization should be added once the Rust AppKit shell is complete.
