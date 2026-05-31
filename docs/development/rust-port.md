# Rust Port Development

Run the local quality gate before committing Rust port changes:

```sh
./scripts/check-rust.sh
```

The script runs formatting, strict Clippy (`-D warnings`), tests, a build check, and JSON validation for the Rust default config.

## Dependency updates

Check compatible dependency updates with:

```sh
cargo update --dry-run --verbose
```

When updating direct dependencies, use `cargo add <crate>` or `cargo add <crate>@<major>` instead of editing `Cargo.toml` manually. This asks Cargo to resolve the newest available compatible requirement and keeps `Cargo.toml` and `Cargo.lock` in sync through Cargo's resolver.

## Git hooks

Install the local hooks by copying or symlinking the Husky-compatible scripts into `.git/hooks`:

```sh
ln -sf ../../.husky/pre-commit .git/hooks/pre-commit
ln -sf ../../.husky/commit-msg .git/hooks/commit-msg
```

The pre-commit hook runs `./scripts/check-rust.sh`. The commit-msg hook enforces Conventional Commits with these types: `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, and `test`.

GitHub also enforces Conventional Commit PR titles in `.github/workflows/conventional-commits.yml`. This protects squash merges because GitHub uses the PR title as the default squash commit title.

Update hooks by editing `.husky/*`. Bypass only for emergencies with `git commit --no-verify`, and run `./scripts/check-rust.sh` before pushing.

Tests should live next to focused modules for unit-level compatibility checks. Use `tests/` for integration tests once process, socket, and macOS-bound launch seams exist.
