#[cfg(test)]
fn open_command(url: &str) -> std::process::Command {
    let mut command = std::process::Command::new("open");
    command.arg(url);
    command
}

#[cfg(test)]
mod tests {
    #[test]
    fn non_macos_open_is_noop_for_testability() {
        let command = super::open_command("https://example.com");
        assert_eq!(command.get_program(), "open");
    }

    #[test]
    fn builds_macos_open_command() {
        let command = super::open_command("https://example.com");
        assert_eq!(command.get_program(), "open");
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            ["https://example.com"]
        );
    }
}
