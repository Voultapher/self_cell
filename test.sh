#!/usr/bin/env bash

set -euxo pipefail

rm Cargo.lock
cargo +1.36.0-x86_64-unknown-linux-gnu test --features=old_rust --test self_cell
cargo test
cargo +nightly miri test --test self_cell