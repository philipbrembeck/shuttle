#[cfg(test)]
use super::{LaunchRequest, LaunchTarget};
#[cfg(test)]
#[cfg(test)]
use thiserror::Error;

#[cfg(test)]
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

#[cfg(test)]
pub fn detect_application() -> Result<(), GhosttyError> {
    if std::path::Path::new("/Applications/Ghostty.app").exists() {
        Ok(())
    } else {
        Err(GhosttyError::MissingApplication)
    }
}

#[cfg(test)]
pub fn automation_denied_error() -> GhosttyError {
    GhosttyError::AppleScriptUnavailable
}

#[cfg(test)]
pub fn open_args(request: &LaunchRequest) -> Result<Vec<String>, GhosttyError> {
    if request.target != LaunchTarget::New {
        return Err(GhosttyError::UnsupportedOpenTarget(request.target.clone()));
    }

    let mut args = vec![
        "open".into(),
        "-a".into(),
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

#[cfg(test)]
pub fn applescript_source(request: &LaunchRequest) -> String {
    let command = crate::macos::util::escape_for_applescript(&request.command);
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
                "-a",
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

    #[test]
    fn escapes_quotes_and_backslashes_in_applescript() {
        let mut req = request(LaunchTarget::New);
        req.command = r#"echo "hi" && cd C:\tmp"#.into();
        let script = applescript_source(&req);
        assert!(script.contains(r#"echo \"hi\" && cd C:\\tmp"#));
    }
}
