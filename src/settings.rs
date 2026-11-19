use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config_dir;

/// Application settings persisted to disk as TOML.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    /// Window width in logical pixels.
    pub width: u32,
    /// Window height in logical pixels.
    pub height: u32,
    /// Background color as a `#rrggbb` hex string.
    pub color: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            color: "#000000".to_owned(),
        }
    }
}

impl Settings {
    /// Returns the filesystem path to the settings file.
    fn path() -> std::path::PathBuf {
        config_dir().join("settings.toml")
    }

    /// Loads settings from disk, falling back to defaults if the file is missing.
    pub fn load() -> Result<Self> {
        let path = Self::path();
        match std::fs::read_to_string(&path) {
            Ok(contents) => {
                let settings = toml::from_str(&contents)?;
                Ok(settings)
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(anyhow::anyhow!(
                "Can not read settings file {}: {err}",
                path.display()
            )),
        }
    }

    /// Writes all settings to disk, creating the parent directory if necessary.
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                anyhow::anyhow!(
                    "Can not create settings directory {}: {err}",
                    parent.display()
                )
            })?;
        }
        let contents = toml::to_string(self)
            .map_err(|err| anyhow::anyhow!("Can not serialize settings: {err}"))?;
        std::fs::write(&path, contents).map_err(|err| {
            anyhow::anyhow!("Can not write settings file {}: {err}", path.display())
        })?;
        Ok(())
    }
}
