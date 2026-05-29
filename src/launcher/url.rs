#[cfg(target_os = "macos")]
pub fn open_url(url: &str) -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .map(|_| ())
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
}
