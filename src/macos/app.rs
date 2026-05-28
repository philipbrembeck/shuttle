#![allow(deprecated, unexpected_cfgs)]

use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicyAccessory};
use cocoa::base::nil;
use cocoa::foundation::NSAutoreleasePool;
use objc::{msg_send, sel, sel_impl};

pub fn run() {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicyAccessory);

        let menu_entries = match crate::build_menu_entries() {
            Ok(entries) => entries,
            Err(error) => {
                eprintln!("Shuttle config error: {error}");
                crate::menu_model::error_menu("Error parsing config")
            }
        };

        let delegate = crate::macos::delegate::create_delegate();
        let _: () = msg_send![delegate, retain];

        let status_item = crate::macos::menu::install_status_menu(&menu_entries, delegate);
        // Keep the status item alive for the lifetime of the app
        let _: () = msg_send![status_item, retain];

        app.run();
    }
}
