#![allow(deprecated, unexpected_cfgs)]

use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::Once;

static REGISTER: Once = Once::new();

pub fn register_delegate_class() -> &'static Class {
    REGISTER.call_once(|| {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("ShuttleDelegate", superclass).unwrap();

        unsafe {
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
        }

        decl.register();
    });

    Class::get("ShuttleDelegate").unwrap()
}

pub fn create_delegate() -> id {
    let cls = register_delegate_class();
    unsafe {
        let obj: id = msg_send![cls, new];
        obj
    }
}

extern "C" fn shuttle_configure(_this: &Object, _sel: Sel, _sender: id) {
    unsafe {
        let config_path = crate::config::discover_paths()
            .map(|paths| paths.main.to_string_lossy().to_string())
            .unwrap_or_else(|_| "~/.shuttle.json".to_string());
        let ns_path = NSString::alloc(nil).init_str(&config_path);
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let _: () = msg_send![workspace, openFile: ns_path];
    }
}

extern "C" fn shuttle_import(_this: &Object, _sel: Sel, _sender: id) {
    unsafe {
        let panel: id = msg_send![class!(NSOpenPanel), openPanel];
        let result: isize = msg_send![panel, runModal];
        if result == 1 {
            // NSOKButton == 1
            let url: id = msg_send![panel, URL];
            let path: id = msg_send![url, path];
            if let Ok(paths) = crate::config::discover_paths() {
                let dest = NSString::alloc(nil).init_str(&paths.main.to_string_lossy());
                let fm: id = msg_send![class!(NSFileManager), defaultManager];
                let _: () = msg_send![fm, removeItemAtPath: dest error: nil];
                let _: () = msg_send![fm, copyItemAtPath: path toPath: dest error: nil];
            }
        }
    }
}

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
