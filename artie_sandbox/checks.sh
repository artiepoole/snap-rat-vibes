#!/usr/bin/bash

set -x

cargo check
cargo clippy --workspace --all-targets --all-features --fix --allow-dirty
yamlfmt .
cargo fmt