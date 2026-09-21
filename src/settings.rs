use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::paths;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub fn index(self) -> u32 {
        match self {
            Self::System => 0,
            Self::Light => 1,
            Self::Dark => 2,
        }
    }
    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::Light,
            2 => Self::Dark,
            _ => Self::System,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub last_output: Option<PathBuf>,
    pub last_target: u32,
    pub padding: f32,
    pub keep_transparency: bool,
    #[serde(default)]
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            last_output: Some(paths::default_output()),
            last_target: 5,
            padding: 0.08,
            keep_transparency: true,
            theme: Theme::System,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let Ok(path) = paths::settings_path() else {
            return Self::default();
        };
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let dir = paths::config_dir()?;
        paths::ensure_dir(&dir)?;
        fs::write(paths::settings_path()?, serde_json::to_string_pretty(self).unwrap())?;
        Ok(())
    }
}
