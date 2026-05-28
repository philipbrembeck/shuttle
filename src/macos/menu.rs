#![allow(deprecated, unexpected_cfgs, dead_code)]

use crate::menu_model::MenuEntry;
use cocoa::appkit::{NSMenu, NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use objc::{class, msg_send, sel, sel_impl};

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
        .map(|entry| match entry {
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

#[cfg(target_os = "macos")]
pub fn install_status_menu(entries: &[MenuEntry], delegate: id) -> id {
    unsafe {
        let menu = build_ns_menu(entries);

        // Separator before actions
        menu.addItem_(NSMenuItem::separatorItem(nil));

        // Configure...
        let configure = menu_item_with_action("Configure...", sel!(shuttleConfigure:));
        configure.setTarget_(delegate);
        menu.addItem_(configure);

        // Import...
        let import = menu_item_with_action("Import...", sel!(shuttleImport:));
        import.setTarget_(delegate);
        menu.addItem_(import);

        // Export...
        let export = menu_item_with_action("Export...", sel!(shuttleExport:));
        export.setTarget_(delegate);
        menu.addItem_(export);

        // About Shuttle
        let about = menu_item_with_action("About Shuttle", sel!(orderFrontStandardAboutPanel:));
        menu.addItem_(about);

        // Quit
        let quit = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            NSString::alloc(nil).init_str("Quit"),
            sel!(terminate:),
            NSString::alloc(nil).init_str("q"),
        );
        menu.addItem_(quit);

        // Status item
        let status_bar = NSStatusBar::systemStatusBar(nil);
        let item = status_bar.statusItemWithLength_(NSVariableStatusItemLength);
        let button: id = item.button();
        if button != nil {
            // Use SF Symbol "paperplane.fill" (rocket-like, available since macOS 11)
            let symbol_name = NSString::alloc(nil).init_str("paperplane.fill");
            let image: id = msg_send![class!(NSImage), imageWithSystemSymbolName: symbol_name accessibilityDescription: nil];
            if image != nil {
                let _: () = msg_send![image, setTemplate: YES];
                let _: () = msg_send![button, setImage: image];
            } else {
                // Fallback for older macOS
                let title = NSString::alloc(nil).init_str("🚀");
                let _: () = msg_send![button, setTitle: title];
            }
        }
        item.setMenu_(menu);
        item
    }
}

#[cfg(target_os = "macos")]
unsafe fn menu_item_with_action(title: &str, action: objc::runtime::Sel) -> id {
    NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
        NSString::alloc(nil).init_str(title),
        action,
        NSString::alloc(nil).init_str(""),
    )
}

#[cfg(target_os = "macos")]
unsafe fn build_ns_menu(entries: &[MenuEntry]) -> id {
    let menu = NSMenu::new(nil).autorelease();
    let _: () = msg_send![menu, setAutoenablesItems: NO];
    for entry in entries {
        match entry {
            MenuEntry::Menu {
                title, children, ..
            } => {
                let item = menu_item(title, true);
                let submenu = build_ns_menu(children);
                item.setSubmenu_(submenu);
                menu.addItem_(item);
            }
            MenuEntry::Command { title, .. } => {
                let item = menu_item(title, true);
                menu.addItem_(item);
            }
            MenuEntry::Disabled { title } => {
                let item = menu_item(title, false);
                menu.addItem_(item);
            }
            MenuEntry::Separator => menu.addItem_(NSMenuItem::separatorItem(nil)),
        }
    }
    menu
}

#[cfg(target_os = "macos")]
unsafe fn menu_item(title: &str, enabled: bool) -> id {
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
}
