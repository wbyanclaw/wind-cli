//! Configuration and standard platform paths

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    if let Some(proj) = ProjectDirs::from("com", "wind", "wind") {
        Ok(proj.config_dir().join("config.json"))
    } else {
        // Fallback
        Ok(PathBuf::from("./wind.config.json"))
    }
}

/// Get the active workspace root from config
pub fn get_workspace_root() -> anyhow::Result<PathBuf> {
    let config = Config::load()?;
    config
        .active_workspace
        .ok_or_else(|| anyhow::anyhow!("no active workspace; run 'wind init' first"))
}

/// Workspace data directory
pub fn workspace_data_dir() -> anyhow::Result<PathBuf> {
    if let Some(proj) = ProjectDirs::from("com", "wind", "wind") {
        Ok(proj.data_dir().to_path_buf())
    } else {
        Ok(PathBuf::from("./wind.data"))
    }
}
