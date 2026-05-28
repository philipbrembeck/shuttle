use crate::config::model::{CommandHost, Config};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchTarget {
    New,
    Tab,
    Current,
    Virtual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ITermVersion {
    Stable,
    Nightly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backend {
    TerminalApp,
    ITerm { version: ITermVersion },
    Screen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchKind {
    Url(String),
    Terminal(LaunchRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchRequest {
    pub command: String,
    pub title: String,
    pub theme_or_profile: String,
    pub target: LaunchTarget,
    pub backend: Backend,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LaunchError {
    #[error("'{0}' is not a valid value for inTerminal")]
    InvalidTarget(String),
    #[error("'{0}' is not a valid value for iTerm_version")]
    InvalidITermVersion(String),
}

pub fn normalize(
    config: &Config,
    host: &CommandHost,
    menu_title: &str,
) -> Result<LaunchKind, LaunchError> {
    let target = normalize_target(host.in_terminal.as_deref(), config.open_in.as_deref())?;
    if target != LaunchTarget::Virtual && is_url(&host.cmd) {
        return Ok(LaunchKind::Url(host.cmd.clone()));
    }

    let backend = if target == LaunchTarget::Virtual {
        Backend::Screen
    } else if config
        .terminal
        .as_deref()
        .unwrap_or("Terminal.app")
        .to_lowercase()
        .contains("iterm")
    {
        Backend::ITerm {
            version: normalize_iterm_version(config.iterm_version.as_deref())?,
        }
    } else {
        Backend::TerminalApp
    };

    Ok(LaunchKind::Terminal(LaunchRequest {
        command: host.cmd.clone(),
        title: host.title.clone().unwrap_or_else(|| menu_title.to_string()),
        theme_or_profile: host
            .theme
            .clone()
            .unwrap_or_else(|| default_theme(config, &backend)),
        target,
        backend,
    }))
}

fn normalize_target(
    host_target: Option<&str>,
    config_target: Option<&str>,
) -> Result<LaunchTarget, LaunchError> {
    match host_target
        .or(config_target)
        .unwrap_or("tab")
        .to_lowercase()
        .as_str()
    {
        "new" => Ok(LaunchTarget::New),
        "current" => Ok(LaunchTarget::Current),
        "tab" => Ok(LaunchTarget::Tab),
        "virtual" => Ok(LaunchTarget::Virtual),
        invalid if host_target.is_some() => Err(LaunchError::InvalidTarget(invalid.to_string())),
        _ => Ok(LaunchTarget::Tab),
    }
}

fn normalize_iterm_version(version: Option<&str>) -> Result<ITermVersion, LaunchError> {
    match version.unwrap_or("stable").to_lowercase().as_str() {
        "stable" => Ok(ITermVersion::Stable),
        "nightly" => Ok(ITermVersion::Nightly),
        invalid => Err(LaunchError::InvalidITermVersion(invalid.to_string())),
    }
}

fn default_theme(config: &Config, backend: &Backend) -> String {
    config
        .default_theme
        .clone()
        .unwrap_or_else(|| match backend {
            Backend::ITerm { .. } => "Default".into(),
            Backend::TerminalApp | Backend::Screen => "basic".into(),
        })
}

fn is_url(command: &str) -> bool {
    command.starts_with("http://")
        || command.starts_with("https://")
        || command.starts_with("ssh://")
        || command.starts_with("file://")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn host() -> CommandHost {
        CommandHost {
            cmd: "echo hi".into(),
            name: "Hi".into(),
            in_terminal: None,
            theme: None,
            title: None,
            backend: None,
            strategy: None,
            extra: BTreeMap::new(),
        }
    }

    #[test]
    fn maps_legacy_terminal_app_defaults() {
        let config = Config {
            terminal: Some("Terminal.app".into()),
            open_in: Some("new".into()),
            ..Config::default()
        };
        assert_eq!(
            normalize(&config, &host(), "Menu").unwrap(),
            LaunchKind::Terminal(LaunchRequest {
                command: "echo hi".into(),
                title: "Menu".into(),
                theme_or_profile: "basic".into(),
                target: LaunchTarget::New,
                backend: Backend::TerminalApp
            })
        );
    }

    #[test]
    fn maps_iterm_nightly_and_theme() {
        let config = Config {
            terminal: Some("iTerm".into()),
            iterm_version: Some("nightly".into()),
            default_theme: Some("Homebrew".into()),
            ..Config::default()
        };
        let LaunchKind::Terminal(request) = normalize(&config, &host(), "Menu").unwrap() else {
            panic!("terminal expected")
        };
        assert_eq!(
            request.backend,
            Backend::ITerm {
                version: ITermVersion::Nightly
            }
        );
        assert_eq!(request.theme_or_profile, "Homebrew");
    }

    #[test]
    fn detects_urls_unless_virtual() {
        let mut host = host();
        host.cmd = "https://example.com".into();
        assert_eq!(
            normalize(&Config::default(), &host, "Web").unwrap(),
            LaunchKind::Url("https://example.com".into())
        );
        host.in_terminal = Some("virtual".into());
        assert!(matches!(
            normalize(&Config::default(), &host, "Web").unwrap(),
            LaunchKind::Terminal(_)
        ));
    }

    #[test]
    fn validates_bad_host_target() {
        let mut host = host();
        host.in_terminal = Some("sideways".into());
        assert_eq!(
            normalize(&Config::default(), &host, "Menu"),
            Err(LaunchError::InvalidTarget("sideways".into()))
        );
    }
}
