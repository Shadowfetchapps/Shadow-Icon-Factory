#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SVG="$ROOT/data/icons/hicolor/scalable/apps/shadow-icon-factory.svg"
BASE="$ROOT/data/icons/hicolor"
for size in 16 24 32 48 64 128 256 512; do
  dir="$BASE/${size}x${size}/apps"
  mkdir -p "$dir"
  rsvg-convert -w "$size" -h "$size" -o "$dir/shadow-icon-factory.png" "$SVG"
done
mkdir -p "$BASE/symbolic/apps"
rsvg-convert -w 16 -h 16 -o "$BASE/symbolic/apps/shadow-icon-factory-symbolic.png" "$SVG"
echo "Generated PNG icons from $SVG"
