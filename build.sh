#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-target}"
set -e

cargo fmt

cargo build --all --release
mkdir -p bin
cp "$TARGET"/release/multi_send ./bin/