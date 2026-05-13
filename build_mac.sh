#!/bin/bash
set -e
cd "$(dirname "$0")"
VERSION="${1:-}"
if [ -n "$VERSION" ]; then
    sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
fi
cargo build --release --features "mac" 2>&1 | tee build.log