#![allow(deprecated, unexpected_cfgs)]

#[cfg(not(test))]
use cocoa::base::{id, nil};
#[cfg(not(test))]
use cocoa::foundation::NSString;
#[cfg(not(test))]
use objc::declare::ClassDecl;
#[cfg(not(test))]
use objc::runtime::{Class, Object, Protocol, Sel};
#[cfg(not(test))]
use objc::{class, msg_send, sel, sel_impl};
#[cfg(not(test))]
use std::sync::Once;

#[cfg(not(test))]
static REGISTER: Once = Once::new();

#[cfg(not(test))]
pub fn register_delegate_class() -> Option<&'static Class> {
    REGISTER.call_once(|| {
        let superclass = class!(NSObject);
        let Some(mut decl) = ClassDecl::new("ShuttleDelegate", superclass) else {
            eprintln!("Shuttle: Objective-C class ShuttleDelegate is already registered");
            return;
        };

        // Conform to NSMenuDelegate so menuWillOpen: fires.
        if let Some(proto) = Protocol::get("NSMenuDelegate") {
            decl.add_protocol(proto);
        }

        unsafe {
            decl.add_method(
                sel!(menuWillOpen:),
                menu_will_open as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(checkReload:),
                check_reload as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(shuttleConfigure:),
                shuttle_configure as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(shuttleImport:),
                shuttle_import as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(shuttleExport:),
                shuttle_export as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(shuttleCheckForUpdates:),
                shuttle_check_for_updates as extern "C" fn(&Object, Sel, id),
            );
        }

        decl.register();
    });

    Class::get("ShuttleDelegate")
}

#[cfg(not(test))]
pub fn create_delegate() -> id {
    let Some(cls) = register_delegate_class() else {
        eprintln!("Shuttle: Objective-C class ShuttleDelegate is unavailable");
        return nil;
    };
    unsafe { msg_send![cls, new] }
}

// ── NSMenuDelegate ────────────────────────────────────────────────────────────

#[cfg(not(test))]
extern "C" fn menu_will_open(_this: &Object, _sel: Sel, _menu: id) {
    crate::macos::state::reload_if_needed();
}

/// Fired by the NSTimer in app.rs every second — reliable hot reload fallback.
#[cfg(not(test))]
extern "C" fn check_reload(_this: &Object, _sel: Sel, _timer: id) {
    crate::macos::state::reload_if_needed();
}

// ── Menu actions ──────────────────────────────────────────────────────────────

#[cfg(not(test))]
extern "C" fn shuttle_configure(_this: &Object, _sel: Sel, _sender: id) {
    let paths = crate::config::discover_paths();
    let config_path = paths
        .as_ref()
        .map(|p| p.main.to_string_lossy().to_string())
        .unwrap_or_else(|_| "~/.config/shuttle/config.json".to_string());
    let editor = paths
        .ok()
        .and_then(|paths| crate::config::load_config(&paths.main).ok())
        .and_then(|config| config.editor)
        .unwrap_or_else(|| "default".to_string());
    open_config_with_editor(&config_path, &editor);
}

#[cfg(not(test))]
fn open_config_with_editor(config_path: &str, editor: &str) {
    match editor_action(config_path, editor) {
        EditorAction::Default => unsafe {
            let ns_path = NSString::alloc(nil).init_str(config_path);
            let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
            let _: () = msg_send![workspace, openFile: ns_path];
        },
        EditorAction::TerminalScript(script) => {
            let _ = std::process::Command::new("/usr/bin/osascript")
                .arg("-e")
                .arg(script)
                .spawn();
        }
        EditorAction::OpenWithApp { app, path } => {
            let _ = std::process::Command::new("/usr/bin/open")
                .args(["-a", &app, &path])
                .spawn();
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum EditorAction {
    Default,
    TerminalScript(String),
    OpenWithApp { app: String, path: String },
}

fn editor_action(config_path: &str, editor: &str) -> EditorAction {
    let editor = editor.trim();
    if editor.is_empty() || editor.eq_ignore_ascii_case("default") {
        return EditorAction::Default;
    }

    if matches!(editor, "nano" | "vi" | "vim" | "nvim" | "emacs") {
        let command = format!("{} {}", shell_quote(editor), shell_quote(config_path));
        let script = format!(
            "tell application \"Terminal\"\n    activate\n    do script {}\nend tell",
            applescript_string(&command)
        );
        return EditorAction::TerminalScript(script);
    }

    EditorAction::OpenWithApp {
        app: editor.to_string(),
        path: config_path.to_string(),
    }
}

fn applescript_string(value: &str) -> String {
    format!("{:?}", value)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(not(test))]
extern "C" fn shuttle_import(_this: &Object, _sel: Sel, _sender: id) {
    unsafe {
        let panel: id = msg_send![class!(NSOpenPanel), openPanel];
        let result: isize = msg_send![panel, runModal];
        if result == 1 {
            let url: id = msg_send![panel, URL];
            let src_path: id = msg_send![url, path];
            if let Ok(paths) = crate::config::discover_paths() {
                // Ensure the config directory exists
                if let Some(parent) = paths.main.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let dest = NSString::alloc(nil).init_str(&paths.main.to_string_lossy());
                let fm: id = msg_send![class!(NSFileManager), defaultManager];
                let _: () = msg_send![fm, removeItemAtPath: dest error: nil];
                let _: () = msg_send![fm, copyItemAtPath: src_path toPath: dest error: nil];
            }
        }
    }
}

#[cfg(not(test))]
extern "C" fn shuttle_export(_this: &Object, _sel: Sel, _sender: id) {
    unsafe {
        let panel: id = msg_send![class!(NSSavePanel), savePanel];
        let result: isize = msg_send![panel, runModal];
        if result == 1 {
            let url: id = msg_send![panel, URL];
            let dest: id = msg_send![url, path];
            if let Ok(paths) = crate::config::discover_paths() {
                let src = NSString::alloc(nil).init_str(&paths.main.to_string_lossy());
                let fm: id = msg_send![class!(NSFileManager), defaultManager];
                let _: () = msg_send![fm, copyItemAtPath: src toPath: dest error: nil];
            }
        }
    }
}

#[cfg(not(test))]
extern "C" fn shuttle_check_for_updates(_this: &Object, _sel: Sel, _sender: id) {
    crate::update::check_for_updates_async();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_editor_actions() {
        assert_eq!(
            editor_action("/tmp/config.json", "default"),
            EditorAction::Default
        );
        assert_eq!(
            editor_action("/tmp/config.json", "  "),
            EditorAction::Default
        );

        let terminal = editor_action("/tmp/it's.json", "vim");
        match terminal {
            EditorAction::TerminalScript(script) => {
                assert!(script.contains("Terminal"));
                assert!(script.contains("it"));
                assert!(script.contains("s.json"));
            }
            other => panic!("unexpected editor action: {other:?}"),
        }

        assert_eq!(
            editor_action("/tmp/config.json", "TextEdit"),
            EditorAction::OpenWithApp {
                app: "TextEdit".into(),
                path: "/tmp/config.json".into(),
            }
        );
    }

    #[test]
    fn quotes_strings_for_shell_and_applescript() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
        assert_eq!(applescript_string("say \"hi\""), "\"say \\\"hi\\\"\"");
    }
}
