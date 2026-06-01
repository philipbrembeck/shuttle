# ADR 0004: Coverage boundaries for macOS glue

## Status

Accepted

## Context

The project tracks unit-test coverage with `cargo llvm-cov`. Most Shuttle logic is plain Rust and can be tested directly: config discovery/parsing, SSH import, menu modeling, backend resolution, command construction, update response parsing, and reload decisions.

Some macOS integration code is not safely executable in unit tests because it crosses into AppKit, Objective-C runtime callbacks, modal system panels, the NSApplication event loop, or process-spawning side effects. These paths require a real interactive macOS application session, system permissions, and user-owned UI state. Executing them in automated unit tests would be flaky and potentially disruptive.

## Decision

Keep untestable Cocoa glue compiled for normal builds, but compile it out of unit-test builds with `#[cfg(not(test))]`. Keep testable decision logic in small helper functions that remain compiled and covered in tests.

This applies to:

- `src/main.rs`: the process entry point delegates to the macOS app loop or local diagnostic output. Unit tests exercise `build_menu_entries` and lower-level modules instead.
- `src/macos/app.rs`: `run` creates `NSApplication`, status items, timers, and enters `app.run()`, which is an interactive event loop and must not run in unit tests.
- `src/macos/delegate.rs`: Objective-C delegate registration and callbacks for menu opening, import/export panels, config opening, and update actions require AppKit runtime objects or spawn external applications. Pure editor-action and quoting helpers remain tested.
- `src/macos/menu.rs`: native `NSMenu` and `NSStatusItem` construction requires AppKit objects. The platform-neutral `NativeMenuSpec` and launch payload resolution remain tested.
- `src/macos/state.rs`: hot reload decision logic is tested; the final AppKit `rebuild_menu` call is excluded in test builds because it requires a native status item.
- `src/macos/login_item.rs`: System Events automation is not executed in tests. Login-item AppleScript generation and the non-macOS safe stub are tested.

## What remains actually tested

- Config loading, path precedence, JSON/YAML parsing, default creation, reload snapshots, and SSH host import.
- Menu model construction, sorting, separators, disabled error entries, and native-menu specs.
- Launcher normalization and backend-specific argument/script builders.
- cmux socket request serialization and fake Unix socket I/O.
- URL command construction.
- Update release JSON parsing, version comparison, and script generation.
- macOS action object payload storage.
- Hot reload change detection.

## Consequences

Coverage reports represent code that can be exercised deterministically in automated unit tests. Interactive Cocoa event-loop and modal UI code must be verified by manual app testing or future UI/integration tests running in a controlled macOS GUI session.

This is not a license to hide ordinary business logic from coverage. New behavior should be factored into testable helpers before being called from Cocoa glue.
