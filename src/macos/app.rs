#![allow(deprecated, unexpected_cfgs)]

use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicyAccessory};
use cocoa::base::{id, nil, YES};
use cocoa::foundation::NSAutoreleasePool;
use objc::{class, msg_send, sel, sel_impl};

pub fn run() {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicyAccessory);

        let delegate = crate::macos::delegate::create_delegate();
        let _: () = msg_send![delegate, retain];

        let (entries, config, paths, snapshot) = match crate::build_menu_entries() {
            Ok((entries, config)) => {
                let paths = crate::config::discover_paths().unwrap_or_else(|_| {
                    crate::config::ConfigPaths {
                        main: dirs::home_dir()
                            .unwrap_or_default()
                            .join(".config/shuttle/config.json"),
                        alt: None,
                        used_main_override: false,
                    }
                });
                let snapshot = crate::config::snapshot(&paths);
                (entries, config, paths, snapshot)
            }
            Err(error) => {
                eprintln!("Shuttle config error: {error}");
                let entries = crate::menu_model::error_menu("Error parsing config");
                let config = crate::config::model::Config::default();
                let paths = crate::config::ConfigPaths {
                    main: dirs::home_dir()
                        .unwrap_or_default()
                        .join(".config/shuttle/config.json"),
                    alt: None,
                    used_main_override: false,
                };
                let snapshot = crate::config::ReloadSnapshot::default();
                (entries, config, paths, snapshot)
            }
        };

        // Apply launch-at-login preference
        if config.launch_at_login {
            if let Err(e) = crate::macos::login_item::set_launch_at_login(true) {
                eprintln!("Shuttle: launch-at-login error: {e}");
            }
        }

        let status_item = crate::macos::menu::install_status_menu(&entries, &config, delegate);
        let _: () = msg_send![status_item, retain];

        // Store global state so the menu delegate can hot-reload
        crate::macos::state::init(paths, snapshot, delegate, status_item);

        // NSTimer fires every second to check for config file changes.
        // This is more reliable than relying solely on NSMenuDelegate.menuWillOpen:
        let timer: id = msg_send![
            class!(NSTimer),
            scheduledTimerWithTimeInterval: 1.0_f64
            target: delegate
            selector: sel!(checkReload:)
            userInfo: nil
            repeats: YES
        ];
        let _: () = msg_send![timer, retain];

        app.run();
    }
}
