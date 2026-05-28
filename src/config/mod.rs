pub mod model;

use model::Config;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;

const DEFAULT_CONFIG: &str = include_str!("../../resources/shuttle.default.json");

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not determine the current home directory")]
    MissingHome,
    #[error("I/O error for {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("invalid JSON in {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPaths {
    pub main: PathBuf,
    pub alt: Option<PathBuf>,
    pub used_main_override: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReloadSnapshot {
    pub main_config: Option<SystemTime>,
    pub alt_config: Option<SystemTime>,
    pub ssh_system: Option<SystemTime>,
    pub ssh_user: Option<SystemTime>,
}

pub fn load_default() -> Result<Config, ConfigError> {
    let paths = discover_paths()?;
    ensure_default_config(&paths)?;
    load_config(&paths.main)
}

pub fn discover_paths() -> Result<ConfigPaths, ConfigError> {
    discover_paths_in(home_dir()?)
}

pub fn discover_paths_in(home: PathBuf) -> Result<ConfigPaths, ConfigError> {
    let main_override = home.join(".shuttle.path");
    let (main, used_main_override) = if main_override.exists() {
        (read_path_file(&main_override)?, true)
    } else {
        (home.join(".shuttle.json"), false)
    };

    let alt_override = home.join(".shuttle-alt.path");
    let default_alt = home.join(".shuttle-alt.json");
    let alt = if alt_override.exists() {
        Some(read_path_file(&alt_override)?)
    } else if default_alt.exists() {
        Some(default_alt)
    } else {
        None
    };

    Ok(ConfigPaths {
        main,
        alt,
        used_main_override,
    })
}

pub fn ensure_default_config(paths: &ConfigPaths) -> Result<(), ConfigError> {
    if paths.used_main_override || paths.main.exists() {
        return Ok(());
    }
    fs::write(&paths.main, DEFAULT_CONFIG).map_err(|source| ConfigError::Io {
        path: paths.main.clone(),
        source,
    })
}

pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let bytes = fs::read(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| ConfigError::Json {
        path: path.to_path_buf(),
        source,
    })
}

pub fn load_merged(paths: &ConfigPaths) -> Result<Config, ConfigError> {
    let mut config = load_config(&paths.main)?;
    if let Some(alt) = &paths.alt {
        let mut alt_config = load_config(alt)?;
        config.hosts.append(&mut alt_config.hosts);
    }
    Ok(config)
}

pub fn snapshot(paths: &ConfigPaths) -> ReloadSnapshot {
    ReloadSnapshot {
        main_config: modified(&paths.main),
        alt_config: paths.alt.as_deref().and_then(modified),
        ssh_system: modified(Path::new("/etc/ssh/ssh_config")),
        ssh_user: home_dir()
            .ok()
            .and_then(|home| modified(&home.join(".ssh/config"))),
    }
}

pub fn needs_reload(old: &ReloadSnapshot, new: &ReloadSnapshot) -> bool {
    new.main_config > old.main_config
        || new.alt_config > old.alt_config
        || new.ssh_system > old.ssh_system
        || new.ssh_user > old.ssh_user
}

fn read_path_file(path: &Path) -> Result<PathBuf, ConfigError> {
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(PathBuf::from(contents.trim()))
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}

fn home_dir() -> Result<PathBuf, ConfigError> {
    dirs::home_dir().ok_or(ConfigError::MissingHome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_default_config_path() {
        let temp = tempfile::tempdir().unwrap();
        let paths = discover_paths_in(temp.path().to_path_buf()).unwrap();
        assert_eq!(paths.main, temp.path().join(".shuttle.json"));
        assert_eq!(paths.alt, None);
        assert!(!paths.used_main_override);
    }

    #[test]
    fn discovers_main_override_path() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join(".shuttle.path"), "/tmp/custom.json\n").unwrap();
        let paths = discover_paths_in(temp.path().to_path_buf()).unwrap();
        assert_eq!(paths.main, PathBuf::from("/tmp/custom.json"));
        assert!(paths.used_main_override);
    }

    #[test]
    fn discovers_alt_override_and_default_alt() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join(".shuttle-alt.json"), "{}").unwrap();
        let paths = discover_paths_in(temp.path().to_path_buf()).unwrap();
        assert_eq!(paths.alt, Some(temp.path().join(".shuttle-alt.json")));

        fs::write(temp.path().join(".shuttle-alt.path"), "/tmp/alt.json\n").unwrap();
        let paths = discover_paths_in(temp.path().to_path_buf()).unwrap();
        assert_eq!(paths.alt, Some(PathBuf::from("/tmp/alt.json")));
    }

    #[test]
    fn copies_default_config_on_first_run() {
        let temp = tempfile::tempdir().unwrap();
        let paths = discover_paths_in(temp.path().to_path_buf()).unwrap();
        ensure_default_config(&paths).unwrap();
        assert!(paths.main.exists());
        load_config(&paths.main).unwrap();
    }

    #[test]
    fn loads_legacy_default_json() {
        let config: Config = serde_json::from_str(DEFAULT_CONFIG).unwrap();
        assert_eq!(config.terminal.as_deref(), Some("Terminal.app"));
        assert!(!config.hosts.is_empty());
    }

    #[test]
    fn invalid_json_returns_structured_error() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("bad.json");
        fs::write(&config_path, "{ nope").unwrap();
        assert!(matches!(
            load_config(&config_path),
            Err(ConfigError::Json { .. })
        ));
    }
}
