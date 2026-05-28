#![allow(deprecated, unexpected_cfgs)]

use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicyAccessory};
use cocoa::base::{id, nil};
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

        let status_item = crate::macos::menu::install_status_menu(&menu_entries);
        retain_forever(status_item);
        app.run();
    }
}

unsafe fn retain_forever(object: id) {
    let _: id = msg_send![object, retain];
}
