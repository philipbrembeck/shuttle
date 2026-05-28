pub fn run() {
    // Native AppKit status-item integration will be filled in as the menu model and
    // callback bridge land. For now, bootstrap config loading so the Rust target is
    // runnable during Phase 1 on macOS and surfaces parse/path errors clearly.
    if let Err(error) = crate::bootstrap_config() {
        eprintln!("Shuttle config error: {error}");
    }
}
