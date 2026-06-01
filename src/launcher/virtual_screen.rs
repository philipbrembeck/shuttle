#[cfg(test)]
use super::LaunchRequest;

#[cfg(test)]
pub fn screen_args(request: &LaunchRequest) -> Vec<String> {
    vec![
        "screen".into(),
        "-d".into(),
        "-m".into(),
        "-S".into(),
        request.title.clone(),
        "sh".into(),
        "-lc".into(),
        request.command.clone(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::launcher::{Backend, LaunchStrategy, LaunchTarget};

    #[test]
    fn builds_screen_argument_vector() {
        let request = LaunchRequest {
            command: "echo hi".into(),
            title: "Task".into(),
            theme_or_profile: "basic".into(),
            target: LaunchTarget::Virtual,
            backend: Backend::Screen,
            strategy: LaunchStrategy::Default,
        };
        assert_eq!(
            screen_args(&request),
            ["screen", "-d", "-m", "-S", "Task", "sh", "-lc", "echo hi"]
        );
    }
}
