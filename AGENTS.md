# AGENTS.md — Shuttle

Rust-native macOS menu-bar app. RFC 2119 meanings apply to MUST, SHOULD, and MAY.

## Project map

| Area | Path |
| --- | --- |
| App bootstrap | `src/main.rs` |
| Config + SSH import | `src/config/` |
| Menu model | `src/menu_model.rs` |
| Launch backends | `src/launcher/` |
| macOS integration | `src/macos/` |
| Bundled config/resources | `resources/` |
| Quality/build scripts | `scripts/check-rust.sh`, `scripts/build-rust-app.sh` |

## Need to know

- The Rust app is source of truth; removed Objective-C/Xcode code MUST NOT guide changes.
- Shuttle builds menus from user JSON or experimental YAML plus SSH config, then launches via Terminal.app, iTerm, Ghostty, cmux, URLs, or virtual/background `screen`.
- First-run default config is `~/.config/shuttle/config.json`; `~/.shuttle.path` MAY override it. Standard `config.yaml`/`config.yml` files in `~/.config/shuttle/` are experimental and win over standard JSON when present.
- Alternate config uses `~/.shuttle-alt.path`, experimental `~/.config/shuttle/alt.yaml`/`alt.yml`, `~/.config/shuttle/alt.json`, or legacy `~/.shuttle-alt.json` when present.
- Menus reload when main config, alternate config, `/etc/ssh/ssh_config`, or `~/.ssh/config` mtimes change.
- SSH import supports `# shuttle.<key> <value>` metadata and legacy `Include` behavior.
- Existing config keys (`terminal`, `iTerm_version`, `open_in`, `default_theme`, `inTerminal`, `theme`, `title`) MUST keep working.

## Workflow rules

- You MUST work on a new branch; do not change `main` directly.
  Allowed branch prefixes are "feat/", "fix/", "refactor/", "chore/", "deps/", and "docs/".
  This does NOT apply to trivial changes like typos, formatting, or planning; those can be made directly on `main`.
- You MUST regularly update your branch from `main` during longer work.
- You MUST keep commits free of build artifacts, personal config, editor state, and machine-specific files.
- You MUST document every architecture decision in `docs/ADR/`; create the directory if missing.
- You MUST keep user-facing, developer, and architecture docs up to date with behavior changes.
- You MUST explicitly document exceptions to these rules in the relevant PR, commit, or ADR.

## Test instructions

- Before finishing code changes, you MUST run `./scripts/check-rust.sh`.
- `./scripts/check-rust.sh` MUST pass; it runs format check, Clippy with `-D warnings`, tests, `cargo check`, and default JSON validation.
- If you change behavior, you MUST add or update tests in the same change.
- If you change config, menu, SSH import, launcher, or compatibility behavior, you MUST cover it with unit tests.
- Code Coverage SHOULD be >80% for new code; you MUST add or modify tests to reach that threshold.
- If you edit `resources/shuttle.default.json`, you MUST run `python3 -m json.tool resources/shuttle.default.json >/dev/null`.
- If you edit `tests/.shuttle.json`, you MUST run `python3 -m json.tool tests/.shuttle.json >/dev/null`.
- If you build the app bundle, use `./scripts/build-rust-app.sh`; output belongs under `target/release/Shuttle.app`.

## Coding rules

- You MUST keep modules small, typed, and aligned with existing domain concepts (`Config`, host entries, menu entries, launch requests, backends).
- You MUST NOT add dead code.
- You MUST NOT add unused `#![allow(dead_code)]`, `#![allow(unused)]`, or equivalent broad suppressions.
- You MUST NOT use `unwrap()`, `expect()`, or similar panicking code; handle errors explicitly.
- You MUST NOT use `#[allow(unsafe_code)]` or `unsafe` blocks without a comment explaining why it's necessary and how safety is ensured.
- Shuttle SHOULD use as few dependencies as possible; add new dependencies only when they are clearly needed.
- You MUST add or update direct dependencies with Cargo CLI commands (`cargo add`, `cargo update`), not by editing `Cargo.toml` manually, so Cargo resolves the latest appropriate version.
- You SHOULD prefer explicit error types with actionable messages.
- You SHOULD prefer argument vectors and structured data over shell-string construction.
- You MUST keep JSON examples strict JSON.
- You MUST preserve macOS compatibility where practical; newer APIs MUST be availability-guarded.
- You MUST NOT introduce webview, Electron, or Tauri-style runtimes.

## Load when needed

You MUST load these docs when the "When?" condition applies.

| What? | Where? | When? |
| --- | --- | --- |
| Development workflow | `docs/development/rust-port.md` | When changing build, checks, hooks, or local workflow. |
| Migration/compatibility | `docs/rust-port-migration.md` | When changing legacy config behavior or compatibility guarantees. |
| Terminal backends | `docs/terminal-backends.md` | When changing launch backends, strategies, or backend config. |
| Packaging | `docs/packaging-rust-port.md` | When changing app bundle, release, signing, or packaging behavior. |
| Troubleshooting | `docs/troubleshooting-rust-port.md` | When diagnosing runtime failures or documenting known issues. |
