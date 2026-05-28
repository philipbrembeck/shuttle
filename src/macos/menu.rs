#![allow(deprecated, unexpected_cfgs, dead_code)]

use crate::config::model::Config;
use crate::launcher::{Backend, ITermVersion, LaunchKind};
use crate::menu_model::MenuEntry;
use cocoa::appkit::{NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use objc::{class, msg_send, sel, sel_impl};

// ── Public spec type (used in tests without AppKit) ──────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeMenuSpec {
    Menu {
        title: String,
        children: Vec<NativeMenuSpec>,
    },
    Command {
        title: String,
        enabled: bool,
    },
    Separator,
}

pub fn build_spec(entries: &[MenuEntry]) -> Vec<NativeMenuSpec> {
    entries
        .iter()
        .map(|e| match e {
            MenuEntry::Menu {
                title, children, ..
            } => NativeMenuSpec::Menu {
                title: title.clone(),
                children: build_spec(children),
            },
            MenuEntry::Command { title, .. } => NativeMenuSpec::Command {
                title: title.clone(),
                enabled: true,
            },
            MenuEntry::Disabled { title } => NativeMenuSpec::Command {
                title: title.clone(),
                enabled: false,
            },
            MenuEntry::Separator => NativeMenuSpec::Separator,
        })
        .collect()
}

// ── Native menu construction ──────────────────────────────────────────────────

#[cfg(target_os = "macos")]
pub fn install_status_menu(entries: &[MenuEntry], config: &Config, delegate: id) -> id {
    unsafe {
        let menu = build_ns_menu(entries, config);

        menu.addItem_(NSMenuItem::separatorItem(nil));

        for (label, action) in [
            ("Configure...", sel!(shuttleConfigure:)),
            ("Import...", sel!(shuttleImport:)),
            ("Export...", sel!(shuttleExport:)),
        ] {
            let item = titled_action_item(label, action);
            item.setTarget_(delegate);
            menu.addItem_(item);
        }

        let about = titled_action_item("About Shuttle", sel!(orderFrontStandardAboutPanel:));
        menu.addItem_(about);

        let quit = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            NSString::alloc(nil).init_str("Quit"),
            sel!(terminate:),
            NSString::alloc(nil).init_str("q"),
        );
        menu.addItem_(quit);

        // Build status item
        let status_bar = NSStatusBar::systemStatusBar(nil);
        let item = status_bar.statusItemWithLength_(NSVariableStatusItemLength);
        let button: id = item.button();
        if button != nil {
            let sym = NSString::alloc(nil).init_str("paperplane.fill");
            let image: id = msg_send![
                class!(NSImage),
                imageWithSystemSymbolName: sym
                accessibilityDescription: nil
            ];
            if image != nil {
                let _: () = msg_send![image, setTemplate: YES];
                let _: () = msg_send![button, setImage: image];
            } else {
                let title = NSString::alloc(nil).init_str("🚀");
                let _: () = msg_send![button, setTitle: title];
            }
        }
        item.setMenu_(menu);
        item
    }
}

// ── Recursive menu builder ────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
unsafe fn build_ns_menu(entries: &[MenuEntry], config: &Config) -> id {
    let menu = NSMenu::new(nil).autorelease();
    let _: () = msg_send![menu, setAutoenablesItems: NO];

    for entry in entries {
        match entry {
            MenuEntry::Menu {
                title, children, ..
            } => {
                let item = static_item(title, true);
                let submenu = build_ns_menu(children, config);
                item.setSubmenu_(submenu);
                menu.addItem_(item);
            }
            MenuEntry::Command { title, command, .. } => {
                // Resolve what backend this command would use
                let backend_str = resolve_backend_str(config, &command.cmd, command);
                let item = command_item(title, &command.cmd, &backend_str);
                menu.addItem_(item);
            }
            MenuEntry::Disabled { title } => {
                menu.addItem_(static_item(title, false));
            }
            MenuEntry::Separator => {
                menu.addItem_(NSMenuItem::separatorItem(nil));
            }
        }
    }
    menu
}

/// Build an NSMenuItem that fires ShuttleAction.launch: on click.
#[cfg(target_os = "macos")]
unsafe fn command_item(title: &str, cmd: &str, backend: &str) -> id {
    let action = crate::macos::action::create_action(cmd, backend);
    let item = NSMenuItem::alloc(nil)
        .initWithTitle_action_keyEquivalent_(
            NSString::alloc(nil).init_str(title),
            sel!(launch:),
            NSString::alloc(nil).init_str(""),
        )
        .autorelease();
    item.setTarget_(action);
    let _: () = msg_send![item, setEnabled: YES];
    item
}

/// Build a non-clickable (or action-less) NSMenuItem.
#[cfg(target_os = "macos")]
unsafe fn static_item(title: &str, enabled: bool) -> id {
    let item = NSMenuItem::alloc(nil)
        .initWithTitle_action_keyEquivalent_(
            NSString::alloc(nil).init_str(title),
            sel!(noop:),
            NSString::alloc(nil).init_str(""),
        )
        .autorelease();
    let flag = if enabled { YES } else { NO };
    let _: () = msg_send![item, setEnabled: flag];
    item
}

#[cfg(target_os = "macos")]
unsafe fn titled_action_item(title: &str, action: objc::runtime::Sel) -> id {
    NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
        NSString::alloc(nil).init_str(title),
        action,
        NSString::alloc(nil).init_str(""),
    )
}

// ── Backend resolution ────────────────────────────────────────────────────────

/// Returns the executor backend string for a given command and config.
fn resolve_backend_str(
    config: &Config,
    cmd: &str,
    host: &crate::config::model::CommandHost,
) -> String {
    // URL commands are dispatched directly
    if is_url(cmd) {
        return format!("url:{cmd}");
    }
    match crate::launcher::normalize(config, host, &host.name) {
        Ok(LaunchKind::Url(url)) => format!("url:{url}"),
        Ok(LaunchKind::Terminal(req)) => backend_to_str(&req.backend),
        Err(_) => "terminal-app".to_string(),
    }
}

fn backend_to_str(backend: &Backend) -> String {
    match backend {
        Backend::TerminalApp => "terminal-app".to_string(),
        Backend::ITerm {
            version: ITermVersion::Stable,
        } => "iterm-stable".to_string(),
        Backend::ITerm {
            version: ITermVersion::Nightly,
        } => "iterm-nightly".to_string(),
        Backend::GhosttyOpen => "ghostty-open".to_string(),
        Backend::GhosttyAppleScript => "ghostty-applescript".to_string(),
        Backend::CmuxCli => "cmux-cli".to_string(),
        Backend::CmuxSocket => "cmux-socket".to_string(),
        Backend::Screen => "screen".to_string(),
    }
}

fn is_url(cmd: &str) -> bool {
    let cmd = cmd.trim();
    cmd.starts_with("http://")
        || cmd.starts_with("https://")
        || cmd.starts_with("ssh://")
        || cmd.starts_with("file://")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::CommandHost;
    use std::collections::BTreeMap;

    #[test]
    fn converts_menu_model_to_native_spec() {
        let entries = vec![
            MenuEntry::Disabled {
                title: "Error parsing config".into(),
            },
            MenuEntry::Separator,
            MenuEntry::Command {
                title: "Prod".into(),
                command: CommandHost {
                    cmd: "ssh prod".into(),
                    name: "Prod".into(),
                    in_terminal: None,
                    theme: None,
                    title: None,
                    backend: None,
                    strategy: None,
                    extra: BTreeMap::new(),
                },
                separator_after: false,
            },
        ];
        assert_eq!(
            build_spec(&entries),
            vec![
                NativeMenuSpec::Command {
                    title: "Error parsing config".into(),
                    enabled: false
                },
                NativeMenuSpec::Separator,
                NativeMenuSpec::Command {
                    title: "Prod".into(),
                    enabled: true
                },
            ]
        );
    }

    #[test]
    fn url_cmds_resolve_to_url_backend() {
        let config = Config::default();
        let host = CommandHost {
            cmd: "https://example.com".into(),
            name: "Website".into(),
            in_terminal: None,
            theme: None,
            title: None,
            backend: None,
            strategy: None,
            extra: BTreeMap::new(),
        };
        assert!(resolve_backend_str(&config, &host.cmd, &host).starts_with("url:"));
    }

    #[test]
    fn ghostty_terminal_key_resolves_to_ghostty_open() {
        let config = Config {
            terminal: Some("ghostty".into()),
            ..Config::default()
        };
        let host = CommandHost {
            cmd: "ssh prod".into(),
            name: "Prod".into(),
            in_terminal: None,
            theme: None,
            title: None,
            backend: None,
            strategy: None,
            extra: BTreeMap::new(),
        };
        assert_eq!(
            resolve_backend_str(&config, &host.cmd, &host),
            "ghostty-open"
        );
    }
}
