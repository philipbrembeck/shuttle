#![allow(deprecated, unexpected_cfgs)]

/// Register or deregister Shuttle as a login item via macOS System Events.
/// Uses osascript so it works on all macOS versions without extra framework linking.
#[cfg(all(target_os = "macos", not(test)))]
pub fn set_launch_at_login(enabled: bool) -> Result<(), String> {
    let path = bundle_path().ok_or_else(|| "Could not determine app bundle path".to_string())?;

    // Skip if we're running from target/ (dev build, not installed)
    if path.contains("/target/") {
        return Err(
            "launch_at_login has no effect for dev builds; install to /Applications first".into(),
        );
    }

    let script = launch_at_login_script(enabled, &path);

    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err("System Events refused to update login items — check Automation permission in System Settings → Privacy & Security".into())
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set_launch_at_login(_enabled: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_at_login_script(enabled: bool, path: &str) -> String {
    if enabled {
        format!(
            r#"tell application "System Events"
    set loginItems to get login items
    repeat with i in loginItems
        if path of i is "{path}" then return
    end repeat
    make new login item at end of login items with properties {{path:"{path}", hidden:false}}
end tell"#
        )
    } else {
        format!(
            r#"tell application "System Events"
    repeat with i in get login items
        if path of i is "{path}" then
            delete i
            return
        end if
    end repeat
end tell"#
        )
    }
}

#[cfg(all(target_os = "macos", not(test)))]
fn bundle_path() -> Option<String> {
    use objc::{class, msg_send, sel, sel_impl};
    unsafe {
        use cocoa::base::nil;
        let bundle: cocoa::base::id = msg_send![class!(NSBundle), mainBundle];
        if bundle == nil {
            return None;
        }
        let path: cocoa::base::id = msg_send![bundle, bundlePath];
        if path == nil {
            return None;
        }
        let c_str: *const std::os::raw::c_char = msg_send![path, UTF8String];
        if c_str.is_null() {
            return None;
        }
        Some(
            std::ffi::CStr::from_ptr(c_str)
                .to_string_lossy()
                .into_owned(),
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn non_macos_stub_is_safe() {
        #[cfg(not(target_os = "macos"))]
        super::set_launch_at_login(false).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn builds_launch_at_login_scripts() {
        let enable = super::launch_at_login_script(true, "/Applications/Shuttle.app");
        assert!(enable.contains("make new login item"));
        assert!(enable.contains("/Applications/Shuttle.app"));

        let disable = super::launch_at_login_script(false, "/Applications/Shuttle.app");
        assert!(disable.contains("delete i"));
        assert!(disable.contains("get login items"));
    }
}
