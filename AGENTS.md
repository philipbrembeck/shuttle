# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project overview

Shuttle is a Rust-native macOS menu-bar app. It builds a status-bar menu from a user JSON config and SSH config entries, then launches commands through explicit backends for Terminal.app, iTerm, Ghostty, cmux, URLs, or virtual/background `screen` execution.

The old Objective-C/Xcode app has been removed. Treat the Rust implementation as the source of truth.

Primary code paths:

- `Cargo.toml` — Rust package definition and dependency allowlist.
- `src/main.rs` — app entry point and bootstrap wiring.
- `src/config/` — config path discovery, JSON model/loading, default config copy, reload timestamp helpers, SSH config import.
- `src/menu_model.rs` — normalized menu tree, sorting, sort-marker stripping, separators, disabled error menu items.
- `src/launcher/` — launch request normalization and backend-specific builders for Terminal.app, iTerm, Ghostty, cmux, URL, and virtual screen execution.
- `src/macos/` — macOS shell, menu specs, launch-at-login integration stubs, and AppKit-facing code.
- `resources/Shuttle-Info.plist` — app bundle metadata with `LSUIElement` and Automation usage text.
- `resources/shuttle.default.json` — bundled first-run default config.
- `resources/apple-scpt/*.scpt` — bundled AppleScript resources for Terminal.app/iTerm/screen compatibility.
- `scripts/check-rust.sh` — local quality gate.
- `scripts/build-rust-app.sh` — assembles `target/release/Shuttle.app`.
- `tests/.shuttle.json` — sample config for manual testing.

## Build and verification

Run the Rust quality gate before finishing changes:

```sh
./scripts/check-rust.sh
```

This runs:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo check
python3 -m json.tool resources/shuttle.default.json >/dev/null
```

Build the app bundle:

```sh
./scripts/build-rust-app.sh
```

The bundle is created at:

```text
target/release/Shuttle.app
```

Validate sample/default JSON when editing configs:

```sh
python3 -m json.tool resources/shuttle.default.json >/dev/null
python3 -m json.tool tests/.shuttle.json >/dev/null
```

## Manual testing

Use an isolated config rather than changing a real user config:

```sh
cp tests/.shuttle.json /tmp/shuttle-test.json
printf '/tmp/shuttle-test.json\n' > ~/.shuttle.path
./target/release/Shuttle.app/Contents/MacOS/shuttle-rs
# Remove ~/.shuttle.path after testing to restore default behavior.
```

If testing alternate-config behavior, use `~/.shuttle-alt.path` or `~/.shuttle-alt.json` similarly, and clean them up afterward.

## Runtime behavior to preserve

- Default config path is `~/.shuttle.json`; `~/.shuttle.path` can override it.
- Alternate config can be enabled by `~/.shuttle-alt.path` or default `~/.shuttle-alt.json` if present.
- On first run, `resources/shuttle.default.json` is copied to the default config path if missing and no main override is present.
- The menu should reload when the main config, alternate config, `/etc/ssh/ssh_config`, or `~/.ssh/config` modification times change.
- SSH config parsing supports comments of the form `# shuttle.<key> <value>` for per-host metadata and supports legacy `Include` handling.
- `hosts` entries can be nested menus, commands, separators via `[---]` in names, and sorted through `[aaa]`-style prefixes.
- Existing configs should keep working with `terminal`, `iTerm_version`, `open_in`, `default_theme`, and per-host `inTerminal`, `theme`, and `title`.
- New `backend` and `strategy` keys are optional. Per-host values override top-level values.

## Coding conventions

- Keep modules small and typed around legacy concepts (`Config`, host entries, menu entries, launch requests, backends).
- Prefer explicit error types with actionable messages.
- Add unit tests for compatibility behavior whenever changing config, menu, SSH, or launcher code.
- Avoid shell-string construction. Prefer argument vectors for process launches and structured JSON for socket calls.
- Keep the native macOS app lightweight. Do not introduce webview/Electron/Tauri-style runtimes.
- Preserve old macOS compatibility where practical; guard newer AppKit/ServiceManagement APIs with availability checks when implemented.
- Keep JSON examples valid strict JSON.

## Repository hygiene

- Build artifacts belong under `target/` and are ignored.
- Do not commit personal config files (`~/.shuttle.json`, `.shuttle.path`, etc.).
- Do not add machine-specific editor or IDE state.
- Be careful with binary resources (`.icns`, `.scpt`); update them only when required.

## Useful docs

- `docs/development/rust-port.md` — Rust development workflow and hooks.
- `docs/rust-port-migration.md` — compatibility and migration guidance.
- `docs/terminal-backends.md` — backend and strategy configuration.
- `docs/troubleshooting-rust-port.md` — common runtime issues.
- `docs/packaging-rust-port.md` — app bundle packaging.
