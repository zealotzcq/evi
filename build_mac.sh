#!/bin/bash
set -e
cd "$(dirname "$0")"
cargo build --release --features "mac" 2>&1 | tee build.log