#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
cargo fmt --all
cargo check --offline
cargo check --offline --examples
