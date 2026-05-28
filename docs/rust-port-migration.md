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

Default config discovery is unchanged:

- `~/.shuttle.path` overrides the main config path.
- Otherwise `~/.shuttle.json` is used and created from the bundled default on first run.
- `~/.shuttle-alt.path` overrides the alternate config path.
- Otherwise `~/.shuttle-alt.json` is used if present.

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
2. Add one per-host `backend` override for Ghostty or cmux.
3. Move to a top-level `backend` only when you want most hosts to use the new backend.
4. Keep legacy `terminal` settings during the transition for fallback behavior.
