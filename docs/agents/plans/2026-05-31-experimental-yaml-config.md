---
date: 2026-05-31T19:36:21.875725+00:00
git_commit: 07049be611a73da6618154ce0c884c4141df8df6
branch: plan/experimental-yaml-config
topic: "Experimental YAML Config Format"
tags: [plan, config, yaml, docs]
status: draft
---

# PLAN: Add Experimental YAML Config Format

Add YAML as an experimental alternative config format for Shuttle while preserving the current JSON format as the stable first-run default and backwards-compatible format. YAML should be opt-in by file presence or explicit override path, not auto-generated as the default config.

## Acceptance Criteria

- JSON remains supported for main and alternate configs.
- First run still creates `~/.config/shuttle/config.json` from `resources/shuttle.default.json`.
- Existing `~/.config/shuttle/config.json`, `~/.config/shuttle/alt.json`, `~/.shuttle.json`, and `~/.shuttle-alt.json` behavior keeps working.
- Standard YAML config paths are supported as experimental alternatives:
  - `~/.config/shuttle/config.yaml`
  - `~/.config/shuttle/config.yml`
  - `~/.config/shuttle/alt.yaml`
  - `~/.config/shuttle/alt.yml`
- Standard YAML paths win over standard JSON paths when both exist.
- Main config standard path precedence is:
  1. `~/.shuttle.path`
  2. `~/.config/shuttle/config.yaml`
  3. `~/.config/shuttle/config.yml`
  4. `~/.config/shuttle/config.json`
  5. legacy `~/.shuttle.json` migration/fallback
- Alternate config standard path precedence is:
  1. `~/.shuttle-alt.path`
  2. `~/.config/shuttle/alt.yaml`
  3. `~/.config/shuttle/alt.yml`
  4. `~/.config/shuttle/alt.json`
  5. legacy `~/.shuttle-alt.json`
- Explicit override paths in `~/.shuttle.path` and `~/.shuttle-alt.path` require `.json`, `.yaml`, or `.yml` extensions.
- Explicit override paths with missing or unknown extensions fail during path discovery with a clear `ConfigError` message.
- Once an alternate config path is selected, alternate config parse/load errors are surfaced instead of silently ignored.
- YAML parsing uses the maintained `yaml_serde` crate, added with Cargo CLI.
- Invalid YAML returns a structured config error with an actionable validation hint.
- `resources/shuttle.example.yaml` exists as a copyable YAML example but is not auto-created.
- User-facing docs clearly explain:
  - YAML is experimental.
  - JSON remains the stable default and compatibility format.
  - YAML wins over JSON when standard YAML files exist.
  - When to choose JSON vs YAML.
  - Override path extension rules.
- Architecture docs include an ADR for format precedence and YAML parser dependency choice.
- Config behavior changes are covered by unit tests.
- `./scripts/check-rust.sh` passes.

## Technical Key Decisions and Tradeoffs

1. **YAML is opt-in but wins when present:** If a standard YAML config exists, Shuttle loads it before the JSON file at the same config role.
   - Why: A user who creates `config.yaml` or `config.yml` has intentionally opted into YAML.
   - Impact: path discovery checks YAML names before JSON names, but default creation still writes JSON.

2. **JSON remains the generated default:** `resources/shuttle.default.json` remains the only first-run default.
   - Why: JSON is the existing, stable, backwards-compatible format.
   - Impact: no first-run YAML migration or conversion is required.

3. **Support both `.yaml` and `.yml`:** Both extensions are accepted and discovered.
   - Why: both are common YAML conventions.
   - Impact: parser selection and path discovery must include both variants.

4. **Explicit override paths require recognized extensions:** Override files do not use content sniffing or parser fallback.
   - Why: extension-based dispatch is predictable and avoids YAML accepting JSON-like input unexpectedly.
   - Impact: extensionless custom paths that previously worked as JSON will now need `.json`; this compatibility-affecting rule must be documented prominently.

5. **Use `yaml_serde`:** Use the maintained YAML Organization fork of `serde_yaml`.
   - Why: Shuttle already uses Serde `Deserialize`; `yaml_serde` offers the lowest-risk Serde-native implementation path and avoids the unmaintained original `serde_yaml` crate.
   - Impact: adds one new direct dependency via `cargo add yaml_serde`; ADR documents this dependency choice and its tradeoffs.

## Current State

Config loading is centralized in `src/config/mod.rs` and uses JSON unconditionally:

```text
main.rs
  └─ build_menu_entries()
      ├─ config::discover_paths()
      │   ├─ ~/.shuttle.path override
      │   ├─ ~/.config/shuttle/config.json
      │   └─ legacy ~/.shuttle.json migration/fallback
      ├─ config::ensure_default_config()
      ├─ config::snapshot()
      ├─ config::load_merged()
      │   ├─ load_config(main) -> serde_json::from_slice()
      │   └─ load_config(alt)  -> serde_json::from_slice()
      ├─ config::apply_ssh_hosts()
      └─ menu_model::build()
```

Relevant current code:

- `src/config/mod.rs:11` embeds `resources/shuttle.default.json`.
- `src/config/mod.rs:16` defines `CONFIG_FILE_NAME` as `config.json`.
- `src/config/mod.rs:52` implements main config discovery.
- `src/config/mod.rs:98` implements alternate config discovery.
- `src/config/mod.rs:117` writes the bundled JSON default on first run.
- `src/config/mod.rs:134` reads bytes and always parses with `serde_json::from_slice`.
- `src/config/model.rs:4` and nested types already derive `Deserialize`, so the typed model can be reused for YAML.

## Desired End State

Config loading dispatches based on extension while preserving the existing typed `Config` model:

```text
build_menu_entries()
  └─ config::load_merged(paths)
      ├─ load_config(main)
      │   ├─ .json       -> serde_json::from_slice::<Config>()
      │   ├─ .yaml/.yml  -> yaml_serde::from_slice::<Config>()
      │   └─ other/none  -> ConfigError::UnsupportedExtension
      └─ load_config(alt) with same parser dispatch
```

Standard discovery after the change:

```text
Main config:
  ~/.shuttle.path
  ~/.config/shuttle/config.yaml
  ~/.config/shuttle/config.yml
  ~/.config/shuttle/config.json
  ~/.shuttle.json

Alternate config:
  ~/.shuttle-alt.path
  ~/.config/shuttle/alt.yaml
  ~/.config/shuttle/alt.yml
  ~/.config/shuttle/alt.json
  ~/.shuttle-alt.json
```

First run remains:

```text
if no override and no config exists:
  create ~/.config/shuttle/config.json
```

## Abstractions and Code Reuse

- `src/config/model.rs` - no expected structural changes; reuse existing Serde `Deserialize` model.
- `src/config/mod.rs`
  - Add a small `ConfigFormat` enum or equivalent helper.
  - Add extension detection helper, e.g. `ConfigFormat::from_path(path: &Path)`.
  - Update `ConfigError` with YAML and unsupported-extension variants.
  - Update `discover_paths_in` and `resolve_alt_path` to prefer YAML standard files.
  - Update `load_config` to dispatch to JSON or YAML parser.
- `resources/shuttle.example.yaml` - add a hand-written, strict example equivalent to README quick-start/default concepts.
- `README.md` - document user-facing rules and examples.
- `docs/rust-port-migration.md` - document compatibility and override extension requirement.
- `docs/ADR/0003-experimental-yaml-config.md` or next available ADR number - document decisions.
- `Cargo.toml` / `Cargo.lock` - add `yaml_serde` via `cargo add yaml_serde`.

## Logging & Observability

No new runtime logging is required. Errors should remain surfaced through existing config error handling. Error messages should identify the path and format, for example:

```text
unsupported config extension for /Users/me/shuttle-config. Use .json, .yaml, or .yml.
invalid YAML in /Users/me/.config/shuttle/config.yaml: <parser error>. Validate with `ruby -e 'require "yaml"; YAML.load_file(ARGV[0])' /Users/me/.config/shuttle/config.yaml` or another YAML linter before reloading Shuttle.
```

If the implementation chooses a different validation command, document the same command in README and error text.

## Implementation

Before implementation, switch from this planning branch to an implementation branch such as `feat/experimental-yaml-config`, unless the implementer intentionally documents a workflow exception.

### Phase 1: Add Format Detection and YAML Parsing

Dependencies: None.

Introduce explicit config format detection and parser dispatch while keeping existing JSON behavior intact.

**Tasks**:

- [x] Add `yaml_serde` using Cargo CLI:
  ```sh
  cargo add yaml_serde
  ```
- [x] Add a private `ConfigFormat` helper in `src/config/mod.rs` that recognizes `.json`, `.yaml`, and `.yml` extensions.
- [x] Add `ConfigError::Yaml { path, source }` with an actionable user-facing error message.
- [x] Add `ConfigError::UnsupportedExtension { path }` or equivalent for missing/unknown extensions.
- [x] Update `load_config(path: &Path)` to parse JSON with `serde_json::from_slice` and YAML with `yaml_serde::from_slice`.
- [x] Update `load_merged` so main config parse/load errors always fail and selected alternate config parse/load errors also fail with their structured error instead of being silently ignored.
- [x] Add unit tests in `src/config/mod.rs` for:
  - [x] valid JSON still loads.
  - [x] valid YAML loads into the same `Config` model.
  - [x] `.yml` loads as YAML.
  - [x] invalid YAML returns `ConfigError::Yaml`.
  - [x] extensionless path returns `ConfigError::UnsupportedExtension`.
  - [x] unknown extension returns `ConfigError::UnsupportedExtension`.

**Automated Verification**:

- [x] `cargo test config::tests::loads_default_config_json`
- [x] New YAML parser unit tests pass.
- [x] `cargo check`

### Phase 2: Update Discovery and Precedence

Dependencies: Phase 1.

Make standard YAML files opt-in and higher priority than JSON files, without changing first-run default creation.

**Tasks**:

- [x] Add constants or helper functions for standard main candidates: `config.yaml`, `config.yml`, `config.json`.
- [x] Update `discover_paths_in` so `~/.config/shuttle/config.yaml` wins over `config.yml`, which wins over `config.json`.
- [x] Preserve legacy `~/.shuttle.json` migration/fallback when no standard config exists.
- [x] Ensure legacy migration still copies `~/.shuttle.json` to `~/.config/shuttle/config.json`, not YAML.
- [x] Update `resolve_alt_path` so `alt.yaml` wins over `alt.yml`, which wins over `alt.json`, then legacy `~/.shuttle-alt.json`.
- [x] Validate recognized extensions while reading `~/.shuttle.path` and `~/.shuttle-alt.path` so discovery errors early for both main and alternate overrides.
- [x] Change `resolve_alt_path` to return `Result<Option<PathBuf>, ConfigError>` or an equivalent error-preserving shape instead of swallowing invalid alternate override paths with `.ok()`.
- [x] Add unit tests for:
  - [x] `config.yaml` beats `config.json`.
  - [x] `config.yml` beats `config.json`.
  - [x] `config.yaml` beats `config.yml` when both exist.
  - [x] first run with no config still targets `config.json`.
  - [x] legacy `.shuttle.json` migration still produces/uses `config.json`.
  - [x] `alt.yaml` beats `alt.json`.
  - [x] `alt.yml` beats `alt.json`.
  - [x] invalid main override extension errors clearly.
  - [x] invalid alternate override extension does not get swallowed silently.

**Automated Verification**:

- [x] Config discovery unit tests pass.
- [x] `cargo test config::tests`
- [x] `cargo check`

### Phase 3: Add YAML Example and Documentation

Dependencies: Phases 1 and 2.

Document the new experimental format thoroughly and add a copyable YAML example.

**Tasks**:

- [x] Add `resources/shuttle.example.yaml` with a copyable example covering:
  - [x] global settings such as `terminal`, `open_in`, and `show_ssh_config_hosts`.
  - [x] simple command/SSH/URL hosts.
  - [x] nested menus.
  - [x] per-host `backend` and `strategy`.
  - [x] at least one YAML comment showing why YAML can be useful for hand-written configs.
- [x] Update `README.md` first-run/config sections to state JSON remains the generated default.
- [x] Add a README subsection for experimental YAML support with exact supported paths and precedence.
- [x] Add README guidance:
  - [x] Use JSON for stable/default/backwards-compatible/machine-generated configs and existing `jq`/`json.tool` workflows.
  - [x] Use YAML for hand-written configs, comments, and deeply nested menus, accepting experimental status.
- [x] Update README custom path docs to state override files must point to `.json`, `.yaml`, or `.yml` paths.
- [x] Update README examples or add a YAML example near the JSON quick-start without removing JSON examples.
- [x] Update `docs/rust-port-migration.md` with compatibility notes and the explicit override extension requirement.
- [x] Update `AGENTS.md` config path guidance if implementation confirms it is stale, especially any statement that the default config is only `~/.shuttle.json`; document any exception if project instructions should remain unchanged.
- [x] Add an ADR under `docs/ADR/` documenting:
  - [x] YAML is experimental and opt-in.
  - [x] YAML standard paths win over JSON standard paths.
  - [x] JSON remains first-run default.
  - [x] both `.yaml` and `.yml` are supported.
  - [x] override paths require recognized extensions.
  - [x] `yaml_serde` was selected over `serde_yaml`, `serde_yml`, `serde-saphyr`, `noyalib`, and manual parsing.
- [x] If docs mention validation commands, ensure the commands are realistic and aligned with error messages.

**Automated Verification**:

- [x] `python3 -m json.tool resources/shuttle.default.json >/dev/null`
- [x] A Rust unit test loads `resources/shuttle.example.yaml` through `load_config` so example validity is checked in CI.
- [x] Documentation links in changed Markdown are valid relative paths.
- [x] `cargo test`
- [x] `./scripts/check-rust.sh`

## Implementation Notes

During implementation, document user feedback, problems, and decisions here.

## References

- `AGENTS.md` - project workflow, testing, dependency, and documentation requirements.
- `src/config/mod.rs` - current config discovery, default creation, parser, and tests.
- `src/config/model.rs` - Serde config model reused for JSON and YAML.
- `resources/shuttle.default.json` - current bundled JSON default.
- `README.md` - user-facing config documentation.
- `docs/rust-port-migration.md` - compatibility guarantees.
- `docs/terminal-backends.md` - backend values for examples if needed.
- `yaml_serde` crate docs: <https://docs.rs/yaml_serde/latest/yaml_serde/>
- `yaml_serde` repository: <https://github.com/yaml/yaml-serde>
