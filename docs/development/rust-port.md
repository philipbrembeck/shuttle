# Rust Port Development

Run the local quality gate before committing Rust port changes:

```sh
./scripts/check-rust.sh
```

The script runs formatting, strict Clippy (`-D warnings`), tests, a build check, and JSON validation for the Rust default config.

## Pre-commit hook

Install the local hook by copying or symlinking `.husky/pre-commit` into `.git/hooks/pre-commit`:

```sh
ln -sf ../../.husky/pre-commit .git/hooks/pre-commit
```

Update the hook by editing `.husky/pre-commit`. Bypass only for emergencies with `git commit --no-verify`, and run `./scripts/check-rust.sh` before pushing.

Tests should live next to focused modules for unit-level compatibility checks. Use `tests/` for integration tests once process, socket, and macOS-bound launch seams exist.
