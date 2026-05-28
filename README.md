# Shuttle

Shuttle is now a Rust-native macOS menu-bar app for launching shortcuts from a `.shuttle.json` config.

It preserves the legacy Shuttle config contract while adding explicit launch backends for Terminal.app, iTerm, Ghostty, cmux, URL opening, and virtual/background `screen` execution.

## Current status

The Rust core is ready for local testing:

- Config discovery and default config creation
- Legacy JSON config loading
- Alternate config host merge
- SSH config import
- Menu model sorting/separators
- Launch normalization and backend selection
- Terminal.app/iTerm AppleScript resource mapping
- Ghostty and cmux launch builders
- App bundle assembly

The native AppKit status-item shell is still being completed, so expect rough edges during manual testing.

## Build

```sh
./scripts/check-rust.sh
./scripts/build-rust-app.sh
```

The app bundle is created at:

```text
target/release/Shuttle.app
```

## Test with an isolated config

Avoid touching your real Shuttle config while testing:

```sh
cp tests/.shuttle.json /tmp/shuttle-test.json
printf '/tmp/shuttle-test.json\n' > ~/.shuttle.path
./target/release/Shuttle.app/Contents/MacOS/shuttle-rs
# Remove ~/.shuttle.path when done.
```

## Config compatibility

Existing `.shuttle.json` files should continue to load. Legacy keys remain supported:

- `terminal`
- `iTerm_version`
- `open_in`
- `default_theme`
- `editor`
- `launch_at_login`
- `show_ssh_config_hosts`
- `ssh_config_ignore_hosts`
- `ssh_config_ignore_keywords`
- host `cmd`, `name`, `inTerminal`, `theme`, and `title`

New optional keys:

- `backend`: `terminal-app`, `iterm-stable`, `iterm-nightly`, `ghostty-open`, `ghostty-applescript`, `cmux-cli`, `cmux-socket`, `screen`
- `strategy`: `default`, `workspace`, `socket`, `applescript`

Per-host `backend` / `strategy` override top-level values.

## Docs

- `docs/rust-port-migration.md`
- `docs/terminal-backends.md`
- `docs/troubleshooting-rust-port.md`
- `docs/packaging-rust-port.md`
- `docs/development/rust-port.md`

## Verification

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
python3 -m json.tool resources/shuttle.default.json >/dev/null
```

Or run all checks:

```sh
./scripts/check-rust.sh
```
