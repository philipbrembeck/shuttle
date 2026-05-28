use super::{ITermVersion, LaunchRequest, LaunchTarget};

pub fn applescript_resource(request: &LaunchRequest, version: &ITermVersion) -> &'static str {
    match (version, &request.target) {
        (ITermVersion::Stable, LaunchTarget::New) => "iTerm2-stable-new-window.scpt",
        (ITermVersion::Stable, LaunchTarget::Current) => "iTerm2-stable-current-window.scpt",
        (ITermVersion::Stable, LaunchTarget::Tab) => "iTerm2-stable-new-tab-default.scpt",
        (ITermVersion::Nightly, LaunchTarget::New) => "iTerm2-nightly-new-window.scpt",
        (ITermVersion::Nightly, LaunchTarget::Current) => "iTerm2-nightly-current-window.scpt",
        (ITermVersion::Nightly, LaunchTarget::Tab) => "iTerm2-nightly-new-tab-default.scpt",
        (_, LaunchTarget::Virtual) => "virtual-with-screen.scpt",
    }
}

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
            theme_or_profile: "Default".into(),
            target,
            backend: Backend::ITerm {
                version: ITermVersion::Stable,
            },
            strategy: LaunchStrategy::Default,
        }
    }

    #[test]
    fn maps_stable_and_nightly_targets_to_legacy_scripts() {
        assert_eq!(
            applescript_resource(&request(LaunchTarget::New), &ITermVersion::Stable),
            "iTerm2-stable-new-window.scpt"
        );
        assert_eq!(
            applescript_resource(&request(LaunchTarget::Tab), &ITermVersion::Nightly),
            "iTerm2-nightly-new-tab-default.scpt"
        );
        assert_eq!(
            applescript_resource(&request(LaunchTarget::Current), &ITermVersion::Nightly),
            "iTerm2-nightly-current-window.scpt"
        );
    }

    #[test]
    fn passes_legacy_parameter_order() {
        assert_eq!(
            script_parameters(&request(LaunchTarget::Tab)),
            ["ssh prod", "Default", "Prod"]
        );
    }
}
