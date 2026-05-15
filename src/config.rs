//! Configuration and standard platform paths

use crate::errors::WindError;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// wind config schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub schema_version: u32,
    pub active_workspace: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: 1,
            active_workspace: None,
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn set_active_workspace(&mut self, root: PathBuf) {
        self.active_workspace = Some(root);
    }
}

/// Platform-standard config path
pub fn config_path() -> anyhow::Result<PathBuf> {
    if let Some(path) = std::env::var_os("WIND_CONFIG_PATH") {
        return Ok(PathBuf::from(path));
    }

    if let Some(proj) = ProjectDirs::from("com", "wind", "wind") {
        Ok(proj.config_dir().join("config.json"))
    } else {
        Err(WindError::ConfigPathUnwritable(
            "unable to determine platform config directory".to_string(),
        )
        .into())
    }
}

/// Get the active workspace root from config
pub fn get_workspace_root() -> anyhow::Result<PathBuf> {
    let config = Config::load()?;
    config
        .active_workspace
        .ok_or_else(|| WindError::NoActiveWorkspace.into())
}
