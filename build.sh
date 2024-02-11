#!/bin/sh

set -e

cargo build --verbose
cargo test --verbose
cargo bench --no-run
cargo clippy
cargo fmt --check
