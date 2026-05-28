# Rust Port Launch at Login

The Objective-C app uses the existing `LaunchAtLoginController` login item integration. The Rust port keeps the `launch_at_login` config key in the typed model so existing configs continue to load.

Implementation direction for the Rust app:

- Use Apple's ServiceManagement APIs for supported macOS versions.
- Keep the behavior opt-in through `launch_at_login: true`.
- Surface actionable errors when registration fails due to permissions, app translocation, or unsigned local builds.

Until the native AppKit shell is complete, the Rust module exposes a safe stub so config loading and validation can proceed without changing user configs.
