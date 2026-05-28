# Rust Port Transition Decision

During the Rust port, the Objective-C application remains in-tree in its current `Shuttle/` and `Shuttle.xcodeproj/` locations.

Rationale:

- It is the compatibility oracle for config loading, menu ordering, SSH import, and legacy AppleScript behavior.
- It remains buildable while the Rust AppKit shell reaches parity.
- Keeping paths stable avoids Xcode project churn and accidental binary/resource movement.

Revisit moving the Objective-C app under a legacy directory only after the Rust app has passed manual parity checks for menu rendering, Terminal.app, iTerm, URL, virtual, Ghostty, cmux, import/export/configure/about/quit, and launch-at-login behavior.
