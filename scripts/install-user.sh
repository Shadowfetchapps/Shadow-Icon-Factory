#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
[[ -x "$ROOT/target/release/shadow-icon-factory" ]] || (cd "$ROOT" && cargo build --release)
PREFIX="${XDG_DATA_HOME:-$HOME/.local/share}"
install -D -m 0755 "$ROOT/target/release/shadow-icon-factory" "$HOME/.local/bin/shadow-icon-factory"
install -D -m 0644 "$ROOT/data/com.shadowfetch.IconFactory.desktop" "$PREFIX/applications/com.shadowfetch.IconFactory.desktop"
install -D -m 0644 "$ROOT/data/icons/hicolor/scalable/apps/shadow-icon-factory.svg" \
  "$PREFIX/icons/hicolor/scalable/apps/shadow-icon-factory.svg"
for size in 16 24 32 48 64 128 256 512; do
  install -D -m 0644 "$ROOT/data/icons/hicolor/${size}x${size}/apps/shadow-icon-factory.png" \
    "$PREFIX/icons/hicolor/${size}x${size}/apps/shadow-icon-factory.png"
done
update-desktop-database "$PREFIX/applications" || true
gtk-update-icon-cache -f -t "$PREFIX/icons/hicolor" >/dev/null 2>&1 || true
desktop-file-validate "$PREFIX/applications/com.shadowfetch.IconFactory.desktop"
test -x "$HOME/.local/bin/shadow-icon-factory"
echo "Installed $HOME/.local/bin/shadow-icon-factory"
