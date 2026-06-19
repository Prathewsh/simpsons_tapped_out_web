#!/usr/bin/env bash
# build.sh — compile Rust to WASM and copy output to web/pkg/
set -e

echo "→ Building WASM (release)..."
wasm-pack build --target web --out-dir web/pkg --release

echo "→ Done. Serve the web/ directory with any static server."
echo "   e.g.  npx -y serve web"
