#!/usr/bin/env bash
set -euo pipefail

python3 scripts/sync-version.py --check
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo check
python3 -m json.tool resources/shuttle.default.json >/dev/null
