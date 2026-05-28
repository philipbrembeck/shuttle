# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project overview

Shuttle is a legacy macOS menu-bar app written in Objective-C/Cocoa, with an in-progress Rust native macOS port. It builds a status-bar menu from a user JSON config and SSH config entries, then launches commands in Terminal.app, iTerm, or a virtual/screen-backed mode via bundled compiled AppleScripts. The Rust port should preserve that config contract while introducing typed config/menu/launcher modules.

Primary Objective-C code paths:

- `Shuttle/AppDelegate.m` / `.h` — legacy app startup, config path discovery, menu construction, JSON and SSH config parsing, command dispatch, settings actions.
- `Shuttle/LaunchAtLoginController.m` / `.h` — login item integration. `LaunchAtLoginController.m` is intentionally built with `-fno-objc-arc` in the Xcode project.
- `Shuttle/AboutWindowController.m` / `.h` — About window behavior.
- `Shuttle/shuttle.default.json` — default config copied to `~/.shuttle.json` on first run.
- `Shuttle/apple-scpt/*.scpt` — compiled AppleScript resources included in the app bundle.
- `apple-scripts/**` — source AppleScripts and compile helper scripts.
- `Shuttle/*.lproj/` — localized strings, XIBs, and credits.
- `tests/.shuttle.json` — sample config for manual testing.

Primary Rust port code paths:

- `Cargo.toml` — Rust package definition and small dependency allowlist.
- `src/main.rs` — Rust app entry point.
- `src/config/` — config path discovery, JSON model/loading, default copy, reload timestamp helpers.
- `src/macos/` — native macOS/AppKit shell and lifecycle integration.
- `resources/Shuttle-Info.plist` — Rust app bundle metadata, including `LSUIElement` and Automation usage text.
- `resources/shuttle.default.json` — default config embedded/copied by the Rust port.

## Build and verification

Use Cargo for the Rust port:

```sh
cargo check
cargo test
cargo fmt --check
```

Use Xcode or `xcodebuild` for the legacy Objective-C app on macOS:

```sh
xcodebuild -project Shuttle.xcodeproj -scheme Shuttle -configuration Debug build
xcodebuild -project Shuttle.xcodeproj -scheme Shuttle -configuration Release build
```

The Xcode project has one target and scheme: `Shuttle`. There is no automated Objective-C unit-test target in the repository. Verify Objective-C changes by building and, when possible, running the app manually with a disposable Shuttle config.

For local manual testing, prefer an isolated config file rather than changing a real user config:

```sh
cp tests/.shuttle.json /tmp/shuttle-test.json
printf '/tmp/shuttle-test.json\n' > ~/.shuttle.path
# Remove ~/.shuttle.path after testing to restore default behavior.
```

If testing alternate-config behavior, use `~/.shuttle-alt.path` or `~/.shuttle-alt.json` similarly, and clean them up afterward.

## AppleScript resources

The app uses compiled `.scpt` files in `Shuttle/apple-scpt/`. Source scripts live under `apple-scripts/` and are compiled by helper scripts such as:

```sh
./apple-scripts/compile-Terminal.sh
./apple-scripts/compile-iTermStable.sh
./apple-scripts/compile-iTermNightly.sh
./apple-scripts/compile-Virtual.sh
```

Caution: these scripts contain hard-coded paths like `~/Git/shuttle/...`. Adjust locally or run from an environment where those paths are valid before relying on them. If you change AppleScript source, ensure the corresponding compiled `.scpt` resource is regenerated and included in the Xcode project.

## Coding conventions

- Match the existing Objective-C style in legacy files: Cocoa APIs, explicit `NSString`/`NSDictionary`/`NSMutableArray` types, and existing brace/spacing conventions.
- For Rust code, keep modules small and typed around legacy concepts (`Config`, host entries, menu model, launch requests). Prefer explicit error types and tests for compatibility behavior.
- Preserve ARC assumptions. Most files are under ARC; `LaunchAtLoginController.m` is explicitly non-ARC via Xcode build settings.
- Keep compatibility in mind. The project carries old macOS deployment-target settings and legacy AppKit behavior checks; avoid introducing APIs without availability checks.
- Prefer small, targeted changes in `AppDelegate.m`; it is large and mixes responsibilities, so avoid broad refactors unless requested.
- Use localized strings for user-visible UI text when adding or changing app-facing copy. Update each relevant `.lproj` where practical, and keep Base/en behavior sane.
- Keep JSON examples valid. Validate edited config files with `python -m json.tool <file>` or equivalent.
- Do not commit personal config files (`~/.shuttle.json`, `.shuttle.path`, etc.) or machine-specific Xcode user data.

## Runtime behavior to preserve

- Default config path is `~/.shuttle.json`; `~/.shuttle.path` can override it.
- Alternate config can be enabled by `~/.shuttle-alt.path` or default `~/.shuttle-alt.json` if present.
- On first run, `Shuttle/shuttle.default.json` is copied to the default config path if missing in the Objective-C app; the Rust port embeds/copies `resources/shuttle.default.json`.
- The menu reloads when the main config, alternate config, `/etc/ssh/ssh_config`, or `~/.ssh/config` modification times change.
- SSH config parsing supports comments of the form `# shuttle.<key> <value>` for per-host metadata and supports `Include` handling.
- `hosts` entries can be nested menus, commands, separators via `[---]` in names, and sorted through name prefixes.
- Commands may open URLs directly, open in Terminal/iTerm windows or tabs, or use the virtual/screen script depending on config.

## Repository hygiene

- Build artifacts belong under `build/` and are ignored.
- Do not add `xcuserdata`, `.xcworkspace`, `.pbxuser`, or other local Xcode state.
- Be careful editing `Shuttle.xcodeproj/project.pbxproj`; keep changes minimal and review diffs for accidental UUID churn.
- Keep binary assets (`.png`, `.icns`, `.psd`, `.scpt`, `.xib`) untouched unless the task specifically requires them.

## Suggested checks before finishing

Run the relevant subset:

```sh
python -m json.tool Shuttle/shuttle.default.json >/dev/null
python -m json.tool resources/shuttle.default.json >/dev/null
python -m json.tool tests/.shuttle.json >/dev/null
cargo test
xcodebuild -project Shuttle.xcodeproj -scheme Shuttle -configuration Debug build
```

If you cannot run Xcode-based checks, state that clearly in your final response and mention any lighter validation you did run.
