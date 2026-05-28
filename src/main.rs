mod config;
#[cfg(target_os = "macos")]
mod macos;

fn main() {
    #[cfg(target_os = "macos")]
    macos::app::run();

    #[cfg(not(target_os = "macos"))]
    {
        if let Err(error) = bootstrap_config() {
            eprintln!("Shuttle config error: {error}");
        }
    }
}

fn bootstrap_config() -> Result<(), config::ConfigError> {
    let paths = config::discover_paths()?;
    config::ensure_default_config(&paths)?;
    let before = config::snapshot(&paths);
    let _config = config::load_merged(&paths)?;
    let after = config::snapshot(&paths);
    let _needs_reload = config::needs_reload(&before, &after);
    Ok(())
}
