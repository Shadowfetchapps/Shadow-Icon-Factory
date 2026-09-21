use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use serde_json::{json, Value};

use crate::error::{Error, Result};
use crate::paths;
use crate::spec::{self, RasterSlot, Target};

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub target: Target,
    pub padding: f32,
    pub background: [u8; 4],
    pub keep_transparency: bool,
    pub output: PathBuf,
    pub overwrite_xcassets: bool,
}

#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub size: u32,
    pub upscale_blocked: bool,
}

#[allow(dead_code)]
pub fn generated_file_path(file: &GeneratedFile) -> &Path {
    &file.path
}

#[derive(Debug, Clone)]
pub struct GenerateReport {
    pub files: Vec<GeneratedFile>,
    pub warnings: Vec<String>,
    pub output_root: PathBuf,
}

pub fn load_master(path: &Path) -> Result<DynamicImage> {
    if path.is_dir() {
        return Err(Error::user("Drop a master image, not a folder."));
    }
    if path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("svg")) == Some(true)
    {
        return rasterize_svg(path);
    }
    image::open(path).map_err(|err| Error::detailed("Could not open the master image.", err.to_string()))
}

fn rasterize_svg(path: &Path) -> Result<DynamicImage> {
    let rsvg = which::which("rsvg-convert").map_err(|_| {
        Error::user("This master is SVG. Install librsvg (rsvg-convert) to rasterize it.")
    })?;
    let tmp = std::env::temp_dir().join(format!("shadow-icon-factory-{}.png", std::process::id()));
    let status = Command::new(rsvg)
        .args(["-w", "1024", "-h", "1024"])
        .arg("-o")
        .arg(&tmp)
        .arg(path)
        .status()
        .map_err(|e| Error::detailed("Could not start rsvg-convert.", e.to_string()))?;
    if !status.success() {
        return Err(Error::user("Could not rasterize the SVG master."));
    }
    let img = image::open(&tmp).map_err(|e| Error::detailed("Rasterized SVG was unreadable.", e.to_string()))?;
    let _ = fs::remove_file(tmp);
    Ok(img)
}

pub fn generate(master: &DynamicImage, source: &Path, opts: &GenerateOptions) -> Result<GenerateReport> {
    let (mw, mh) = master.dimensions();
    let min_side = mw.min(mh);
    if min_side < 16 {
        return Err(Error::user("The master image is too small to make icons."));
    }
    let required = spec::max_required(opts.target);
    let mut warnings = Vec::new();
    if min_side < required {
        warnings.push(format!(
            "Master is {min_side}px on the short side. Sizes above that will be skipped (no silent upscale). A 1024px master is recommended."
        ));
    }
    paths::ensure_dir(&opts.output)?;
    let mut files = Vec::new();
    for slot in spec::slots_for(opts.target) {
        if slot.size > min_side {
            warnings.push(format!(
                "Skipped {}px ({}) — master is only {min_side}px.",
                slot.size, slot.rel_path
            ));
            files.push(GeneratedFile {
                path: opts.output.join(&slot.rel_path),
                size: slot.size,
                upscale_blocked: true,
            });
            continue;
        }
        let dest = opts.output.join(&slot.rel_path);
        if dest.exists()
            && dest.to_string_lossy().contains("AppIcon.appiconset")
            && !opts.overwrite_xcassets
        {
            return Err(Error::user(
                "An existing AppIcon.appiconset would be overwritten. Confirm overwrite to continue.",
            ));
        }
        if let Some(parent) = dest.parent() {
            paths::ensure_dir(parent)?;
        }
        let raster = render_slot(master, slot.size, opts);
        raster
            .save(&dest)
            .map_err(|e| Error::detailed(format!("Could not write {}.", dest.display()), e.to_string()))?;
        validate_png(&dest, slot.size)?;
        files.push(GeneratedFile {
            path: dest,
            size: slot.size,
            upscale_blocked: false,
        });
    }

    if matches!(opts.target, Target::Ios | Target::All) {
        write_contents_json(&opts.output.join("ios/AppIcon.appiconset/Contents.json"), &spec::slots_for(Target::Ios))?;
    }
    if matches!(opts.target, Target::Web | Target::All) {
        write_favicon(&opts.output.join("web/favicon.ico"), master, opts, min_side, &mut warnings)?;
    }
    if matches!(opts.target, Target::Linux | Target::All)
        && source.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("svg")) == Some(true)
    {
        let svg_dest = opts.output.join("linux/hicolor/scalable/apps/icon.svg");
        if let Some(parent) = svg_dest.parent() {
            paths::ensure_dir(parent)?;
        }
        fs::copy(source, &svg_dest)?;
        files.push(GeneratedFile {
            path: svg_dest,
            size: 0,
            upscale_blocked: false,
        });
    }

    Ok(GenerateReport {
        files,
        warnings,
        output_root: opts.output.clone(),
    })
}

fn render_slot(master: &DynamicImage, size: u32, opts: &GenerateOptions) -> RgbaImage {
    let padded = apply_padding(master, opts.padding);
    let resized = image::imageops::resize(&padded, size, size, image::imageops::FilterType::Lanczos3);
    if opts.keep_transparency && opts.background[3] == 0 {
        return resized;
    }
    let mut canvas = ImageBuffer::from_pixel(size, size, Rgba(opts.background));
    image::imageops::overlay(&mut canvas, &resized, 0, 0);
    canvas
}

fn apply_padding(master: &DynamicImage, padding: f32) -> RgbaImage {
    let rgba = master.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pad = ((w.max(h) as f32) * padding.clamp(0.0, 0.4)) as u32;
    let side = w.max(h) + pad * 2;
    let mut canvas: RgbaImage = ImageBuffer::from_pixel(side, side, Rgba([0, 0, 0, 0]));
    let x = (side - w) / 2;
    let y = (side - h) / 2;
    image::imageops::overlay(&mut canvas, &rgba, x.into(), y.into());
    canvas
}

fn validate_png(path: &Path, expected: u32) -> Result<()> {
    let img = image::open(path).map_err(|e| {
        Error::detailed("An exported icon could not be opened back.", e.to_string())
    })?;
    let (w, h) = img.dimensions();
    if w != expected || h != expected {
        return Err(Error::detailed(
            "An exported icon has the wrong size.",
            format!("{} is {w}×{h}, expected {expected}×{expected}", path.display()),
        ));
    }
    Ok(())
}

fn write_contents_json(path: &Path, slots: &[RasterSlot]) -> Result<()> {
    let images: Vec<Value> = slots
        .iter()
        .filter(|s| s.idiom.is_some())
        .map(|s| {
            json!({
                "filename": Path::new(&s.rel_path).file_name().unwrap().to_string_lossy(),
                "idiom": s.idiom.unwrap(),
                "scale": s.scale.unwrap_or("1x"),
                "size": point_size(s),
            })
        })
        .collect();
    let doc = json!({
        "images": images,
        "info": { "author": "shadow-icon-factory", "version": 1 }
    });
    if let Some(parent) = path.parent() {
        paths::ensure_dir(parent)?;
    }
    let mut f = fs::File::create(path)?;
    f.write_all(serde_json::to_string_pretty(&doc).unwrap().as_bytes())?;
    Ok(())
}

fn point_size(slot: &RasterSlot) -> String {
    let scale: f32 = slot
        .scale
        .unwrap_or("1x")
        .trim_end_matches('x')
        .parse()
        .unwrap_or(1.0);
    let pts = slot.size as f32 / scale;
    if (pts - pts.round()).abs() < 0.05 {
        format!("{}x{}", pts.round() as u32, pts.round() as u32)
    } else {
        format!("{pts:.1}x{pts:.1}")
    }
}

fn write_favicon(
    dest: &Path,
    master: &DynamicImage,
    opts: &GenerateOptions,
    min_side: u32,
    warnings: &mut Vec<String>,
) -> Result<()> {
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    for size in [16u32, 32, 48] {
        if size > min_side {
            warnings.push(format!("favicon.ico skipped {size}px — master too small."));
            continue;
        }
        let rgba = render_slot(master, size, opts);
        let img = ico::IconImage::from_rgba_data(size, size, rgba.into_raw());
        icon_dir.add_entry(ico::IconDirEntry::encode(&img).map_err(|e| {
            Error::detailed("Could not encode favicon.ico.", e.to_string())
        })?);
    }
    if let Some(parent) = dest.parent() {
        paths::ensure_dir(parent)?;
    }
    let mut file = fs::File::create(dest)?;
    icon_dir
        .write(&mut file)
        .map_err(|e| Error::detailed("Could not write favicon.ico.", e.to_string()))?;
    Ok(())
}

#[allow(dead_code)]
pub fn write_into_xcassets(
    report: &GenerateReport,
    xcassets: &Path,
    confirm: bool,
) -> Result<PathBuf> {
    if !confirm {
        return Err(Error::user(
            "Writing into an existing xcassets folder needs an explicit confirm.",
        ));
    }
    let dest = xcassets.join("AppIcon.appiconset");
    if dest.exists() {
        // Replace only our icon set folder after confirm.
        fs::remove_dir_all(&dest)?;
    }
    let src = report.output_root.join("ios/AppIcon.appiconset");
    copy_dir(&src, &dest)?;
    Ok(dest)
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    paths::ensure_dir(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let dest = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}
