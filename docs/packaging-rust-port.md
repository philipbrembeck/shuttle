# Rust Port Packaging

Build and assemble a local `.app` bundle:

```sh
./scripts/check-rust.sh
./scripts/build-rust-app.sh
```

The bundle is written to:

```text
target/release/Shuttle.app
```

Create a release archive with:

```sh
./scripts/package-release.sh
```

The archive is written to:

```text
target/release/Shuttle.zip
```

`Cargo.toml` `package.version` is the canonical app version. Keep bundle metadata synchronized with:

```sh
python3 scripts/sync-version.py
python3 scripts/sync-version.py --check
```

`./scripts/check-rust.sh` and `./scripts/build-rust-app.sh` both fail when `resources/Shuttle-Info.plist` does not match the Cargo version. The app bundle's `CFBundleShortVersionString` and `CFBundleVersion` must both equal `package.version`.

The packaging script copies:

- `resources/Shuttle-Info.plist` to `Contents/Info.plist`
- `target/release/shuttle-rs` to `Contents/MacOS/shuttle-rs`
- `resources/shuttle.default.json` to `Contents/Resources/shuttle.default.json`
- `shuttle.icns` to `Contents/Resources/shuttle.icns` when present

`package-release.sh` uses `ditto -c -k --sequesterRsrc --keepParent Shuttle.app` so bundle metadata, permissions, symlinks, and future embedded frameworks are preserved.

## Automated GitHub releases

CI runs `./scripts/check-rust.sh` on `macos-latest` for pull requests and pushes. The release workflow runs on pushes to `main` and uses Release Please to create release PRs, calculate Conventional Commits SemVer bumps, update `Cargo.toml`, `Cargo.lock`, `resources/Shuttle-Info.plist`, `CHANGELOG.md`, create `vX.Y.Z` tags, and create GitHub Releases.

Required repository workflow permissions:

- `contents: write` for tags, releases, and release asset uploads.
- `pull-requests: write` for Release Please release PRs.
- Repository setting: **Settings → Actions → General → Workflow permissions → Allow GitHub Actions to create and approve pull requests** must be enabled for the default `GITHUB_TOKEN` to open Release Please PRs.

Optional release automation secret:

- `RELEASE_PLEASE_TOKEN`: fine-grained PAT or GitHub App token with repository contents and pull-request write access. Use this when the repository cannot enable GitHub Actions PR creation for the default `GITHUB_TOKEN`.

Optional signing/notarization secrets:

- `APPLE_DEVELOPER_ID_CERTIFICATE_P12`: base64-encoded Developer ID Application certificate in `.p12` format.
- `APPLE_DEVELOPER_ID_CERTIFICATE_PASSWORD`: password for the `.p12` certificate.
- `APPLE_DEVELOPER_ID_IDENTITY`: codesign identity name, for example `Developer ID Application: Example, Inc. (TEAMID)`.
- `APPLE_ID`: Apple ID used with `notarytool`.
- `APPLE_TEAM_ID`: Apple Developer Team ID.
- `APPLE_APP_SPECIFIC_PASSWORD`: app-specific password for notarization.

When Developer ID signing secrets are absent, the workflow ad-hoc signs the app before packaging. This is useful for preserving bundle integrity in experimental archives, but it does not satisfy Gatekeeper for downloaded production releases. Users may still need to remove quarantine manually with `xattr -dr com.apple.quarantine /Applications/Shuttle.app`, and production auto-update releases should be Developer ID signed and notarized before being advertised to users.
