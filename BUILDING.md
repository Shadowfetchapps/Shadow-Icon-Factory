# Building

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev librsvg2-bin desktop-file-utils
cargo test --lib --tests
cargo build --release
./scripts/generate-icons.sh
./scripts/install-user.sh
./scripts/build-deb.sh
```

`rustfmt` and `clippy` are optional. This environment may not ship them with the system `rustc`.
