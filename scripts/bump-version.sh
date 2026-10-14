#!/usr/bin/env bash
set -euo pipefail

# Scripts to automatically increment patch version across CAMark manifests
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

CURRENT_VER=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VER"
NEW_PATCH=$((PATCH + 1))
NEW_VER="${MAJOR}.${MINOR}.${NEW_PATCH}"

echo "Bumping version: $CURRENT_VER -> $NEW_VER"

# 1. Cargo.toml workspace
sed -i "s/^version = \"$CURRENT_VER\"/version = \"$NEW_VER\"/" Cargo.toml

# 2. Crates referencing caf-core exact version
sed -i "s/caf-core = { path = \"\.\.\/caf-core\", version = \"=$CURRENT_VER\" }/caf-core = { path = \"..\/caf-core\", version = \"=$NEW_VER\" }/g" crates/caf-app/Cargo.toml crates/caf-cli/Cargo.toml
sed -i "s/caf-core = { version = \"$CURRENT_VER\", path = \"\.\.\/caf-core\" }/caf-core = { version = \"$NEW_VER\", path = \"..\/caf-core\" }/g" crates/caf-migrator/Cargo.toml

# 3. app.toml
sed -i "s/^version = \"$CURRENT_VER\"/version = \"$NEW_VER\"/" app.toml

# 4. tauri.conf.json
sed -i "s/\"version\": \"$CURRENT_VER\"/\"version\": \"$NEW_VER\"/" crates/caf-app/tauri.conf.json

# 5. frontend/package.json
sed -i "s/\"version\": \"$CURRENT_VER\"/\"version\": \"$NEW_VER\"/" frontend/package.json

# 6. Run codegen
cargo run -p caf-xtask -- codegen

echo "Version successfully bumped to $NEW_VER!"
