#[cfg(test)]
use super::{LaunchRequest, LaunchTarget};

#[cfg(test)]
pub fn applescript_resource(request: &LaunchRequest) -> &'static str {
    match request.target {
        LaunchTarget::New => "terminal-new-window.scpt",
        LaunchTarget::Current => "terminal-current-window.scpt",
        LaunchTarget::Tab => "terminal-new-tab-default.scpt",
        LaunchTarget::Virtual => "virtual-with-screen.scpt",
    }
}

#[cfg(test)]
pub fn script_parameters(request: &LaunchRequest) -> Vec<String> {
    if request.target == LaunchTarget::Virtual {
        vec![request.command.clone(), request.title.clone()]
    } else {
        vec![
            request.command.clone(),
            request.theme_or_profile.clone(),
            request.title.clone(),
        ]
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
            backend: Backend::TerminalApp,
            strategy: LaunchStrategy::Default,
        }
    }

    #[test]
    fn maps_terminal_targets_to_legacy_scripts() {
        assert_eq!(
            applescript_resource(&request(LaunchTarget::New)),
            "terminal-new-window.scpt"
        );
        assert_eq!(
            applescript_resource(&request(LaunchTarget::Tab)),
            "terminal-new-tab-default.scpt"
        );
        assert_eq!(
            applescript_resource(&request(LaunchTarget::Current)),
            "terminal-current-window.scpt"
        );
    }

    #[test]
    fn passes_legacy_parameter_order() {
        assert_eq!(
            script_parameters(&request(LaunchTarget::New)),
            ["ssh prod", "basic", "Prod"]
        );
        assert_eq!(
            script_parameters(&request(LaunchTarget::Virtual)),
            ["ssh prod", "Prod"]
        );
    }
}
