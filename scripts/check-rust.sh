#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo check
python3 -m json.tool resources/shuttle.default.json >/dev/null
