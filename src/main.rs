mod config;
#[cfg(target_os = "macos")]
mod macos;
mod menu_model;

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
    let mut config = config::load_merged(&paths)?;
    let mut ssh_hosts = std::collections::BTreeMap::new();
    if let Ok(hosts) = config::ssh::parse_file(std::path::Path::new("/etc/ssh/ssh_config")) {
        ssh_hosts.extend(hosts);
    }
    if let Some(home) = dirs::home_dir() {
        if let Ok(hosts) = config::ssh::parse_file(&home.join(".ssh/config")) {
            ssh_hosts.extend(hosts);
        }
    }
    config::ssh::merge_hosts(
        &mut config.hosts,
        &ssh_hosts,
        &config.ssh_config_ignore_hosts,
        &config.ssh_config_ignore_keywords,
    );
    let _menu = menu_model::with_separators(menu_model::build(&config.hosts));
    let after = config::snapshot(&paths);
    let _needs_reload = config::needs_reload(&before, &after);
    Ok(())
}
