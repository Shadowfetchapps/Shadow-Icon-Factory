use std::path::PathBuf;

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use serde_json::Value;
use shadow_icon_factory::generate::{self, GenerateOptions};
use shadow_icon_factory::spec::{self, Target};

fn write_png(dir: &std::path::Path, name: &str, w: u32, h: u32) -> PathBuf {
    let path = dir.join(name);
    let img = DynamicImage::ImageRgba8(ImageBuffer::from_fn(w, h, |x, y| {
        Rgba([(x % 255) as u8, (y % 255) as u8, 160, 255])
    }));
    img.save(&path).unwrap();
    path
}

fn opts(dir: &std::path::Path, target: Target) -> GenerateOptions {
    GenerateOptions {
        target,
        padding: 0.0,
        background: [0, 0, 0, 0],
        keep_transparency: true,
        output: dir.to_path_buf(),
        overwrite_xcassets: true,
    }
}

#[test]
fn linux_dims_match_spec() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_png(dir.path(), "master.png", 1024, 1024);
    let master = generate::load_master(&src).unwrap();
    let out = dir.path().join("out");
    let report = generate::generate(&master, &src, &opts(&out, Target::Linux)).unwrap();
    let written: Vec<_> = report.files.iter().filter(|f| !f.upscale_blocked).collect();
    assert_eq!(written.len(), spec::slots_for(Target::Linux).len());
    for file in written {
        let img = image::open(&file.path).unwrap();
        assert_eq!(img.dimensions(), (file.size, file.size), "{}", file.path.display());
        assert!(file.path.exists());
    }
    assert_eq!(std::fs::metadata(&src).unwrap().len() > 0, true);
}

#[test]
fn ios_manifest_and_dims() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_png(dir.path(), "ios.png", 1024, 1024);
    let master = generate::load_master(&src).unwrap();
    let out = dir.path().join("out");
    generate::generate(&master, &src, &opts(&out, Target::Ios)).unwrap();
    let json_path = out.join("ios/AppIcon.appiconset/Contents.json");
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap()).unwrap();
    let images = doc["images"].as_array().unwrap();
    assert_eq!(images.len(), spec::slots_for(Target::Ios).len());
    for slot in spec::slots_for(Target::Ios) {
        let path = out.join(&slot.rel_path);
        let img = image::open(&path).expect(&slot.rel_path);
        assert_eq!(img.dimensions(), (slot.size, slot.size));
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(images.iter().any(|i| i["filename"] == name.as_ref()));
    }
}

#[test]
fn android_web_macos_trees() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_png(dir.path(), "all.png", 1024, 1024);
    let original = std::fs::read(&src).unwrap();
    let master = generate::load_master(&src).unwrap();
    let out = dir.path().join("out");
    let report = generate::generate(&master, &src, &opts(&out, Target::All)).unwrap();
    assert!(report.files.iter().any(|f| f.path.ends_with("ic_launcher.png")));
    assert!(out.join("web/favicon.ico").exists());
    assert!(out.join("web/apple-touch-icon.png").exists());
    assert!(out.join("macos/AppIcon.iconset/icon_512x512@2x.png").exists());
    let touch = image::open(out.join("web/apple-touch-icon.png")).unwrap();
    assert_eq!(touch.dimensions(), (180, 180));
    let mdpi = image::open(out.join("android/mipmap-mdpi/ic_launcher.png")).unwrap();
    assert_eq!(mdpi.dimensions(), (48, 48));
    assert_eq!(std::fs::read(&src).unwrap(), original);
}

#[test]
fn no_silent_upscale() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_png(dir.path(), "small.png", 64, 64);
    let master = generate::load_master(&src).unwrap();
    let out = dir.path().join("out");
    let report = generate::generate(&master, &src, &opts(&out, Target::Linux)).unwrap();
    assert!(report.warnings.iter().any(|w| w.contains("skipped") || w.contains("Skipped") || w.contains("Master")));
    let big = out.join("linux/hicolor/1024x1024/apps/icon.png");
    assert!(!big.exists());
    let ok = out.join("linux/hicolor/64x64/apps/icon.png");
    assert!(ok.exists());
    let img = image::open(&ok).unwrap();
    assert_eq!(img.dimensions(), (64, 64));
}

#[test]
fn reject_directory_and_zero_byte() {
    let dir = tempfile::tempdir().unwrap();
    assert!(generate::load_master(dir.path()).is_err());
    let empty = dir.path().join("empty.png");
    std::fs::write(&empty, []).unwrap();
    assert!(generate::load_master(&empty).is_err());
}

#[test]
fn unicode_output_path() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_png(dir.path(), "иконка.png", 128, 128);
    let master = generate::load_master(&src).unwrap();
    let out = dir.path().join("выход icons");
    let report = generate::generate(&master, &src, &opts(&out, Target::Web)).unwrap();
    assert!(report.files.iter().any(|f| f.path.exists() && !f.upscale_blocked));
    assert!(out.join("web/favicon-32.png").exists());
}
