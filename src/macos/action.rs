#![allow(deprecated, unexpected_cfgs)]

use cocoa::base::id;
use objc::declare::ClassDecl;
use objc::runtime::Sel;
use objc::runtime::{Class, Object};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::Once;

static REGISTER: Once = Once::new();

pub fn register_class() {
    REGISTER.call_once(|| {
        let superclass = class!(NSObject);
        let Some(mut decl) = ClassDecl::new("ShuttleAction", superclass) else {
            eprintln!("Shuttle: Objective-C class ShuttleAction is already registered");
            return;
        };
        decl.add_ivar::<usize>("shuttleCmd");
        decl.add_ivar::<usize>("shuttleBackend");
        unsafe {
            #[cfg(not(test))]
            decl.add_method(
                sel!(launch:),
                launch_handler as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&mut Object, Sel));
        }
        decl.register();
    });
}

/// Creates a retained ShuttleAction instance. The cmd and backend strings are
/// heap-allocated and intentionally leaked for the lifetime of the menu.
pub fn create_action(cmd: &str, backend: &str) -> id {
    register_class();
    let cmd_ptr = Box::into_raw(Box::new(cmd.to_string())) as usize;
    let backend_ptr = Box::into_raw(Box::new(backend.to_string())) as usize;
    unsafe {
        let Some(cls) = Class::get("ShuttleAction") else {
            eprintln!("Shuttle: Objective-C class ShuttleAction is unavailable");
            return cocoa::base::nil;
        };
        let obj: id = msg_send![cls, new];
        (*obj).set_ivar("shuttleCmd", cmd_ptr);
        (*obj).set_ivar("shuttleBackend", backend_ptr);
        // retain so ARC doesn't collect before the menu is gone
        let _: id = msg_send![obj, retain];
        obj
    }
}

#[cfg(not(test))]
extern "C" fn launch_handler(this: &Object, _sel: Sel, _sender: id) {
    let (cmd, backend) = action_strings(this);
    // Fire on a background thread so the menu closes immediately
    std::thread::spawn(move || {
        crate::macos::executor::execute(&cmd, &backend);
    });
}

extern "C" fn dealloc(this: &mut Object, _sel: Sel) {
    unsafe {
        let cmd_ptr: usize = *this.get_ivar("shuttleCmd");
        let backend_ptr: usize = *this.get_ivar("shuttleBackend");
        if cmd_ptr != 0 {
            drop(Box::from_raw(cmd_ptr as *mut String));
            this.set_ivar("shuttleCmd", 0_usize);
        }
        if backend_ptr != 0 {
            drop(Box::from_raw(backend_ptr as *mut String));
            this.set_ivar("shuttleBackend", 0_usize);
        }
        let superclass = class!(NSObject);
        let _: () = msg_send![super(this, superclass), dealloc];
    }
}

fn action_strings(this: &Object) -> (String, String) {
    unsafe {
        let cmd_ptr: usize = *this.get_ivar("shuttleCmd");
        let backend_ptr: usize = *this.get_ivar("shuttleBackend");
        let cmd = (*(cmd_ptr as *const String)).clone();
        let backend = (*(backend_ptr as *const String)).clone();
        (cmd, backend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_action_with_command_and_backend() {
        let action = create_action("ssh prod", "terminal-app");
        assert!(!action.is_null());
        let (cmd, backend) = action_strings(unsafe { &*action });
        assert_eq!(cmd, "ssh prod");
        assert_eq!(backend, "terminal-app");
    }
}
