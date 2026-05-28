# Rust Port Troubleshooting

## Config parse errors

If the menu shows or logs an invalid JSON error, validate your config:

```sh
python3 -m json.tool ~/.shuttle.json >/dev/null
```

If you use `~/.shuttle.path`, validate the file named in that path instead.

## Automation permission

AppleScript backends require macOS Automation permission. If Terminal.app, iTerm, or Ghostty AppleScript actions fail:

1. Open System Settings → Privacy & Security → Automation.
2. Find Shuttle.
3. Enable the target terminal application.
4. Restart Shuttle and retry.

## Missing terminal apps

- `ghostty-open` and `ghostty-applescript` require `Ghostty.app` in `/Applications`.
- iTerm backends require iTerm installed and the matching `iTerm_version` / backend value.
- `screen` mode requires `/usr/bin/screen` or a compatible `screen` in PATH.

## cmux socket access

`cmux-socket` uses a Unix socket, defaulting to `/tmp/cmux.sock`. Override it with:

```sh
export CMUX_SOCKET_PATH=/path/to/cmux.sock
```

If cmux blocks external local clients, start/configure cmux with:

```sh
CMUX_SOCKET_MODE=allowAll
```

Use `cmux-cli` if you cannot enable socket access.
