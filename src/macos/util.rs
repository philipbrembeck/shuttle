/// Escape a string for embedding inside an AppleScript double-quoted string.
pub fn escape_for_applescript(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
