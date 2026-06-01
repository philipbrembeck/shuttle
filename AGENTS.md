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
- You MUST Document decisions in `docs/ADR/` when they're non-obvious or hard to reverse. Skip ADRs for routine choices.
- You MUST keep user-facing, developer, and architecture docs up to date with behavior changes.
- You MUST explicitly document exceptions to these rules in the relevant PR, commit, or ADR.

## Test instructions

- Before finishing code changes, you MUST run `./scripts/check-rust.sh`.
- `./scripts/check-rust.sh` MUST pass; it runs format check, Clippy with `-D warnings`, tests, `cargo check`, and default JSON validation.
- If you change behavior, you MUST add or update tests in the same change.
- If you change config, menu, SSH import, launcher, or compatibility behavior, you MUST cover it with unit tests.
- Tests that mutate process-wide state such as environment variables MUST serialize access with a mutex guard.
- Code Coverage SHOULD be >80% for new code; you MUST add or modify tests to reach that threshold.
- If you edit `resources/shuttle.default.json`, you MUST run `python3 -m json.tool resources/shuttle.default.json >/dev/null`.
- If you edit `tests/.shuttle.json`, you MUST run `python3 -m json.tool tests/.shuttle.json >/dev/null`.
- If you build the app bundle, use `./scripts/build-rust-app.sh`; output belongs under `target/release/Shuttle.app`.

## Coding rules

- You MUST keep modules small, typed, and aligned with existing domain concepts (`Config`, host entries, menu entries, launch requests, backends).
- You MUST NOT add dead code. Platform-conditional code that compiles on all targets but only executes on some (e.g. macOS-only callers) may trigger the dead_code lint; in that case, add a narrowly-scoped `#[allow(dead_code)]` on the specific item with a comment explaining which platform uses it. File-level `#![allow(dead_code)]` suppressions are not permitted.
- You MUST NOT add unused `#![allow(dead_code)]`, `#![allow(unused)]`, or equivalent broad suppressions; test-only helpers SHOULD be `#[cfg(test)]` instead of hidden behind broad allows.
- You MUST NOT use `unwrap()` in production code. `expect()` is permitted for invariants that represent programming errors rather than recoverable conditions. Always supply a message that explains the invariant: `mutex.lock().expect("app state mutex poisoned: this is a programming error")`. In test code, `unwrap()` and `expect()` are freely permitted.
- You MUST NOT use `#[allow(unsafe_code)]` or `unsafe` outside of `src/macos/` and its tests. Every block needs a safety comment. `#![allow(deprecated, unexpected_cfgs)]` is expected at the top of `src/macos/` files due to the cocoa crate; nowhere else.
- Shuttle SHOULD use as few dependencies as possible; add new dependencies only when they are clearly needed.
- You MUST NOT write dependency version strings from memory into `Cargo.toml`; use `cargo add <crate>` so Cargo resolves the correct latest version. Direct `Cargo.toml` edits are permitted when adjusting features, renaming, or pinning a specific version for a documented reason.
- You SHOULD prefer explicit error types with actionable messages.
- You SHOULD prefer argument vectors and structured data over shell-string construction.
- AppleScript string escaping MUST be shared or kept behaviorally identical across launch paths, including quote and backslash handling.
- ObjC objects that store Rust heap pointers MUST release them in `dealloc`; hot-reload paths MUST NOT intentionally leak per-menu-item state.
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
