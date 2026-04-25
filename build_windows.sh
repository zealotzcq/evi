#!/bin/bash
set -e
cd "$(dirname "$0")"
cargo build --release --target x86_64-pc-windows-msvc 2>&1 | tee build.log