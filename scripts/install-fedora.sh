#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_SRC="$ROOT_DIR/crates/caf-app/target/release/camark"
CLI_SRC="$ROOT_DIR/target/release/camarkctl"
ICON_SRC="$ROOT_DIR/crates/caf-app/icons/128x128.png"

INSTALL_BIN_DIR="$HOME/.local/bin"
INSTALL_APP_DIR="$HOME/.local/share/applications"
INSTALL_ICON_DIR="$HOME/.local/share/icons/hicolor/128x128/apps"

mkdir -p "$INSTALL_BIN_DIR" "$INSTALL_APP_DIR" "$INSTALL_ICON_DIR"

echo "==> Menginstal binary CAMark ke $INSTALL_BIN_DIR..."
cp -f "$BIN_SRC" "$INSTALL_BIN_DIR/camark"
chmod +x "$INSTALL_BIN_DIR/camark"

if [[ -f "$CLI_SRC" ]]; then
  cp -f "$CLI_SRC" "$INSTALL_BIN_DIR/camarkctl"
  chmod +x "$INSTALL_BIN_DIR/camarkctl"
  echo "==> Menginstal binary camarkctl ke $INSTALL_BIN_DIR/camarkctl"
fi

echo "==> Menginstal ikon aplikasi..."
cp -f "$ICON_SRC" "$INSTALL_ICON_DIR/camark.png"

echo "==> Mendaftarkan desktop entry..."
cat <<EOF > "$INSTALL_APP_DIR/com.fathforce.camark.desktop"
[Desktop Entry]
Type=Application
Version=1.0
Name=CAMark
Comment=Markdown Studio & Sovereign Technical Document Editor
Exec=$INSTALL_BIN_DIR/camark %F
Icon=camark
Terminal=false
Categories=Office;Development;TextEditor;Utility;
MimeType=text/markdown;text/x-markdown;text/plain;
Keywords=Markdown;Editor;Notes;Docs;KaTeX;Mermaid;
StartupWMClass=camark
EOF

chmod +x "$INSTALL_APP_DIR/com.fathforce.camark.desktop"

# Refresh GNOME / Desktop database jika command tersedia
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$INSTALL_APP_DIR" 2>/dev/null || true
fi

echo "==> Berhasil! CAMark telah terinstal di Fedora Anda."
echo "    Bisa dijalankan langsung lewat App Menu / Launcher Fedora, atau lewat terminal:"
echo "    $ camark"
