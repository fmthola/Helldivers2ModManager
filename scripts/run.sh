#!/usr/bin/env bash
#
# run.sh — launch the app in the foreground so all logs and errors stream to the
# terminal. This is how you see errors in this desktop app; it has no browser
# devtools. The same output is also written to ~/.local/share/hd2mm/hd2mm.log.
#
# Captures Rust logs, frontend log calls, and uncaught frontend errors.

set -euo pipefail

APP="$HOME/.local/bin/HD2ModManager.AppImage"
if [ ! -x "$APP" ]; then
  APP="$(dirname "$0")/../src-tauri/target/release/hd2mm"
fi
[ -x "$APP" ] || { echo "No built app found. Run scripts/build.sh (and scripts/install.sh)." >&2; exit 1; }

echo ">> launching: $APP"
echo ">> logs also at: ${XDG_DATA_HOME:-$HOME/.local/share}/hd2mm/hd2mm.log"
echo ">> close the window to stop."
exec "$APP" "$@"
