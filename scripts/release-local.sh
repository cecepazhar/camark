#!/usr/bin/env bash
# ==============================================================================
# CAMark Standalone Local Release Pipeline
# Builds, packages, calculates SHA256 checksums, and uploads directly via
# GitHub CLI / mirrors without requiring GitHub Actions compute runners.
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"

RAW_VER="$(grep '^version' "$ROOT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)"
TAG="${1:-v$RAW_VER}"
if [[ "$TAG" != v* ]]; then
  TAG="v$TAG"
fi

TARGET="${2:-linux}"

echo "=================================================="
echo "  CAMark Standalone Local Release Pipeline"
echo "  Tag: $TAG | Platform Target: $TARGET"
echo "=================================================="

# 1. Run local CI verification
echo "--> [1/4] Running pre-release local CI verification..."
"$SCRIPT_DIR/ci-local.sh"

# 2. Build multi-platform packages
echo "--> [2/4] Building packages via build-all.sh ($TARGET)..."
"$SCRIPT_DIR/build-all.sh" "$TARGET"

# 3. Generate SHA256SUMS
echo "--> [3/4] Generating cryptographic SHA256 checksums..."
cd "$DIST_DIR"
rm -f SHA256SUMS
sha256sum * > SHA256SUMS 2>/dev/null || shasum -a 256 * > SHA256SUMS
cat SHA256SUMS

# 4. Distribution instructions
echo ""
echo "=================================================="
echo "  Release artifacts ready in: $DIST_DIR"
echo "=================================================="
echo "To publish draft release to GitHub without GitHub Actions:"
echo "  gh release create \"$TAG\" $DIST_DIR/* --draft --title \"CAMark $TAG\" --notes-file \"$ROOT_DIR/CHANGELOG.md\""
echo ""
echo "To publish across all mirrors (R2, YPC MinIO, GCC):"
echo "  RELEASE_TAG=\"$TAG\" RELEASE_SLUG=\"camark\" ./scripts/distribute-release.sh --dir dist"
