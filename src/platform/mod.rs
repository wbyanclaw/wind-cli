//! Platform abstraction layer

use crate::errors::WindError;
use std::path::PathBuf;

/// Open a URI via the system (sandboxed — no arbitrary command execution).
/// P0: always returns an error to prevent silent failure.
pub fn open_uri(_uri: &str) -> anyhow::Result<()> {
    Err(anyhow::anyhow!(
        "open_uri is not supported in this release"
    ))
}

/// Get the active workspace root path.
pub fn get_workspace_root() -> anyhow::Result<PathBuf> {
    crate::config::get_workspace_root()
}