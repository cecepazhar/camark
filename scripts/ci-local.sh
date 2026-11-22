#!/usr/bin/env bash
# ==============================================================================
# CAMark Local Continuous Integration Suite
# Runs the complete CI pipeline locally with zero dependency on GitHub Actions.
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=================================================="
echo "  CAMark Local CI Suite"
echo "=================================================="

echo "--> [1/5] Architecture & Version Sync Guards..."
cd "$ROOT_DIR"
cargo run -p caf-xtask -- guard

echo "--> [2/5] Rust Formatting & Clippy Lints..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

echo "--> [3/5] Rust Workspace Tests..."
cargo test --workspace

echo "--> [4/5] Frontend Typecheck (svelte-check)..."
cd "$ROOT_DIR/frontend"
npm run check

echo "--> [5/5] Frontend Unit & Integration Tests (vitest)..."
npm test -- --run

echo ""
echo "=================================================="
echo "  🎉 All Local CI Checks Passed Successfully!"
echo "=================================================="
