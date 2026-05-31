use super::{LaunchRequest, LaunchTarget};
use serde_json::json;
use std::env;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
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
    if path.trim().is_empty() {
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

pub fn send_socket_request(path: &Path, request: &str) -> std::io::Result<String> {
    let mut stream = std::os::unix::net::UnixStream::connect(path)?;
    stream.write_all(request.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
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
    fn builds_virtual_cli_args() {
        assert_eq!(
            cli_args("/bin/cmux".into(), &request(LaunchTarget::Virtual)),
            ["/bin/cmux", "run", "--background", "ssh prod"]
        );
    }

    #[test]
    fn uses_cmux_binary_environment_override() {
        std::env::set_var("CMUX_BINARY", "/tmp/custom-cmux");
        assert_eq!(default_binary().unwrap(), PathBuf::from("/tmp/custom-cmux"));
        std::env::remove_var("CMUX_BINARY");
    }

    #[test]
    fn uses_default_socket_path() {
        std::env::remove_var("CMUX_SOCKET_PATH");
        assert_eq!(socket_path().unwrap(), PathBuf::from("/tmp/cmux.sock"));
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
        assert!(socket_launch_request(1, &request(LaunchTarget::Current)).contains("surface.send"));
        assert!(socket_launch_request(1, &request(LaunchTarget::Virtual)).contains("command.run"));
    }

    #[test]
    fn rejects_whitespace_only_socket_path() {
        std::env::set_var("CMUX_SOCKET_PATH", "   ");
        assert_eq!(socket_path(), Err(CmuxError::EmptySocketPath));
        std::env::remove_var("CMUX_SOCKET_PATH");
    }

    #[test]
    fn sends_request_to_fake_unix_socket() {
        use std::io::{BufRead, BufReader};
        use std::os::unix::net::UnixListener;
        use std::thread;

        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("cmux.sock");
        let listener = UnixListener::bind(&socket_path).unwrap();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut line = String::new();
            BufReader::new(stream.try_clone().unwrap())
                .read_line(&mut line)
                .unwrap();
            assert!(line.contains("surface.send"));
            stream.write_all(b"{\"id\":1,\"result\":true}\n").unwrap();
        });

        let response = send_socket_request(
            &socket_path,
            &socket_request(1, "surface.send", json!({"text":"hi"})),
        )
        .unwrap();
        handle.join().unwrap();
        assert!(response.contains("result"));
    }
}
