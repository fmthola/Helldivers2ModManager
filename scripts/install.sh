#!/usr/bin/env bash
#
# install.sh — install the AppImage onto the host desktop.
#
# The AppImage is self-contained and runs directly on the host (no distrobox
# needed to run it). This copies it into ~/.local/bin and adds a menu entry.
#
# Build first: scripts/build.sh

set -euo pipefail
cd "$(dirname "$0")/.."

APPIMAGE="$(ls src-tauri/target/release/bundle/appimage/*.AppImage 2>/dev/null | head -1 || true)"
if [ -z "$APPIMAGE" ]; then
  echo "No AppImage found. Run scripts/build.sh first." >&2
  exit 1
fi

BIN_DIR="$HOME/.local/bin"
ICON_DIR="$HOME/.local/share/icons"
DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$BIN_DIR" "$ICON_DIR" "$DESKTOP_DIR"

TARGET="$BIN_DIR/HD2ModManager.AppImage"
install -m 755 "$APPIMAGE" "$TARGET"
cp src-tauri/icons/128x128.png "$ICON_DIR/hd2mm.png" 2>/dev/null || true

cat > "$DESKTOP_DIR/hd2mm.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Helldivers 2 Mod Manager
Comment=Mod manager for Helldivers 2
Exec=env WEBKIT_DISABLE_DMABUF_RENDERER=1 $TARGET
Icon=hd2mm
Categories=Game;Utility;
Terminal=false
EOF

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$DESKTOP_DIR" || true

echo "Installed:"
echo "  binary : $TARGET"
echo "  menu   : Helldivers 2 Mod Manager"
echo
echo "Launch it from the app menu, or run: $TARGET"
