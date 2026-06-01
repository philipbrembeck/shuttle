# ADR 0003: Experimental YAML config

## Status

Accepted

## Context

Shuttle's stable configuration format is JSON. Users who maintain large nested menus by hand benefit from comments and reduced punctuation, but first-run behavior and existing `.shuttle.json` compatibility must remain stable.

## Decision

- YAML is supported as an experimental, opt-in config format.
- First run still creates `~/.config/shuttle/config.json` from `resources/shuttle.default.json`.
- Standard YAML paths win over standard JSON paths when present:
  - main: `config.yaml`, then `config.yml`, then `config.json`
  - alternate: `alt.yaml`, then `alt.yml`, then `alt.json`
- Legacy `~/.shuttle.json` and `~/.shuttle-alt.json` remain fallbacks.
- Explicit override paths in `~/.shuttle.path` and `~/.shuttle-alt.path` must end in `.json`, `.yaml`, or `.yml`.
- Parser dispatch is extension-based; Shuttle does not sniff content or fall back between parsers.
- YAML parsing uses `yaml_serde`.

## Parser choice

`yaml_serde` was selected because it is a maintained YAML Organization fork with Serde-native deserialization, matching Shuttle's existing typed `Config` model.

Alternatives considered:

- `serde_yaml`: familiar API, but unmaintained.
- `serde_yml`: Serde-native, but less clearly aligned with the YAML Organization fork path.
- `serde-saphyr`: viable YAML stack, but would add more adaptation work for Shuttle's simple Serde model.
- `noyalib`: not needed for this config-loading use case.
- Manual parsing: higher maintenance burden and greater compatibility risk.

## Consequences

Users can keep using JSON without changes. Users who create standard YAML files intentionally opt in and see YAML take precedence over JSON. Extensionless override paths that previously parsed as JSON now fail clearly and must be renamed or updated to a supported extension.
