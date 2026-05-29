use std::process::Command;

/// Top-level dispatch. `backend` is a resolved backend string (e.g. "ghostty-open",
/// "terminal-app", "iterm-stable"). `cmd` is the raw command from the config.
pub fn execute(cmd: &str, backend: &str) {
    let cmd = normalize_cmd(cmd);
    match backend {
        "ghostty-open" => launch_ghostty_open(&cmd),
        "ghostty-applescript" => launch_ghostty_applescript(&cmd),
        "iterm-stable" => launch_iterm(&cmd, false),
        "iterm-nightly" => launch_iterm(&cmd, true),
        "cmux-cli" => launch_cmux_cli(&cmd),
        "cmux-socket" => launch_cmux_socket(&cmd),
        "screen" => launch_screen(&cmd),
        // url: prefix injected by menu builder for URL commands
        url if url.starts_with("url:") => launch_url(&url["url:".len()..]),
        // terminal-app and anything unknown fall through to Terminal.app
        _ => launch_terminal_app(&cmd),
    }
}

// ── Command normalisation ─────────────────────────────────────────────────────

/// Collapse multi-line commands (JSON \n) into a single shell-safe command.
fn normalize_cmd(cmd: &str) -> String {
    let lines: Vec<&str> = cmd
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() == 1 {
        lines[0].to_string()
    } else {
        lines.join(" && ")
    }
}

// ── URL ───────────────────────────────────────────────────────────────────────

fn launch_url(url: &str) {
    Command::new("open").arg(url).spawn().ok();
}

// ── Ghostty ───────────────────────────────────────────────────────────────────

fn launch_ghostty_open(cmd: &str) {
    // open -na Ghostty.app --args -e sh -c "cmd"
    // Using sh -c so multi-part commands and quoting work correctly.
    Command::new("open")
        .args(["-na", "Ghostty.app", "--args", "-e", "sh", "-c", cmd])
        .spawn()
        .ok();
}

fn launch_ghostty_applescript(cmd: &str) {
    let escaped = escape_for_applescript(cmd);
    let script = format!(r#"tell application "Ghostty" to create window with command "{escaped}""#);
    run_osascript(&script);
}

// ── Terminal.app ──────────────────────────────────────────────────────────────

fn launch_terminal_app(cmd: &str) {
    let escaped = escape_for_applescript(cmd);
    // do script in a brand-new window; activate brings Terminal to front
    let script = format!(
        r#"tell application "Terminal"
    activate
    do script "{escaped}"
end tell"#
    );
    run_osascript(&script);
}

// ── iTerm ─────────────────────────────────────────────────────────────────────

fn launch_iterm(cmd: &str, _nightly: bool) {
    let escaped = escape_for_applescript(cmd);
    // Works for both stable and nightly; nightly flag reserved for future profile differences
    let script = format!(
        r#"tell application "iTerm2"
    activate
    create window with default profile command "{escaped}"
end tell"#
    );
    run_osascript(&script);
}

// ── cmux ──────────────────────────────────────────────────────────────────────

fn launch_cmux_cli(cmd: &str) {
    let binary = crate::launcher::cmux::default_binary()
        .unwrap_or_else(|_| std::path::PathBuf::from("cmux"));
    Command::new(binary).args(["run", cmd]).spawn().ok();
}

fn launch_cmux_socket(cmd: &str) {
    if let Ok(path) = crate::launcher::cmux::socket_path() {
        let payload = crate::launcher::cmux::socket_request(
            1,
            "surface.send",
            serde_json::json!({ "text": cmd }),
        );
        crate::launcher::cmux::send_socket_request(&path, &payload).ok();
    }
}

// ── Virtual / screen ─────────────────────────────────────────────────────────

fn launch_screen(cmd: &str) {
    Command::new("screen")
        .args(["-d", "-m", "-S", "shuttle", "sh", "-lc", cmd])
        .spawn()
        .ok();
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Escape a string for embedding inside an AppleScript double-quoted string.
fn escape_for_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn run_osascript(script: &str) {
    Command::new("osascript").arg("-e").arg(script).spawn().ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_multiline_commands() {
        assert_eq!(normalize_cmd("ssh host\ncd /var"), "ssh host && cd /var");
        assert_eq!(normalize_cmd("  ssh host  "), "ssh host");
    }

    #[test]
    fn escapes_quotes_and_backslashes() {
        assert_eq!(escape_for_applescript(r#"echo "hi""#), r#"echo \"hi\""#);
    }
}
