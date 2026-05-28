# Shuttle

A native macOS menu-bar app for launching SSH commands, terminal sessions, and URLs from a simple JSON config.

## Installation

```sh
./scripts/build-rust-app.sh
cp -r target/release/Shuttle.app /Applications/
```

Or run directly:

```sh
open target/release/Shuttle.app
```

## Configuration

Shuttle reads `~/.shuttle.json` on first launch and creates it from a bundled default if missing. Override the config path by writing a file path into `~/.shuttle.path`:

```sh
echo '/path/to/my/shuttle.json' > ~/.shuttle.path
```

### Alternate config

An optional second config can add extra hosts. Shuttle checks `~/.shuttle-alt.path` (custom path) or `~/.shuttle-alt.json` (default location). Hosts from the alternate config are appended to the main host list.

### Config format

```json
{
  "editor": "default",
  "launch_at_login": false,
  "terminal": "Terminal.app",
  "iTerm_version": "stable",
  "default_theme": "Homebrew",
  "open_in": "new",
  "show_ssh_config_hosts": true,
  "ssh_config_ignore_hosts": ["bastion"],
  "ssh_config_ignore_keywords": ["internal"],
  "hosts": [
    {
      "cmd": "ssh prod.example.com",
      "name": "Production"
    },
    {
      "My Servers": [
        {
          "cmd": "ssh staging.example.com",
          "name": "Staging",
          "inTerminal": "tab",
          "theme": "Ocean",
          "title": "Staging Server"
        }
      ]
    }
  ]
}
```

### Global settings

| Key | Description | Values |
|-----|-------------|--------|
| `editor` | App to open config with via Configure... | `"default"` (system default), or a terminal editor name like `"nano"`, `"vi"` |
| `launch_at_login` | Start Shuttle on login | `true` / `false` |
| `terminal` | Legacy terminal selection | `"Terminal.app"` or `"iTerm"` |
| `iTerm_version` | iTerm variant | `"stable"` or `"nightly"` |
| `default_theme` | Default terminal profile/theme | Any string (e.g. `"Homebrew"`, `"Default"`) |
| `open_in` | Default window mode | `"new"`, `"tab"` |
| `show_ssh_config_hosts` | Import hosts from SSH config | `true` / `false` |
| `ssh_config_ignore_hosts` | Exact host names to skip | Array of strings |
| `ssh_config_ignore_keywords` | Substrings to filter out | Array of strings |
| `backend` | Global launch backend (optional, new) | See [Backends](#backends) |
| `strategy` | Global launch strategy (optional, new) | See [Strategies](#strategies) |

### Per-host settings

| Key | Description |
|-----|-------------|
| `cmd` | Command to execute (required) |
| `name` | Display name in menu (required) |
| `inTerminal` | Override window mode: `"new"`, `"tab"`, `"current"`, `"virtual"` |
| `theme` | Override terminal profile/theme |
| `title` | Override terminal window/tab title |
| `backend` | Override launch backend for this host |
| `strategy` | Override launch strategy for this host |

### Nested menus

Wrap hosts in a named object to create submenus:

```json
{
  "hosts": [
    {
      "Production": [
        { "cmd": "ssh web1", "name": "Web 1" },
        { "cmd": "ssh web2", "name": "Web 2" }
      ]
    }
  ]
}
```

Submenus can be nested to any depth.

### Sorting and separators

Prefix a name with `[aaa]` (any three lowercase letters) to control sort order — the prefix is stripped from the displayed name. Add `[---]` to insert a separator line after that item:

```json
{ "cmd": "ssh prod", "name": "[aaa][---]Production" }
```

### URL commands

If `cmd` starts with `http://`, `https://`, `ssh://`, or `file://`, Shuttle opens it in the default browser/handler instead of a terminal.

### SSH config import

When `show_ssh_config_hosts` is `true` (or omitted), Shuttle reads `~/.ssh/config` and `/etc/ssh/ssh_config`. Imported hosts appear as menu items with `ssh <alias>` commands.

SSH config features supported:

- `Host` directives (first alias used)
- `Include` directives
- Special comments: `# shuttle.name Folder/Display Name` sets the menu path/name

Hosts with wildcards (`*`), dot-prefixed names, or matching ignore lists are filtered out. Use `/` in shuttle.name to create nested menu paths:

```
Host prod
  HostName prod.example.com
  # shuttle.name Servers/Production
```

## Backends

Shuttle supports multiple terminal backends. Set `backend` globally or per-host. If omitted, legacy `terminal` / `iTerm_version` settings are used.

### Terminal.app (default)

```json
{ "terminal": "Terminal.app" }
```

Or explicitly:

```json
{ "backend": "terminal-app" }
```

Supports `new`, `tab`, `current`, and `virtual` modes via bundled AppleScripts.

### iTerm

```json
{ "terminal": "iTerm", "iTerm_version": "stable" }
```

Or explicitly:

```json
{ "backend": "iterm-stable" }
{ "backend": "iterm-nightly" }
```

Supports `new`, `tab`, `current`, and `virtual` modes via bundled AppleScripts.

### Ghostty

Ghostty is supported through two backends:

#### ghostty-open

Launches a new Ghostty window using `open -na Ghostty.app --args`:

```json
{
  "backend": "ghostty-open",
  "hosts": [
    { "cmd": "ssh prod", "name": "Prod" }
  ]
}
```

- Only supports `new` window mode
- Does not require Automation permission
- Requires Ghostty.app in `/Applications`

#### ghostty-applescript

Uses Ghostty's AppleScript API (requires Ghostty 1.3+):

```json
{
  "backend": "ghostty-applescript",
  "hosts": [
    { "cmd": "ssh prod", "name": "Prod", "inTerminal": "tab" }
  ]
}
```

- Supports `new`, `tab`, and `current` modes
- Requires macOS Automation permission for Ghostty
- Grant permission in System Settings → Privacy & Security → Automation

### cmux

cmux is supported through two backends:

#### cmux-cli

Spawns the `cmux` binary directly:

```json
{
  "backend": "cmux-cli",
  "hosts": [
    { "cmd": "ssh prod", "name": "Prod", "strategy": "workspace" }
  ]
}
```

- Looks for cmux in `/Applications/cmux.app/Contents/Resources/bin/cmux` or `PATH`
- Override with `CMUX_BINARY` environment variable
- `new`/`tab` targets send to a workspace, `current` sends to the focused surface

#### cmux-socket

Communicates with cmux through its Unix socket JSON API:

```json
{
  "backend": "cmux-socket",
  "hosts": [
    { "cmd": "ssh prod", "name": "Prod", "strategy": "socket" }
  ]
}
```

- Default socket path: `/tmp/cmux.sock`
- Override with `CMUX_SOCKET_PATH` environment variable
- Requires cmux socket access — start cmux with `CMUX_SOCKET_MODE=allowAll` if external access is blocked

### Virtual / Screen

Run commands in a detached `screen` session (no visible terminal):

```json
{ "cmd": "long-task.sh", "name": "Background Task", "inTerminal": "virtual" }
```

Or explicitly:

```json
{ "cmd": "long-task.sh", "name": "Background Task", "backend": "screen" }
```

## Strategies

The `strategy` key hints at how the backend should handle the command:

| Strategy | Description |
|----------|-------------|
| `default` | Backend-specific default behavior |
| `workspace` | Target a named workspace (cmux) |
| `socket` | Use socket API (cmux) |
| `applescript` | Use AppleScript automation |

## Backend precedence

1. Per-host `backend` / `strategy`
2. Top-level `backend` / `strategy`
3. Legacy `terminal`, `iTerm_version`, and `open_in`
4. Default: Terminal.app, tab mode

## Menu actions

Click the status bar icon to access:

- Your configured hosts and SSH imports
- **Configure...** — open config in your editor
- **Import...** — replace config from a file
- **Export...** — save config to a file
- **About Shuttle**
- **Quit**

## Build from source

Requirements: Rust toolchain, macOS

```sh
./scripts/check-rust.sh   # format, lint, test
./scripts/build-rust-app.sh
```

Output: `target/release/Shuttle.app`

## Testing with isolated config

```sh
cp tests/.shuttle.json /tmp/shuttle-test.json
printf '/tmp/shuttle-test.json\n' > ~/.shuttle.path
open target/release/Shuttle.app
# Remove ~/.shuttle.path when done
```

## Troubleshooting

See `docs/troubleshooting-rust-port.md` for common issues including:

- Config parse errors
- Automation permission
- Missing terminal apps
- cmux socket access
