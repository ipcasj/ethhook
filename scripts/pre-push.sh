#!/bin/sh
# Pre-push checks for EthHook
set -e

cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --lib --bins
cargo audit || true
