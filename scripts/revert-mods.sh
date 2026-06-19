#!/usr/bin/env bash
#
# revert-mods.sh — remove all deployed mod patch files from the game's data dir,
# restoring the unmodded state. A safety net independent of the app's Purge.
#
# It only deletes files matching the mod-manager patch pattern
# (<16 hex>.patch_<n>[.gpu_resources|.stream]); base game files never match it,
# so they are never touched.
#
# Usage: revert-mods.sh [GAME_PATH]
#   GAME_PATH defaults to the path saved in the app settings.

set -euo pipefail

SETTINGS="${XDG_DATA_HOME:-$HOME/.local/share}/hd2mm/settings.json"
GAME="${1:-}"
if [ -z "$GAME" ] && [ -f "$SETTINGS" ]; then
  GAME="$(python3 -c "import json;print(json.load(open('$SETTINGS')).get('GamePath',''))" 2>/dev/null || true)"
fi
[ -n "$GAME" ] || { echo "usage: revert-mods.sh [GAME_PATH] (or save it in the app first)" >&2; exit 1; }

DATA="$GAME/data"
[ -d "$DATA" ] || { echo "no data dir at: $DATA" >&2; exit 1; }

echo "Reverting mods in: $DATA"
count=0
shopt -s nullglob
for f in "$DATA"/*; do
  base="$(basename "$f")"
  if [[ "$base" =~ ^[0-9a-f]{16}\.patch_[0-9]+(\.gpu_resources|\.stream)?$ ]]; then
    rm -f "$f" && count=$((count + 1))
  fi
done
echo "Removed $count mod patch file(s); base game files untouched."
