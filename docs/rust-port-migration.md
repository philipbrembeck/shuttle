# Rust Port Migration Guide

Existing `.shuttle.json` files are intended to keep working without edits.

## Compatible legacy keys

The Rust port preserves these keys:

- `terminal`
- `iTerm_version`
- `open_in`
- `default_theme`
- `editor`
- `launch_at_login`
- `show_ssh_config_hosts`
- `ssh_config_ignore_hosts`
- `ssh_config_ignore_keywords`
- per-host `cmd`, `name`, `inTerminal`, `theme`, and `title`

## Config discovery compatibility

Default first-run creation now writes the bundled JSON default to `~/.config/shuttle/config.json`. Legacy `~/.shuttle.json` files are still migrated automatically to that JSON path when no standard config exists yet.

Main config precedence is:

1. `~/.shuttle.path`
2. `~/.config/shuttle/config.yaml`
3. `~/.config/shuttle/config.yml`
4. `~/.config/shuttle/config.json`
5. legacy `~/.shuttle.json` migration/fallback

Alternate config precedence is:

1. `~/.shuttle-alt.path`
2. `~/.config/shuttle/alt.yaml`
3. `~/.config/shuttle/alt.yml`
4. `~/.config/shuttle/alt.json`
5. legacy `~/.shuttle-alt.json`

YAML support is experimental and opt-in. JSON remains the stable default and compatibility format. Standard YAML paths win over standard JSON paths because creating one is treated as an explicit opt-in.

Explicit override files (`~/.shuttle.path` and `~/.shuttle-alt.path`) must point to `.json`, `.yaml`, or `.yml` files. Extensionless or unknown-extension override paths now fail during config discovery with a clear error instead of being treated as JSON.

## New optional keys

`backend` and `strategy` may be set globally or per host. Per-host values win over top-level values. If neither is present, legacy `terminal`, `iTerm_version`, and `open_in` decide the launch behavior.

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

## Recommended migration path

1. Run the Rust build against your existing config without adding new keys.
2. Keep JSON if you rely on stable/default/backwards-compatible behavior, generated configs, or `jq`/`json.tool` workflows.
3. Try YAML only for hand-written configs where comments and nesting are worth the experimental status.
4. Add one per-host `backend` override for Ghostty or cmux.
5. Move to a top-level `backend` only when you want most hosts to use the new backend.
6. Keep legacy `terminal` settings during the transition for fallback behavior.
