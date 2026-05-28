use super::{LaunchRequest, LaunchTarget};
use serde_json::json;
use std::env;
use std::path::PathBuf;
use thiserror::Error;

const APP_CMUX: &str = "/Applications/cmux.app/Contents/Resources/bin/cmux";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CmuxError {
    #[error("cmux binary was not found; configure cmux_binary or install cmux")]
    MissingBinary,
    #[error("cmux socket path is empty")]
    EmptySocketPath,
}

pub fn default_binary() -> Result<PathBuf, CmuxError> {
    if let Ok(path) = env::var("CMUX_BINARY") {
        return Ok(PathBuf::from(path));
    }
    let app_path = PathBuf::from(APP_CMUX);
    if app_path.exists() {
        return Ok(app_path);
    }
    env::var_os("PATH")
        .and_then(|paths| {
            env::split_paths(&paths)
                .map(|path| path.join("cmux"))
                .find(|path| path.exists())
        })
        .ok_or(CmuxError::MissingBinary)
}

pub fn cli_args(binary: PathBuf, request: &LaunchRequest) -> Vec<String> {
    let mut args = vec![binary.to_string_lossy().into_owned()];
    match request.target {
        LaunchTarget::New | LaunchTarget::Tab => {
            args.extend([
                "workspace".into(),
                "send".into(),
                request.title.clone(),
                request.command.clone(),
            ]);
        }
        LaunchTarget::Current => {
            args.extend(["send".into(), request.command.clone()]);
        }
        LaunchTarget::Virtual => {
            args.extend(["run".into(), "--background".into(), request.command.clone()]);
        }
    }
    args
}

pub fn socket_path() -> Result<PathBuf, CmuxError> {
    let path = env::var("CMUX_SOCKET_PATH").unwrap_or_else(|_| "/tmp/cmux.sock".into());
    if path.is_empty() {
        Err(CmuxError::EmptySocketPath)
    } else {
        Ok(PathBuf::from(path))
    }
}

pub fn socket_request(id: u64, method: &str, params: serde_json::Value) -> String {
    let mut encoded = json!({ "id": id, "method": method, "params": params }).to_string();
    encoded.push('\n');
    encoded
}

pub fn socket_launch_request(id: u64, request: &LaunchRequest) -> String {
    match request.target {
        LaunchTarget::New | LaunchTarget::Tab => socket_request(
            id,
            "workspace.send",
            json!({ "workspace": request.title, "text": request.command }),
        ),
        LaunchTarget::Current => {
            socket_request(id, "surface.send", json!({ "text": request.command }))
        }
        LaunchTarget::Virtual => socket_request(
            id,
            "command.run",
            json!({ "command": request.command, "background": true }),
        ),
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
            backend: Backend::CmuxCli,
            strategy: LaunchStrategy::Workspace,
        }
    }

    #[test]
    fn builds_workspace_cli_args() {
        assert_eq!(
            cli_args("/bin/cmux".into(), &request(LaunchTarget::New)),
            ["/bin/cmux", "workspace", "send", "Prod", "ssh prod"]
        );
    }

    #[test]
    fn builds_current_cli_args() {
        assert_eq!(
            cli_args("/bin/cmux".into(), &request(LaunchTarget::Current)),
            ["/bin/cmux", "send", "ssh prod"]
        );
    }

    #[test]
    fn serializes_newline_delimited_socket_request() {
        assert_eq!(
            socket_request(7, "surface.send", json!({"text":"hi"})),
            "{\"id\":7,\"method\":\"surface.send\",\"params\":{\"text\":\"hi\"}}\n"
        );
    }

    #[test]
    fn builds_socket_launch_request() {
        assert!(socket_launch_request(1, &request(LaunchTarget::New)).contains("workspace.send"));
    }
}
