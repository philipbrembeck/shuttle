# Rust Port Transition Decision

The repository has switched to the Rust implementation as the only in-tree application.

Removed legacy implementation paths:

- `Shuttle/`
- `Shuttle.xcodeproj/`
- `apple-scripts/`
- Objective-C source, XIBs, and Xcode project metadata

Kept compatibility resources:

- `resources/apple-scpt/*.scpt` compiled AppleScript files used by Rust launch backends
- `resources/shuttle.default.json`
- `shuttle.icns`

Rationale:

- The Rust port is now the source of truth for config loading, menu modeling, and backend dispatch.
- Keeping only the Rust app avoids split-brain maintenance during testing.
- Compiled AppleScript resources remain because Terminal.app/iTerm/screen compatibility still depends on them.
