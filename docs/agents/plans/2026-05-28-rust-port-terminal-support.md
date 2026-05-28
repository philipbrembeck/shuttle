---
date: 2026-05-28T17:58:04.545675+00:00
git_commit: 1080360d4e586935840836e4ac9c5b48a690b9f3
branch: main
topic: "Rust port with Ghostty and cmux support"
tags: [plan, rust-port, macos, terminal-support, ghostty, cmux]
status: draft
---

# Rust Port with Ghostty and cmux Support Implementation Plan

## Overview

Port Shuttle from Objective-C/Cocoa to a lightweight native Rust macOS menu-bar app. The port should preserve existing `.shuttle.json` behavior by default, while adding a backend/strategy launch model that supports Terminal.app, iTerm, Ghostty, cmux, URL opening, and virtual/background execution.

## Current State Analysis

Shuttle is currently a legacy Objective-C macOS menu-bar app with most behavior in `Shuttle/AppDelegate.m`:

- Startup and config discovery happen in `awakeFromNib`, including `~/.shuttle.path`, default `~/.shuttle.json`, and alternate config path handling.
- The app is menu-bar-only through `LSUIElement` in `Shuttle/Shuttle-Info.plist`.
- JSON parsing uses `NSJSONSerialization` and reads global preferences such as `terminal`, `editor`, `iTerm_version`, `open_in`, `default_theme`, `launch_at_login`, `hosts`, and SSH ignore lists.
- SSH hosts are parsed from system/user SSH config with simple `Host`, `Include`, and `# shuttle.*` support.
- Menu building recursively converts nested config objects into `NSMenu` trees, sorts menus/leaves independently, and strips `[aaa]` / `[---]` markers.
- Command dispatch is centralized in `openHost:` and currently mixes command normalization, URL detection, theme/title fallback, validation, AppleScript selection, and execution.
- Terminal.app, iTerm, and virtual/background support are implemented through bundled compiled AppleScripts.

The Rust port should avoid a broad framework rewrite. A tiny dependency allowlist is acceptable for pragmatic pieces such as JSON parsing, but the app should remain a native AppKit menu-bar app rather than a web or cross-platform UI application.

## Desired End State

A Rust-native Shuttle app exists with:

- Native macOS app bundle and menu-bar-only behavior.
- Backward-compatible loading of existing `.shuttle.json` configs.
- Compatible nested menu rendering, SSH config import, alt config merge, reload behavior, import/export/config actions, and launch-at-login behavior where feasible.
- A typed internal model for config entries, menu entries, launch requests, and terminal backends.
- Explicit backend/strategy dispatch supporting:
  - Terminal.app AppleScript
  - iTerm AppleScript
  - Ghostty via `open -na Ghostty.app --args ...`
  - Ghostty via AppleScript
  - cmux CLI
  - cmux Unix socket JSON API
  - virtual/background `screen`
  - URL opening through the OS
- Updated default config and migration documentation for new backend features.
- Strict Rust formatting, linting, testing, and pre-commit hooks enforced early in the rewrite.

## What We're NOT Doing

- [ ] Do not replace the app with a webview, Electron, Tauri, or other heavy UI runtime.
- [ ] Do not require users to migrate existing `.shuttle.json` files for basic behavior.
- [ ] Do not implement a full OpenSSH config parser beyond the legacy-supported subset unless needed for compatibility fixes.
- [ ] Do not remove Terminal.app or iTerm support while adding Ghostty/cmux.
- [ ] Do not make cmux socket access mandatory; users should be able to use CLI integration.
- [ ] Do not make Ghostty AppleScript mandatory for simple new-window launch; `open --args` should be available.

## UI Mockups

Current and desired user-facing menu shape should remain similar:

```text
┌ menu bar ────────────────────────────────┐
│ ...                                  🚀 │
└──────────────────────────────────────────┘
                                      │
                                      ▼
                         ┌────────────────────────┐
                         │ Spouses Servers       ▶│
                         │ Vagrant               ▶│
                         │ SSH blog               │
                         │ ────────────────────── │
                         │ Configure...           │
                         │ Import...              │
                         │ Export...              │
                         │ About Shuttle          │
                         │ Quit                   │
                         └────────────────────────┘
```

New behavior is mostly configuration-driven. Example compatible config plus new optional backend keys:

```json
{
  "terminal": "Terminal.app",
  "open_in": "new",
  "backend": "ghostty-open",
  "hosts": [
    {
      "cmd": "ssh prod",
      "name": "Prod",
      "backend": "cmux-cli",
      "strategy": "workspace"
    }
  ]
}
```

## Architecture and Code Reuse

The port should preserve current behavior by modeling legacy concepts directly, then routing through a new launch abstraction.

```text
AppKit status item
  └─ MenuController
      ├─ ConfigLoader
      │   ├─ path discovery
      │   ├─ JSON parser/model
      │   ├─ alt config merge
      │   └─ SSH config import
      ├─ MenuBuilder
      │   ├─ sorting
      │   ├─ separator/sort marker stripping
      │   └─ NSMenu construction
      └─ Launcher
          ├─ URL opener
          ├─ AppleScript backends
          ├─ process/CLI backends
          ├─ cmux socket backend
          └─ virtual screen backend
```

Candidate Rust module tree:

- `src/main.rs` - app entry point and panic/error bootstrap.
- `src/macos/app.rs` - `NSApplication`, status item, app lifecycle, bundle integration.
- `src/macos/menu.rs` - AppKit `NSMenu` / `NSMenuItem` construction and callbacks.
- `src/macos/dialogs.rs` - open/save panels, alerts, config editor open action.
- `src/macos/applescript.rs` - AppleScript execution boundary.
- `src/config/mod.rs` - config discovery and loading orchestration.
- `src/config/model.rs` - typed config, host, menu, and backend model.
- `src/config/legacy.rs` - legacy compatibility defaults and key mapping.
- `src/config/ssh.rs` - SSH config parser and menu host merge.
- `src/menu_model.rs` - normalized internal menu tree and marker stripping.
- `src/launcher/mod.rs` - `LaunchRequest`, `LaunchTarget`, `Backend`, `LaunchStrategy`.
- `src/launcher/url.rs` - OS URL opening.
- `src/launcher/terminal_app.rs` - Terminal.app AppleScript backend.
- `src/launcher/iterm.rs` - iTerm stable/nightly AppleScript backend.
- `src/launcher/ghostty.rs` - Ghostty `open --args` and AppleScript strategies.
- `src/launcher/cmux.rs` - cmux CLI and socket strategies.
- `src/launcher/virtual_screen.rs` - background `screen` execution.
- `resources/shuttle.default.json` - backward-compatible default config with new optional examples.
- `docs/` or `README.md` - migration, backend strategy, and terminal support docs.

Core launch types:

```rust
enum LaunchTarget {
    New,
    Tab,
    Current,
    Virtual,
}

enum Backend {
    TerminalApp,
    ITerm { version: ITermVersion },
    GhosttyOpen,
    GhosttyAppleScript,
    CmuxCli,
    CmuxSocket,
    Screen,
}

struct LaunchRequest {
    command: String,
    title: String,
    theme_or_profile: Option<String>,
    target: LaunchTarget,
    backend: Backend,
}
```

## Performance Considerations

- Config and SSH parsing should remain lazy and happen when the menu opens, preserving current behavior.
- File modification timestamps should avoid unnecessary parsing.
- cmux socket connections should be short-lived initially unless profiling shows repeated connection overhead.
- Avoid spawning shell processes except for explicit CLI/open/screen strategies.
- Keep Rust binary and app bundle small by avoiding large UI/runtime dependencies.

## Migration Notes

- Existing `.shuttle.json` files should continue to work with `terminal`, `iTerm_version`, `open_in`, `default_theme`, and per-host `inTerminal`, `theme`, `title`.
- New keys such as `backend` and `strategy` should be optional and should not change legacy behavior unless present.
- Legacy `terminal: "iTerm"` or values containing `iterm` should map to the iTerm backend.
- Legacy `terminal: "Terminal.app"` or unknown terminal values should preserve the current effective Terminal.app fallback unless invalid values are intentionally surfaced as warnings.
- `inTerminal: "virtual"` should continue to run the background `screen` strategy.

## Phase 1: Rust App Shell and Compatible Config Core

Create the minimal Rust app foundation and preserve the existing config loading contract.

**Tasks**:
- [x] Create a Rust app target and macOS app bundle metadata equivalent to `Shuttle/Shuttle-Info.plist`, including `LSUIElement` and Apple Events usage text.
- [ ] Implement native AppKit lifecycle setup in `src/macos/app.rs`, including `NSApplication`, status item, icon loading, and menu delegate/callback wiring.
- [x] Implement config path discovery in `src/config/mod.rs` for `~/.shuttle.path`, default `~/.shuttle.json`, `~/.shuttle-alt.path`, and default `~/.shuttle-alt.json`.
- [x] Copy bundled `resources/shuttle.default.json` to the default config path on first run when no override exists.
- [x] Define typed config structs in `src/config/model.rs` for legacy top-level fields and nested hosts.
- [x] Implement JSON loading with the approved tiny dependency allowlist, preserving mutable/merge behavior needed for alt config hosts.
- [x] Implement timestamp-based reload checks for main config, alternate config, `/etc/ssh/ssh_config`, and `~/.ssh/config`.
- [ ] Add user-facing error display for invalid JSON equivalent to the current disabled “Error parsing config” menu item.

**Automated Verification**:
- [x] `cargo check` passes.
- [x] Config path discovery unit tests cover default config, `~/.shuttle.path`, alt path, and alt default detection.
- [x] JSON model tests load `Shuttle/shuttle.default.json` or the new copied equivalent successfully.
- [x] Invalid JSON test produces a structured config error suitable for menu display.

**Manual Verification**:
- [ ] Launch the Rust app with no config present and verify it creates `~/.shuttle.json` from the bundled default.
- [ ] Launch with a temporary `~/.shuttle.path` pointing to a disposable config and verify the app uses that file.

---

## Phase 2: Rust Quality Gates and Pre-commit Hooks

Establish strict formatting, linting, and testing rules immediately after the Rust skeleton exists so every later phase inherits the same quality bar.

**Dependencies**: **Phase 1**

**Tasks**:
- [x] Add `rustfmt` configuration and document the required `cargo fmt --check` workflow.
- [x] Add strict Clippy configuration for the workspace, including `-D warnings` and project-approved deny/warn rules that fit a small native macOS app.
- [x] Add baseline test layout and conventions for unit tests, integration tests, macOS-bound tests, and fake process/socket tests.
- [x] Add a single local verification command or script that runs format, lint, tests, and build checks in the same order expected by CI/pre-commit.
- [x] Add pre-commit hooks via Rust Husky so staged Rust/config/docs changes run formatting, linting, and relevant tests before commit.
- [x] Document how to install, update, bypass, and troubleshoot the Rust Husky hooks for contributors.
- [x] Ensure generated/build artifacts from hooks and tests are ignored and do not pollute the repository.

**Automated Verification**:
- [x] `cargo fmt --check` passes.
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [x] `cargo test` passes.
- [x] The local all-checks script passes from a clean checkout after hook installation.
- [x] A Rust Husky hook dry run or equivalent local invocation runs the expected checks and fails on an intentional formatting/lint error.

---

## Phase 3: Menu Model, SSH Import, and Legacy Launch Parity

Port the visible menu behavior and preserve existing Terminal.app/iTerm/URL/virtual execution semantics.

**Dependencies**: **Phase 2**

**Tasks**:
- [ ] Implement `src/menu_model.rs` to convert nested config hosts into a normalized menu tree.
- [ ] Preserve independent case-insensitive sorting of submenu entries and command leaves.
- [ ] Preserve `[aaa]` sort marker removal and `[---]` separator insertion semantics.
- [ ] Implement `src/config/ssh.rs` with legacy-compatible parsing for `Host`, `Include`, first alias selection, and `# shuttle.*` comments.
- [ ] Implement SSH host filtering for wildcards, dot-prefixed names, exact ignored hosts, and ignored keyword substrings.
- [ ] Implement SSH path splitting by `/` and merging into the config menu tree.
- [ ] Implement `src/macos/menu.rs` to build native `NSMenu` / `NSMenuItem` trees from the normalized menu model.
- [ ] Implement `LaunchRequest` normalization in `src/launcher/mod.rs`, including command, title fallback, theme fallback, target validation, URL detection, and legacy terminal mapping.
- [ ] Implement URL opening through the macOS workspace API.
- [ ] Implement Terminal.app AppleScript strategy matching existing `new`, `tab`, and `current` modes.
- [ ] Implement iTerm stable/nightly AppleScript strategies matching existing `new`, `tab`, and `current` modes.
- [ ] Implement virtual/background execution with `screen -d -m -S <title> <cmd>`.
- [ ] Port import, export, configure, about, and quit menu actions where they are part of the current menu.

**Automated Verification**:
- [ ] `cargo check` passes.
- [ ] `cargo fmt --check` and strict Clippy checks pass.
- [ ] Menu model tests cover nested menus, leaves, sorting, sort marker stripping, and separator markers.
- [ ] SSH parser tests cover `Host`, multiple aliases, `Include`, `# shuttle.name`, wildcard filtering, ignored hosts, and ignored keywords.
- [ ] Launch normalization tests cover legacy Terminal.app, iTerm stable/nightly, URL commands, invalid `inTerminal`, title fallback, theme fallback, and virtual target mapping.

**Manual Verification**:
- [ ] Use `tests/.shuttle.json` through `~/.shuttle.path` and verify the Rust app menu visually matches the Objective-C app menu structure.
- [ ] Trigger a URL command and verify it opens in the default browser instead of a terminal.
- [ ] Trigger Terminal.app `new`, `tab`, and `current` commands and verify command, title, and profile/theme behavior where supported.
- [ ] Trigger iTerm stable/nightly commands and verify command, title, and profile behavior where supported.
- [ ] Trigger a `virtual` command and verify a detached `screen` session is created.

---

## Phase 4: Backend Strategy System

Make terminal dispatch explicit and extensible while keeping legacy keys working.

**Dependencies**: **Phase 3**

**Tasks**:
- [ ] Refactor `src/launcher/mod.rs` so every launch goes through `Backend` and `LaunchStrategy` resolution.
- [ ] Add optional top-level `backend` and `strategy` config keys without changing legacy defaults.
- [ ] Add optional per-host `backend` and `strategy` overrides.
- [ ] Define precedence rules: per-host backend/strategy, top-level backend/strategy, legacy `terminal`/`open_in`, then defaults.
- [ ] Add structured errors/warnings for unsupported backend and target combinations.
- [ ] Update `resources/shuttle.default.json` comments/examples to show legacy keys plus optional backend keys.
- [ ] Document backend selection, target mapping, and compatibility behavior in project docs or README.

**Automated Verification**:
- [ ] `cargo check` passes.
- [ ] Backend resolution tests cover legacy-only config, top-level backend, per-host backend override, invalid backend, and invalid strategy.
- [ ] Config compatibility tests verify old configs produce the same backend/target choices as Phase 2.
- [ ] `python -m json.tool resources/shuttle.default.json` passes, or equivalent JSON validation if comments are not represented in strict JSON.

**Manual Verification**:
- [ ] Configure one host with a per-host backend override and verify only that host uses the override.
- [ ] Configure a top-level backend and verify hosts without overrides inherit it.

---

## Phase 5: Ghostty Support

Add both approved Ghostty strategies: command/open-based launch and AppleScript automation.

**Dependencies**: **Phase 4**

**Tasks**:
- [ ] Implement `GhosttyOpen` strategy in `src/launcher/ghostty.rs` using `open -na Ghostty.app --args ...`.
- [ ] Map `LaunchTarget::New` to the Ghostty open strategy and define clear fallback/error behavior for `Tab` and `Current` if selected with `ghostty-open`.
- [ ] Pass command, title, and supported config/profile arguments through `open --args` with safe argument handling and no shell interpolation.
- [ ] Implement `GhosttyAppleScript` strategy using Ghostty 1.3+ AppleScript support for `new`, `tab`, and `current` where the scripting API supports them.
- [ ] Add detection/error messaging for missing Ghostty, unsupported Ghostty AppleScript version, or Apple Events denial.
- [ ] Add default config examples for `ghostty-open` and `ghostty-applescript`.
- [ ] Document when to use `ghostty-open` versus `ghostty-applescript`.

**Automated Verification**:
- [ ] `cargo check` passes.
- [ ] Ghostty command construction tests verify `open -na Ghostty.app --args` argument vectors for command, title, and target combinations.
- [ ] Ghostty backend resolution tests cover `ghostty-open`, `ghostty-applescript`, unsupported target combinations, and per-host overrides.
- [ ] AppleScript generation/invocation wrapper tests verify parameters are passed as separate arguments rather than shell-concatenated strings.

**Manual Verification**:
- [ ] With Ghostty installed, configure a host with `backend: "ghostty-open"` and verify it opens a new Ghostty instance/window running the command.
- [ ] With Ghostty 1.3+ installed and Automation permission granted, configure `backend: "ghostty-applescript"` and verify `new`, `tab`, and `current` behavior.
- [ ] Deny Automation permission and verify the app surfaces a useful error instead of silently failing.

---

## Phase 6: cmux Support

Add both approved cmux strategies: CLI integration and Unix socket JSON API integration.

**Dependencies**: **Phase 4**

**Tasks**:
- [ ] Implement `CmuxCli` strategy in `src/launcher/cmux.rs` by spawning the `cmux` binary with argument vectors, not shell strings.
- [ ] Add configurable cmux binary path with default discovery for `/Applications/cmux.app/Contents/Resources/bin/cmux` and `PATH` lookup.
- [ ] Define initial `LaunchTarget` mapping for cmux CLI: `new`/`tab` to workspace-oriented behavior, `current` to focused/current surface send, and unsupported cases to structured errors.
- [ ] Implement `CmuxSocket` strategy using newline-delimited JSON requests over Unix domain socket.
- [ ] Add configurable cmux socket path with default `/tmp/cmux.sock` and environment override via `CMUX_SOCKET_PATH`.
- [ ] Implement socket calls for workspace creation/selection/current lookup and surface text sending needed by Shuttle commands.
- [ ] Document cmux access mode requirements, including `CMUX_SOCKET_MODE=allowAll` when external local processes need socket access.
- [ ] Add default config examples for `cmux-cli` and `cmux-socket`.

**Automated Verification**:
- [ ] `cargo check` passes.
- [ ] cmux CLI command construction tests cover workspace creation, current-surface send, binary path override, and missing binary error.
- [ ] cmux socket serialization tests verify newline-delimited JSON request shape with `id`, `method`, and `params`.
- [ ] cmux socket integration test using a fake Unix socket server verifies request/response handling and error propagation.
- [ ] Backend resolution tests cover `cmux-cli`, `cmux-socket`, top-level config, and per-host overrides.

**Manual Verification**:
- [ ] With cmux installed, configure `backend: "cmux-cli"` and verify a command can be sent to the intended workspace/surface behavior.
- [ ] With cmux socket access enabled, configure `backend: "cmux-socket"` and verify commands are sent through the socket.
- [ ] Configure an unavailable cmux binary/socket and verify the app displays an actionable error.

---

## Phase 7: User-Facing Polish, Packaging, and Migration Docs

Finish the port as an installable app with clear compatibility and migration guidance.

**Dependencies**: **Phase 5**, **Phase 6**

**Tasks**:
- [ ] Finalize app bundle resources: icon, default config, AppleScript resources if still bundled, localized user-visible strings where practical.
- [ ] Preserve or replace launch-at-login behavior in a Rust-compatible way appropriate for supported macOS versions.
- [ ] Add a migration guide explaining existing config compatibility and new backend/strategy keys.
- [ ] Add terminal support documentation for Terminal.app, iTerm, Ghostty, cmux, URL commands, and virtual/screen mode.
- [ ] Add troubleshooting docs for Automation permission, missing terminal apps, cmux socket access, and config parse errors.
- [ ] Add packaging/release instructions for building the `.app` bundle.
- [ ] Audit all user-visible errors for actionable messages.
- [ ] Decide whether the Objective-C app remains in-tree during transition or moves under a legacy directory after the Rust app reaches parity.

**Automated Verification**:
- [ ] `cargo check` passes.
- [ ] `cargo test` passes.
- [ ] App bundle build command succeeds and produces a launchable `.app`.
- [ ] JSON validation passes for all default/sample config files.
- [ ] Documentation examples that are JSON blocks validate where they are intended to be copy-pasteable configs.

**Manual Verification**:
- [ ] Install/run the built `.app` and verify it appears only in the menu bar.
- [ ] Exercise a mixed config with legacy hosts plus Ghostty and cmux overrides.
- [ ] Verify configure/import/export/about/quit actions from the status menu.
- [ ] Verify first-run, custom config path, alt config, SSH config merge, and reload-on-menu-open behavior with disposable config files.

---

## References

- Research: `docs/agents/research/2026-05-28-rust-port-terminal-support.md`
- Current app state: `Shuttle/AppDelegate.h`
- Startup/config/menu/launch implementation: `Shuttle/AppDelegate.m`
- App bundle metadata: `Shuttle/Shuttle-Info.plist`
- Default config schema/examples: `Shuttle/shuttle.default.json`
- Terminal.app scripts: `apple-scripts/terminal/*.applescript`
- iTerm scripts: `apple-scripts/iTermStable/*.applescript`, `apple-scripts/iTermNightly/*.applescript`
- Virtual/screen script: `apple-scripts/virtual/virtual-with-screen.applescript`
- Ghostty AppleScript docs: https://ghostty.org/docs/features/applescript
- Ghostty scripting definition: https://github.com/ghostty-org/ghostty/blob/65901966/macos/Ghostty.sdef
- cmux docs/API: https://cmux.com/docs, https://cmux.com/docs/api
