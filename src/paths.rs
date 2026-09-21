use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub const APP_ID: &str = "com.shadowfetch.IconFactory";
pub const APP_NAME: &str = "Shadow Icon Factory";
pub const APP_ICON: &str = "shadow-icon-factory";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_WEBSITE: &str = "https://github.com/ShadowfetchLinux/Shadow-Icon-Factory";

pub fn config_dir() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .ok_or_else(|| Error::user("Could not find the user configuration directory."))?
        .join("shadow-icon-factory"))
}

pub fn settings_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("settings.json"))
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).map_err(|err| {
        Error::detailed(format!("Could not create folder {}", path.display()), err.to_string())
    })
}

pub fn display_home_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

pub fn default_output() -> PathBuf {
    dirs::picture_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Shadow Icon Factory")
}
