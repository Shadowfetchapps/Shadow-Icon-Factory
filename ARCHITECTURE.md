# Architecture

The library (`spec`, `generate`) owns sizes and files. GTK only displays them.

- Size tables live in `spec.rs` (iOS / macOS / Linux / Web / Android)
- `generate` resizes with Lanczos3, writes PNGs, then opens each file back and checks width/height
- Slots larger than the master short side are skipped (no silent upscale)
- iOS `Contents.json` is generated from the same table used to write files
- `favicon.ico` is built with the `ico` crate
- SVG masters go through `rsvg-convert` at 1024, then the same raster pipeline
- Temps for SVG rasterization are unique per process and deleted after load
