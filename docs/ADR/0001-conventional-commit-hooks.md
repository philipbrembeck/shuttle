# ADR 0001: Enforce Conventional Commits in local Git hooks

## Status

Accepted

## Context

Shuttle uses release automation that benefits from machine-readable commit messages. The repository already keeps Husky-compatible hook scripts under `.husky/`, but only the pre-commit quality gate was present.

## Decision

Add a `.husky/commit-msg` hook that validates the first commit-message line against Conventional Commits:

```text
<type>[optional scope][!]: <description>
```

The hook allows the common release-oriented types `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, and `test`. It also allows Git-generated merge/revert messages and autosquash `fixup!`/`squash!` commits.

## Consequences

Local contributors who install the hook get immediate feedback before creating non-conforming commits. The hook remains dependency-free and does not require Node or npm tooling for this Rust-native repository.

Local hooks can be bypassed with `--no-verify`, so protected branches should also use GitHub-side checks or rulesets when strict enforcement is required. GitHub cannot install custom server-side Git hooks, so Shuttle uses a required PR-title check to cover squash merges where the PR title becomes the default commit title.
