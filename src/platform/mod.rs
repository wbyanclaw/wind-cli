//! Platform abstraction layer
//!
//! P0 boundary:
//!   - open_uri: open a URI via the system browser (sandboxed)
//!   - get_workspace_root: resolve active workspace root
//!
//! PLATFORM::launch(cmd) is deleted in P0 — no arbitrary command execution

use std::path::PathBuf;

/// Open a URI via the system (sandboxed — no arbitrary command execution)
#[cfg(target_os = "windows")]
pub fn open_uri(_uri: &str) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn open_uri(_uri: &str) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn open_uri(_uri: &str) -> anyhow::Result<()> {
    // Linux: xdg-open
    // P0 intentionally stubs this — open_uri is for future use
    Ok(())
}

/// Get the active workspace root path
pub fn get_workspace_root() -> anyhow::Result<PathBuf> {
    crate::config::get_workspace_root()
}