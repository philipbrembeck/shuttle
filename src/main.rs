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
        if let Err(error) = bootstrap_config() {
            let _menu = menu_model::error_menu("Error parsing config");
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
    #[cfg(target_os = "macos")]
    let _ = macos::login_item::set_launch_at_login(config.launch_at_login);
    let _menu = menu_model::with_separators(menu_model::build(&config.hosts));
    collect_launch_requests(&config, &config.hosts);
    let after = config::snapshot(&paths);
    let _needs_reload = config::needs_reload(&before, &after);
    Ok(())
}

fn collect_launch_requests(config: &config::model::Config, entries: &[config::model::HostEntry]) {
    for entry in entries {
        match entry {
            config::model::HostEntry::Command(command) => {
                let Ok(kind) = launcher::normalize(config, command, &command.name) else {
                    continue;
                };
                match kind {
                    launcher::LaunchKind::Url(url) => {
                        let _ = launcher::url::open_url(&url);
                    }
                    launcher::LaunchKind::Terminal(request) => match &request.backend {
                        launcher::Backend::TerminalApp => {
                            let _ = launcher::terminal_app::applescript_resource(&request);
                            let _ = launcher::terminal_app::script_parameters(&request);
                        }
                        launcher::Backend::ITerm { version } => {
                            let _ = launcher::iterm::applescript_resource(&request, version);
                            let _ = launcher::iterm::script_parameters(&request);
                        }
                        launcher::Backend::GhosttyOpen => {
                            let _ = launcher::ghostty::detect_application();
                            let _ = launcher::ghostty::open_args(&request);
                        }
                        launcher::Backend::GhosttyAppleScript => {
                            let _ = launcher::ghostty::detect_application();
                            let _ = launcher::ghostty::automation_denied_error();
                            let _ = launcher::ghostty::applescript_source(&request);
                        }
                        launcher::Backend::CmuxCli => {
                            if let Ok(binary) = launcher::cmux::default_binary() {
                                let _ = launcher::cmux::cli_args(binary, &request);
                            }
                        }
                        launcher::Backend::CmuxSocket => {
                            if let Ok(path) = launcher::cmux::socket_path() {
                                let payload = launcher::cmux::socket_launch_request(1, &request);
                                let _ = launcher::cmux::send_socket_request(&path, &payload);
                            }
                        }
                        launcher::Backend::Screen => {
                            let _ = launcher::virtual_screen::screen_args(&request);
                        }
                    },
                }
            }
            config::model::HostEntry::Menu(children) => {
                for child_entries in children.values() {
                    collect_launch_requests(config, child_entries);
                }
            }
        }
    }
}
