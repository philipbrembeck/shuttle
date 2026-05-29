---
date: 2026-05-29T03:29:12.107056+00:00
git_commit: b525d578f19a36d615efc2f4463a4c4e1ba2425b
branch: feat/RustPort
topic: "Automated Releases and App Updates"
tags: [plan, github-actions, semver, packaging, sparkle, macos]
status: draft
---

# Automated Releases and App Updates Implementation Plan

## Overview

Add a production release path for Shuttle's Rust macOS app: GitHub Actions builds on pushes to `main`, automatically bumps SemVer and creates tags/releases, keeps app-visible version metadata in sync, and enables native macOS auto-updates via Sparkle.

## Current State Analysis

- There is no `.github/workflows/` release or CI workflow in the repository.
- `Cargo.toml` declares package version `0.1.0`, while `resources/Shuttle-Info.plist` declares `CFBundleShortVersionString` and `CFBundleVersion` as `0.2.0`; version metadata is duplicated and inconsistent.
- `scripts/build-rust-app.sh` builds `target/release/shuttle-rs`, creates `target/release/Shuttle.app`, and copies `resources/Shuttle-Info.plist` verbatim to `Contents/Info.plist`.
- `docs/packaging-rust-port.md` says release codesigning/notarization still needs to be added.
- Menu construction lives in `src/macos/menu.rs`; the persistent footer currently contains Configuration and Quit items but no update item.
- Runtime bootstrap in `src/macos/app.rs` creates the AppKit app, installs the status menu, and starts a timer for config reloads; there is no network/updater component.
- `Cargo.toml` has no updater/network dependency and only has `cocoa`/`objc` for macOS AppKit integration.
- GitHub-hosted macOS ARM runners are available via labels such as `macos-14-arm64`/newer Apple Silicon macOS runner labels, so native ARM release builds can run in GitHub Actions.

## Desired End State

- A push to `main` runs the quality gate on a macOS ARM runner and, when releasable commits exist, creates a SemVer release.
- `Cargo.toml`, `Cargo.lock`, and `resources/Shuttle-Info.plist` all reflect the same app version before the release artifact is built.
- The release workflow creates a `vX.Y.Z` Git tag, GitHub Release, zipped or DMG-packaged `Shuttle.app` artifact, and Sparkle appcast metadata.
- Shuttle includes a native menu item for user-initiated update checks and supports background update checks through Sparkle.
- Release docs explain required GitHub secrets, signing/notarization behavior, Sparkle key management, and recovery steps.

## What We're NOT Doing

- We are not implementing a DIY updater that downloads GitHub release assets and replaces the running app manually.
- We are not adding Electron/Tauri/webview infrastructure.
- We are not publishing to the Mac App Store.
- We are not requiring paid Apple Developer signing for local development builds; release signing/notarization should be conditional on secrets.
- We are not changing Shuttle's user config format or terminal-launch behavior.
- We are not implementing multi-channel beta/stable updates in the first pass.

## UI Mockups

Current menu footer:

```text
… user hosts …
────────────────────────
Configuration  ›
Quit
```

Initial proposed updater footer using Sparkle's standard flow:

```text
… user hosts …
────────────────────────
Check for Updates…    ⇩
Configuration  ›
Quit
```

Optional later enhancement if custom appcast pre-checking is added:

```text
… user hosts …
────────────────────────
Update to v1.4.0      ⇩
Configuration  ›
Quit
```

The first implementation should use `Check for Updates…` and let Sparkle show native UI containing the latest version, release notes, progress, install/relaunch prompts, and errors.

## Architecture and Code Reuse

Sparkle should own update discovery, signature validation, download, replacement, and native UI. Shuttle should only bundle/configure Sparkle and expose a menu action.

```text
GitHub Actions
  ├─ semver release tool
  ├─ version sync script
  ├─ scripts/check-rust.sh
  ├─ scripts/build-rust-app.sh
  ├─ codesign/notarytool       (conditional)
  ├─ Sparkle sign_update       (required for appcast)
  └─ GitHub Release + appcast

Shuttle.app
  ├─ Contents/Info.plist       (version + Sparkle keys)
  ├─ Contents/Frameworks/Sparkle.framework
  ├─ Contents/MacOS/shuttle-rs
  └─ menu item -> SPUStandardUpdaterController.checkForUpdates:
```

Affected files and expected changes:

- `.github/workflows/`
  - `ci.yml` - run PR/push quality gate on macOS ARM.
  - `release.yml` - push-to-main SemVer release, tag, build, package, sign/notarize conditionally, publish artifacts/appcast.
- `scripts/`
  - `sync-version.py` - sync Cargo package version into plist bundle versions.
  - `build-rust-app.sh` - validate versions before copying plist; copy Sparkle framework when present.
  - `package-release.sh` - create Sparkle-compatible zip using `ditto -c -k --sequesterRsrc --keepParent Shuttle.app ...` so bundle metadata, permissions, symlinks, and framework layout are preserved.
  - `check-rust.sh` - add version consistency validation.
- `Cargo.toml`
  - Keep package version as the canonical version source.
  - Add any minimal macOS-only updater bridge dependency only if direct `objc` calls are insufficient.
- `resources/Shuttle-Info.plist`
  - Keep `CFBundleShortVersionString`/`CFBundleVersion` synchronized.
  - Add Sparkle keys such as `SUFeedURL`, `SUPublicEDKey`, and automatic-check preferences.
- `resources/` or `vendor/`
  - Store or fetch pinned Sparkle framework assets according to repository-size preference.
- `src/macos/`
  - `updater.rs` - create and retain Sparkle updater controller and expose target/action or bridge object.
  - `menu.rs` - insert the update menu item before Configuration.
  - `app.rs` - initialize updater during app startup and pass updater target to menu installation.
  - `mod.rs` - expose updater module.
- `docs/`
  - `packaging-rust-port.md` - document automated release flow, artifacts, signing, notarization, and Sparkle feed.
  - `troubleshooting-rust-port.md` - add update failure diagnostics.

Third-party interfaces:

- GitHub Actions hosted macOS ARM runners for native `aarch64-apple-darwin` release builds.
- GitHub Releases for distributing versioned artifacts.
- GitHub Pages or a committed `gh-pages` branch for hosting Sparkle `appcast.xml`; GitHub release assets host the app zip/DMG.
- Sparkle 2 `SPUStandardUpdaterController` for native macOS update UI and update execution.
- Sparkle framework loading must be explicit: either link the Rust binary with `-framework Sparkle` and configure bundle `@rpath`/`LC_RPATH` correctly, or `dlopen` the bundled `Contents/Frameworks/Sparkle.framework/Sparkle` before Objective-C runtime class lookup. The implementation should prefer build-time linking if it works cleanly in Cargo; otherwise use `dlopen` with clear startup diagnostics.
- Sparkle EdDSA signing tools (`generate_keys`, `sign_update`) for update integrity independent of Apple code signing.

## Performance Considerations

- Sparkle scheduled background checks should use Sparkle's default cadence rather than a custom high-frequency polling timer.
- App startup should not block on network calls; updater initialization should be local and checks should be asynchronous.
- Release workflows should cache Cargo dependencies/build artifacts where safe, but release artifacts must be built from the version-bumped commit.

## Migration Notes

- Existing users without Sparkle-enabled builds will need to install the first Sparkle-enabled release manually. Subsequent releases can update automatically.
- Version `0.1.0` vs `0.2.0` inconsistency must be resolved before the first automated release; use the higher existing app bundle version unless the maintainer chooses otherwise during implementation.
- Sparkle requires stable bundle identifiers and monotonically increasing versions; do not change `CFBundleIdentifier` after shipping updater-enabled builds unless migration is planned.
- If signing/notarization secrets are unavailable initially, publish unsigned experimental artifacts but document that production auto-update should wait for signed/notarized builds.
- Phase 3 may use staging/placeholder `SUFeedURL` and public-key values while wiring runtime behavior; Phase 4 replaces them with the production feed and key.

## Phase 1: Version and Packaging Foundation

Make app versioning deterministic and prepare bundle packaging for release automation.

**Tasks**:

- [x] Add `scripts/sync-version.py` that reads `package.version` from `Cargo.toml` and updates `CFBundleShortVersionString` and `CFBundleVersion` in `resources/Shuttle-Info.plist`.
- [x] Add a validation mode to `scripts/sync-version.py --check` that fails if plist and Cargo versions differ.
- [x] Update `scripts/check-rust.sh` to run the version consistency check.
- [x] Update `scripts/build-rust-app.sh` to run the version consistency check before building or to invoke sync explicitly in release mode.
- [x] Add `scripts/package-release.sh` to produce a release archive from `target/release/Shuttle.app` using `ditto -c -k --sequesterRsrc --keepParent Shuttle.app`.
- [x] Set the initial canonical version to `0.2.0` by syncing `Cargo.toml` and `resources/Shuttle-Info.plist`, preserving the higher currently advertised app bundle version.
- [x] Update `docs/packaging-rust-port.md` with the canonical version source, local packaging commands, and artifact output path.

**Automated Verification**:

- [x] `scripts/sync-version.py --check` passes when versions match.
- [x] `python3 -m unittest discover scripts/tests` passes and covers `sync-version.py` plist update behavior using temporary fixture files.
- [x] `./scripts/check-rust.sh` passes.
- [x] `./scripts/build-rust-app.sh` creates `target/release/Shuttle.app` whose `Contents/Info.plist` version matches `Cargo.toml`.
- [x] `./scripts/package-release.sh` creates a zip artifact containing `Shuttle.app`.

**Manual Verification**:

- [x] Build and inspect the local app bundle version.
  1. Run `./scripts/build-rust-app.sh`.
  2. Run `/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' target/release/Shuttle.app/Contents/Info.plist`.
  3. Confirm it matches `Cargo.toml`.

---

## Phase 2: GitHub Actions CI and SemVer Release Automation

Add hosted CI and an automated main-branch release workflow.

**Tasks**:

- [x] Add `.github/workflows/ci.yml` for pull requests and pushes that runs `./scripts/check-rust.sh` on a macOS ARM runner.
- [x] Add `.github/workflows/release.yml` triggered by pushes to `main` with concurrency protection for releases.
- [x] Configure `googleapis/release-please-action` for Conventional Commits SemVer calculation, release PR/version bump management, changelog generation, tags, and GitHub Release creation.
- [x] Configure Release Please manifest settings so it updates `Cargo.toml`, runs the version sync script for `resources/Shuttle-Info.plist`, and lets Cargo refresh `Cargo.lock` rather than hand-editing it.
- [x] In `release.yml`, guard against recursive releases by relying on Release Please's release-created output and skipping version-bump-only bot commits or commits containing `[skip release]`.
- [x] In `release.yml`, run `./scripts/check-rust.sh`, `./scripts/build-rust-app.sh`, and `./scripts/package-release.sh` from the version-bumped commit.
- [x] Add conditional Apple signing/notarization steps that run only when the required secrets are present: import certificate, sign embedded Sparkle framework and app bundle with hardened runtime, submit with `xcrun notarytool`, wait for acceptance, and staple the notarization ticket.
- [x] Publish the packaged app as a GitHub Release asset with generated release notes.
- [x] Document required repository permissions and secrets in `docs/packaging-rust-port.md`.

**Automated Verification**:

- [x] `actionlint` or equivalent workflow validation passes for `.github/workflows/*.yml`.
- [ ] Release workflow dry-run mode can compute the next version without pushing tags.
- [ ] CI workflow passes on a branch push or pull request.
- [x] `./scripts/check-rust.sh` passes locally after workflow files and scripts are added.

**Manual Verification**:

- [ ] Test release workflow with a temporary branch or `workflow_dispatch` dry run.
  1. Trigger the workflow without publishing a real release.
  2. Confirm it selects a macOS ARM runner.
  3. Confirm computed version, build artifact name, and release notes look correct.
- [ ] After merging to `main`, confirm a real release creates a `vX.Y.Z` tag, GitHub Release, and downloadable Shuttle archive.

---

## Phase 3: Sparkle Runtime Integration

Bundle and initialize Sparkle so users can check for updates from the menu and receive native background update checks.

**Dependencies: Phase 1**

**Tasks**:

- [ ] Add a pinned Sparkle download script for CI/local builds that fetches a specific Sparkle 2 release checksum-verified into `target/vendor/Sparkle.framework`, avoiding committing the binary framework to the repository.
- [ ] Update `scripts/build-rust-app.sh` to place `Sparkle.framework` under `Shuttle.app/Contents/Frameworks/` when Sparkle is available.
- [ ] Add Sparkle metadata to `resources/Shuttle-Info.plist`, including `SUFeedURL`, `SUPublicEDKey`, and automatic-check preferences.
- [ ] Add `src/macos/updater.rs` to explicitly load/link Sparkle, create and retain `SPUStandardUpdaterController` with `startingUpdater: YES`, and report clear errors when Sparkle is unavailable.
- [ ] Update `src/macos/app.rs` to initialize the updater before menu construction and retain it for the app lifetime.
- [ ] Update `src/macos/menu.rs` to add `Check for Updates…` before Configuration and wire it to Sparkle's `checkForUpdates:` action.
- [ ] Add fallback behavior so development builds without Sparkle show a disabled `Updates unavailable in this build` item in the same footer position.
- [ ] Add macOS-only compile guards so non-macOS tests still build.
- [ ] Update `docs/troubleshooting-rust-port.md` with update-check failure cases: missing Sparkle framework, bad appcast URL, invalid EdDSA signature, unsigned/notarization problems.

**Automated Verification**:

- [ ] `cargo test` passes on non-macOS and macOS.
- [ ] Existing `src/macos/menu.rs` menu spec tests are extended or new tests verify the updater menu item appears in the expected footer position when enabled.
- [ ] `./scripts/build-rust-app.sh` produces a bundle containing `Contents/Frameworks/Sparkle.framework` when Sparkle is configured.
- [ ] `./scripts/check-rust.sh` passes.

**Manual Verification**:

- [ ] Launch a local build and verify the update menu item appears.
  1. Build with Sparkle available.
  2. Open Shuttle from `target/release/Shuttle.app`.
  3. Open the status menu.
  4. Confirm `Check for Updates…` appears above Configuration.
- [ ] Click `Check for Updates…` with a test appcast URL and confirm Sparkle shows native update UI or a clear “no updates” message.

---

## Phase 4: Sparkle Appcast Publishing and End-to-End Updates

Generate signed appcast metadata during release and verify that one released build can update to the next.

**Dependencies: Phase 2, Phase 3**

**Tasks**:

- [ ] Generate a Sparkle EdDSA keypair outside the repository and store the private key as a GitHub Actions secret.
- [ ] Store the public EdDSA key in `resources/Shuttle-Info.plist`.
- [ ] Update `release.yml` to run Sparkle `sign_update` on the packaged app archive and capture signature/length/version metadata.
- [ ] Add an appcast generation step that writes `appcast.xml` with release notes, artifact URL, SemVer version, signature, and minimum OS metadata.
- [ ] Publish `appcast.xml` to GitHub Pages or a dedicated release-feed branch and make `SUFeedURL` point to that stable URL.
- [ ] Ensure release asset URLs in the appcast are stable, publicly reachable, and match the signed artifact.
- [ ] Add rollback/recovery docs explaining how to yank a bad appcast entry or publish a fixed update.
- [ ] Add documentation for the first manual install requirement before auto-update can take over.

**Automated Verification**:

- [ ] Appcast generation script validates XML structure.
- [ ] Appcast generation script fails if required signature, version, URL, or length fields are missing.
- [ ] Release workflow uploads both the app archive and updated appcast in a dry run or staging repository.
- [ ] `./scripts/check-rust.sh` passes after appcast tooling changes.

**Manual Verification**:

- [ ] Perform an end-to-end staged update.
  1. Install a lower-version local Shuttle build that points to a staging appcast.
  2. Publish a higher-version staging release artifact and appcast.
  3. Click `Check for Updates…`.
  4. Confirm Sparkle detects the new version, downloads it, verifies it, and offers install/relaunch.
  5. Confirm the relaunched app reports the new bundle version.
- [ ] Confirm a bad signature or modified archive is rejected by Sparkle.

---

## References

- `Cargo.toml:3` - current Rust package version.
- `resources/Shuttle-Info.plist:19-22` - current macOS bundle version fields.
- `scripts/build-rust-app.sh:4-25` - current app bundle assembly.
- `src/macos/menu.rs:61-110` - current footer menu construction.
- `src/macos/app.rs:8-74` - current AppKit startup and status menu install.
- `docs/packaging-rust-port.md:1-23` - current packaging docs and signing/notarization TODO.
- GitHub Actions hosted runner documentation - macOS ARM hosted runners are available.
- Sparkle 2 documentation - `SPUStandardUpdaterController`, appcast feeds, EdDSA update signatures, `SUFeedURL`, `SUPublicEDKey`.
