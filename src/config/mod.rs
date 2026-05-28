pub mod model;
pub mod ssh;

use model::Config;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;

const DEFAULT_CONFIG: &str = include_str!("../../resources/shuttle.default.json");

/// XDG-style default config directory: ~/.config/shuttle/
pub const CONFIG_DIR_NAME: &str = "shuttle";
/// Default config file name inside the config dir.
pub const CONFIG_FILE_NAME: &str = "config.json";
/// Legacy home-root config path kept for migration.
pub const LEGACY_CONFIG_FILE: &str = ".shuttle.json";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not determine the current home directory")]
    MissingHome,
    #[error("I/O error for {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("invalid JSON in {path}: {source}. Validate with `python3 -m json.tool {path}` and fix the syntax before reloading Shuttle.")]
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

pub fn discover_paths() -> Result<ConfigPaths, ConfigError> {
    discover_paths_in(home_dir()?)
}

pub fn discover_paths_in(home: PathBuf) -> Result<ConfigPaths, ConfigError> {
    // Explicit override via ~/.shuttle.path wins over everything.
    let override_file = home.join(".shuttle.path");
    if override_file.exists() {
        let main = read_path_file(&override_file)?;
        let alt = resolve_alt_path(&home);
        return Ok(ConfigPaths {
            main,
            alt,
            used_main_override: true,
        });
    }

    // Preferred location: ~/.config/shuttle/config.json
    let config_dir = home.join(".config").join(CONFIG_DIR_NAME);
    let preferred = config_dir.join(CONFIG_FILE_NAME);

    // Legacy location: ~/.shuttle.json (migrate automatically if found)
    let legacy = home.join(LEGACY_CONFIG_FILE);

    let main = if preferred.exists() {
        preferred
    } else if legacy.exists() {
        // Migrate: copy legacy → preferred location and keep using it
        if let Ok(()) = fs::create_dir_all(&config_dir) {
            let _ = fs::copy(&legacy, &preferred);
        }
        if preferred.exists() {
            preferred
        } else {
            legacy
        }
    } else {
        // Neither exists — first run, use preferred location
        preferred
    };

    let alt = resolve_alt_path(&home);
    Ok(ConfigPaths {
        main,
        alt,
        used_main_override: false,
    })
}

/// Resolve the alternate config path.
fn resolve_alt_path(home: &Path) -> Option<PathBuf> {
    let alt_override = home.join(".shuttle-alt.path");
    if alt_override.exists() {
        read_path_file(&alt_override).ok()
    } else {
        // Check both locations for the alternate config.
        let preferred_alt = home.join(".config").join(CONFIG_DIR_NAME).join("alt.json");
        let legacy_alt = home.join(".shuttle-alt.json");
        if preferred_alt.exists() {
            Some(preferred_alt)
        } else if legacy_alt.exists() {
            Some(legacy_alt)
        } else {
            None
        }
    }
}

/// Write the bundled default config on first run.
pub fn ensure_default_config(paths: &ConfigPaths) -> Result<(), ConfigError> {
    if paths.used_main_override || paths.main.exists() {
        return Ok(());
    }
    // Make sure the directory exists.
    if let Some(parent) = paths.main.parent() {
        fs::create_dir_all(parent).map_err(|source| ConfigError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
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
        if let Ok(mut alt_config) = load_config(alt) {
            config.hosts.append(&mut alt_config.hosts);
        }
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
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn home_dir() -> Result<PathBuf, ConfigError> {
    dirs::home_dir().ok_or(ConfigError::MissingHome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_xdg_config_path_on_first_run() {
        let temp = tempfile::tempdir().unwrap();
        let paths = discover_paths_in(temp.path().to_path_buf()).unwrap();
        assert_eq!(
            paths.main,
            temp.path().join(".config/shuttle/config.json"),
            "default should be ~/.config/shuttle/config.json"
        );
        assert!(!paths.used_main_override);
    }

    #[test]
    fn override_path_file_wins_over_everything() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join(".shuttle.path"), "/tmp/custom.json\n").unwrap();
        let paths = discover_paths_in(temp.path().to_path_buf()).unwrap();
        assert_eq!(paths.main, PathBuf::from("/tmp/custom.json"));
        assert!(paths.used_main_override);
    }

    #[test]
    fn migrates_legacy_config_to_xdg_on_first_run() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = temp.path().join(".shuttle.json");
        fs::write(&legacy, DEFAULT_CONFIG).unwrap();
        let paths = discover_paths_in(temp.path().to_path_buf()).unwrap();
        let preferred = temp.path().join(".config/shuttle/config.json");
        // Should point to the preferred location after migration
        assert_eq!(paths.main, preferred);
        assert!(
            preferred.exists(),
            "config should be copied to XDG location"
        );
    }

    #[test]
    fn uses_xdg_when_it_already_exists() {
        let temp = tempfile::tempdir().unwrap();
        let preferred = temp.path().join(".config/shuttle/config.json");
        fs::create_dir_all(preferred.parent().unwrap()).unwrap();
        fs::write(&preferred, DEFAULT_CONFIG).unwrap();
        let paths = discover_paths_in(temp.path().to_path_buf()).unwrap();
        assert_eq!(paths.main, preferred);
    }

    #[test]
    fn creates_config_dir_on_first_run() {
        let temp = tempfile::tempdir().unwrap();
        let paths = discover_paths_in(temp.path().to_path_buf()).unwrap();
        ensure_default_config(&paths).unwrap();
        assert!(paths.main.exists());
        load_config(&paths.main).unwrap();
    }

    #[test]
    fn alt_config_found_at_xdg_location() {
        let temp = tempfile::tempdir().unwrap();
        let alt = temp.path().join(".config/shuttle/alt.json");
        fs::create_dir_all(alt.parent().unwrap()).unwrap();
        fs::write(&alt, "{}").unwrap();
        let paths = discover_paths_in(temp.path().to_path_buf()).unwrap();
        assert_eq!(paths.alt, Some(alt));
    }

    #[test]
    fn loads_default_config_json() {
        let config: Config = serde_json::from_str(DEFAULT_CONFIG).unwrap();
        assert_eq!(config.terminal.as_deref(), Some("Terminal.app"));
        assert!(!config.hosts.is_empty());
    }

    #[test]
    fn invalid_json_returns_structured_error() {
        let temp = tempfile::tempdir().unwrap();
        let bad = temp.path().join("bad.json");
        fs::write(&bad, "{ nope").unwrap();
        assert!(matches!(load_config(&bad), Err(ConfigError::Json { .. })));
    }
}
