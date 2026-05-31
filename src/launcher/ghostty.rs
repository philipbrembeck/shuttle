use super::{LaunchRequest, LaunchTarget};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GhosttyError {
    #[error(
        "Ghostty.app was not found in /Applications; install Ghostty or choose another backend"
    )]
    MissingApplication,
    #[error("ghostty-open supports only new-window launches; use ghostty-applescript for {0:?}")]
    UnsupportedOpenTarget(LaunchTarget),
    #[error("Ghostty AppleScript requires Ghostty 1.3+ and macOS Automation permission")]
    AppleScriptUnavailable,
}

pub fn detect_application() -> Result<(), GhosttyError> {
    if std::path::Path::new("/Applications/Ghostty.app").exists() {
        Ok(())
    } else {
        Err(GhosttyError::MissingApplication)
    }
}

pub fn automation_denied_error() -> GhosttyError {
    GhosttyError::AppleScriptUnavailable
}

pub fn open_args(request: &LaunchRequest) -> Result<Vec<String>, GhosttyError> {
    if request.target != LaunchTarget::New {
        return Err(GhosttyError::UnsupportedOpenTarget(request.target.clone()));
    }

    let mut args = vec![
        "open".into(),
        "-na".into(),
        "Ghostty.app".into(),
        "--args".into(),
    ];
    args.extend([
        "--title".into(),
        request.title.clone(),
        "-e".into(),
        request.command.clone(),
    ]);
    Ok(args)
}

pub fn applescript_source(request: &LaunchRequest) -> String {
    let command = request.command.replace('"', "\\\"");
    match request.target {
        LaunchTarget::New => {
            format!("tell application \"Ghostty\" to create window with command \"{command}\"")
        }
        LaunchTarget::Tab => {
            format!("tell application \"Ghostty\" to create tab with command \"{command}\"")
        }
        LaunchTarget::Current => {
            format!("tell application \"Ghostty\" to run command \"{command}\"")
        }
        LaunchTarget::Virtual => {
            format!("error \"Ghostty AppleScript does not support virtual target for {command}\"")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::launcher::{Backend, LaunchStrategy};

    fn request(target: LaunchTarget) -> LaunchRequest {
        LaunchRequest {
            command: "ssh prod".into(),
            title: "Prod".into(),
            theme_or_profile: "basic".into(),
            target,
            backend: Backend::GhosttyOpen,
            strategy: LaunchStrategy::Default,
        }
    }

    #[test]
    fn missing_app_has_actionable_error() {
        let error = GhosttyError::MissingApplication.to_string();
        assert!(error.contains("install Ghostty"));
        assert_eq!(
            automation_denied_error(),
            GhosttyError::AppleScriptUnavailable
        );
        let _ = detect_application();
    }

    #[test]
    fn constructs_open_argument_vector() {
        assert_eq!(
            open_args(&request(LaunchTarget::New)).unwrap(),
            [
                "open",
                "-na",
                "Ghostty.app",
                "--args",
                "--title",
                "Prod",
                "-e",
                "ssh prod"
            ]
        );
    }

    #[test]
    fn rejects_open_tab_and_current() {
        assert_eq!(
            open_args(&request(LaunchTarget::Tab)),
            Err(GhosttyError::UnsupportedOpenTarget(LaunchTarget::Tab))
        );
        assert_eq!(
            open_args(&request(LaunchTarget::Current)),
            Err(GhosttyError::UnsupportedOpenTarget(LaunchTarget::Current))
        );
    }

    #[test]
    fn builds_applescript_for_targets() {
        assert!(applescript_source(&request(LaunchTarget::New)).contains("create window"));
        assert!(applescript_source(&request(LaunchTarget::Tab)).contains("create tab"));
        assert!(applescript_source(&request(LaunchTarget::Current)).contains("run command"));
        assert!(applescript_source(&request(LaunchTarget::Virtual))
            .contains("does not support virtual"));
    }
}
