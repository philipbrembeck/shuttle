fn open_command(url: &str) -> std::process::Command {
    let mut command = std::process::Command::new("open");
    command.arg(url);
    command
}

#[cfg(target_os = "macos")]
pub fn open_url(url: &str) -> std::io::Result<()> {
    open_command(url).spawn().map(|_| ())
}

#[cfg(not(target_os = "macos"))]
pub fn open_url(url: &str) -> std::io::Result<()> {
    let _ = url;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn non_macos_open_is_noop_for_testability() {
        #[cfg(not(target_os = "macos"))]
        super::open_url("https://example.com").unwrap();
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
