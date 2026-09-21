# Shadow Icon Factory

**Generate Linux, iOS, macOS, Web, and Android icon sets from one master image.**

Native GTK4 / libadwaita desktop app. Offline. No accounts, ads, telemetry, or cloud.

![Shadow Icon Factory icon](data/icons/hicolor/128x128/apps/shadow-icon-factory.png)

## What it does

- Open or drop a PNG, JPEG, WebP, or SVG master (1024×1024 recommended)
- Preview with a safe-area overlay (inner 80% guide — not an OS mask)
- Targets: **iOS**, **macOS**, **Linux**, **Web**, **Android**, or **All**
- Padding, background (transparent / white / black / brand indigo), keep transparency
- Writes real files and **validates dimensions** after export
- iOS `AppIcon.appiconset` + `Contents.json`
- macOS `AppIcon.iconset` 16–512 @1x/@2x
- Linux hicolor 16–1024 (copies SVG into `scalable/` when the master is SVG)
- Android `mipmap-*` launcher PNGs
- Web favicon PNGs, `favicon.ico`, apple-touch 180
- Never silently upscales. Slots larger than the master are skipped and explained
- Never silently overwrites an existing export folder or App Icon set
- Originals stay intact

## Screenshots

Screenshots belong in `docs/screenshots/` and must not include private filenames or home-directory paths. If this folder is empty, the app still builds and runs; capture them locally after install if you want gallery images.

## Install (user, no root)

```bash
git clone https://github.com/ShadowfetchLinux/Shadow-Icon-Factory.git
cd Shadow-Icon-Factory
cargo build --release
./scripts/install-user.sh
```

This copies the binary to `~/.local/bin/shadow-icon-factory`, the desktop file to `~/.local/share/applications/`, and icons to `~/.local/share/icons/hicolor/`.

Launch from the app menu or:

```bash
shadow-icon-factory
```

A `.deb` is produced with `./scripts/build-deb.sh` (see [BUILDING.md](BUILDING.md)). System-wide `dpkg -i` needs administrator rights; the user install does not.

## Requirements

- Linux desktop with GTK 4 and libadwaita
- `rsvg-convert` (librsvg) only when the master is SVG

## Limitations (honest)

- Masters are exported as **squares**. Non-square sources are centered with padding; they are not cropped automatically
- The app does **not** fake iOS/Android rounded-rect masks. Those are applied by the OS
- Color management is practical RGB, not a full CMS. Prefer an sRGB master for web/app assets
- `favicon.ico` includes 16/32/48 when the master is large enough
- SVG rasterizes at 1024 via `rsvg-convert`

## License

MIT. See [LICENSE](LICENSE).
