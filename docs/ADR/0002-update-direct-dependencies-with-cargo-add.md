# ADR 0002: Update direct dependencies with Cargo commands

## Status

Accepted

## Context

Shuttle keeps Rust dependencies in `Cargo.toml` and the resolved graph in `Cargo.lock`. Manually editing version requirements can accidentally miss the newest compatible requirement, skip resolver feedback, or leave the lockfile out of sync.

## Decision

When changing direct dependency requirements, use Cargo commands such as:

```sh
cargo add <crate>
cargo add <crate>@<major>
```

Do not manually edit direct dependency version requirements unless Cargo cannot express the required change. After changing requirements, run the normal quality gate.

## Consequences

Cargo chooses the current published requirement and updates the manifest through its own parser. The lockfile is updated by Cargo, reducing drift between declared and resolved dependencies.

Major-version upgrades still require compatibility verification because Cargo cannot guarantee application-level behavior across semver-major changes.
