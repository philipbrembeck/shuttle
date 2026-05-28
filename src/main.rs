mod config;
mod launcher;
#[cfg(target_os = "macos")]
mod macos;
mod menu_model;

fn main() {
    #[cfg(target_os = "macos")]
    macos::app::run();

    #[cfg(not(target_os = "macos"))]
    {
        if let Err(error) = build_menu_entries() {
            let _menu = menu_model::error_menu("Error parsing config");
            eprintln!("Shuttle config error: {error}");
        } else {
            println!("Config loaded OK (macOS app not running on this platform)");
        }
    }
}

pub fn build_menu_entries(
) -> Result<(Vec<menu_model::MenuEntry>, config::model::Config), config::ConfigError> {
    let paths = config::discover_paths()?;
    config::ensure_default_config(&paths)?;
    let before = config::snapshot(&paths);
    let mut config = config::load_merged(&paths)?;
    config::apply_ssh_hosts(&mut config);
    #[cfg(target_os = "macos")]
    let _ = macos::login_item::set_launch_at_login(config.launch_at_login);
    let menu = menu_model::with_separators(menu_model::build(&config.hosts));
    let after = config::snapshot(&paths);
    let _needs_reload = config::needs_reload(&before, &after);
    Ok((menu, config))
}
