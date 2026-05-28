#![allow(deprecated)]

use crate::config::{self, ConfigPaths, ReloadSnapshot};
use cocoa::base::id;
use std::sync::Mutex;

pub struct AppState {
    pub paths: ConfigPaths,
    pub snapshot: ReloadSnapshot,
    pub delegate: id,
    pub status_item: id,
}

// id is a raw pointer; all access is on the main thread, guarded by Mutex.
unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

pub static APP_STATE: Mutex<Option<AppState>> = Mutex::new(None);

pub fn init(paths: ConfigPaths, snapshot: ReloadSnapshot, delegate: id, status_item: id) {
    let mut guard = APP_STATE.lock().unwrap();
    *guard = Some(AppState {
        paths,
        snapshot,
        delegate,
        status_item,
    });
}

/// Called by the menu delegate's `menuWillOpen:` on the main thread.
/// Reloads config and rebuilds the NSMenu only when a watched file changed.
pub fn reload_if_needed() {
    let needs = {
        let guard = APP_STATE.lock().unwrap();
        guard.as_ref().is_some_and(|state| {
            let new_snap = config::snapshot(&state.paths);
            config::needs_reload(&state.snapshot, &new_snap)
        })
    };

    if !needs {
        return;
    }

    let (delegate, status_item, result) = {
        let guard = APP_STATE.lock().unwrap();
        let state = guard.as_ref().unwrap();
        let delegate = state.delegate;
        let status_item = state.status_item;
        let result = config::load_merged(&state.paths).map(|mut cfg| {
            config::apply_ssh_hosts(&mut cfg);
            let entries = crate::menu_model::with_separators(crate::menu_model::build(&cfg.hosts));
            let new_snap = config::snapshot(&state.paths);
            (entries, cfg, new_snap)
        });
        (delegate, status_item, result)
    };

    match result {
        Ok((entries, cfg, new_snap)) => {
            {
                let mut guard = APP_STATE.lock().unwrap();
                if let Some(state) = guard.as_mut() {
                    state.snapshot = new_snap;
                }
            }
            unsafe {
                crate::macos::menu::rebuild_menu(status_item, &entries, &cfg, delegate);
            }
        }
        Err(e) => eprintln!("Shuttle config reload error: {e}"),
    }
}
