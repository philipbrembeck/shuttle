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
        LaunchTarget::New => format!(
            r#"tell application "Ghostty"
    activate
    set cfg to new surface configuration
    set command of cfg to "{command}"
    set win to new window with configuration cfg
end tell"#
        ),
        LaunchTarget::Tab => format!(
            r#"tell application "Ghostty"
    activate
    set cfg to new surface configuration
    set command of cfg to "{command}"
    if (count of windows) = 0 then
        set win to new window with configuration cfg
    else
        set win to front window
        set newTab to new tab in win with configuration cfg
        select tab newTab
    end if
end tell"#
        ),
        LaunchTarget::Current => format!(
            r#"tell application "Ghostty"
    activate
    if (count of windows) = 0 then
        set cfg to new surface configuration
        set command of cfg to "{command}"
        set win to new window with configuration cfg
    else
        set term to focused terminal of selected tab of front window
        input text "{command}" to term
        send key "enter" to term
    end if
end tell"#
        ),
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
        assert!(applescript_source(&request(LaunchTarget::New))
            .contains("new window with configuration cfg"));
        assert!(applescript_source(&request(LaunchTarget::Tab))
            .contains("new tab in win with configuration cfg"));
        assert!(applescript_source(&request(LaunchTarget::Current)).contains("input text"));
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
