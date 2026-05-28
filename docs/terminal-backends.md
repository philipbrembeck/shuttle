# Terminal Backends

Shuttle remains compatible with existing `.shuttle.json` files. If no `backend` is set, legacy `terminal`, `iTerm_version`, and `open_in` values are used.

Backend precedence:

1. Per-host `backend` / `strategy`
2. Top-level `backend` / `strategy`
3. Legacy `terminal`, `iTerm_version`, and `open_in`
4. Defaults (`Terminal.app`, `tab`)

Supported backend values planned for the Rust port:

- `terminal-app`
- `iterm-stable`
- `iterm-nightly`
- `ghostty-open`
- `ghostty-applescript`
- `cmux-cli`
- `cmux-socket`
- `screen`

Supported strategy values:

- `default`
- `workspace`
- `socket`
- `applescript`

Example per-host override:

```json
{
  "terminal": "Terminal.app",
  "open_in": "new",
  "hosts": [
    {
      "cmd": "ssh prod",
      "name": "Prod",
      "backend": "ghostty-open"
    },
    {
      "cmd": "ssh pair",
      "name": "Pairing",
      "backend": "cmux-cli",
      "strategy": "workspace"
    }
  ]
}
```
