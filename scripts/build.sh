#!/usr/bin/env bash
#
# build.sh — build the application locally.
#
# Produces the release binary and bundles under
# src-tauri/target/release/ (and bundle/).
#
# No CI service is used; this is the local build step.

set -euo pipefail
cd "$(dirname "$0")/.."

pnpm install
pnpm tauri build

echo
echo "Built:"
echo "  src-tauri/target/release/hd2mm"
echo "  src-tauri/target/release/bundle/appimage/*.AppImage"
echo "  src-tauri/target/release/bundle/deb/*.deb"
