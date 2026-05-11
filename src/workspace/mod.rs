//! Workspace file operations — all paths go through safe_path()

use crate::errors::WindError;
use std::fs;
use std::path::{Path, PathBuf};

/// Validate and resolve a path against workspace root (P0: no-follow symlink)
pub fn safe_path(workspace_root: &Path, user_path: &Path) -> anyhow::Result<PathBuf> {
    // Reject absolute paths
    if user_path.is_absolute() {
        return Err(WindError::PathTraversal.into());
    }

    // Reject symlinks in any component (P0 no-follow)
    if has_symlink_component(user_path) {
        return Err(WindError::SymlinkNotSupported.into());
    }

    // Resolve and canonicalize the user path relative to workspace root
    let joined = workspace_root.join(user_path);
    let canonical = joined
        .canonicalize()
        .map_err(|_| WindError::PathNotFound(user_path.display().to_string()))?;

    // Ensure result is still inside workspace root
    let root_canonical = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());

    if !canonical.starts_with(&root_canonical) {
        return Err(WindError::PathOutsideWorkspace(user_path.display().to_string()).into());
    }

    Ok(canonical)
}

/// Check if any component of the path is a symlink (no-follow P0 strategy)
fn has_symlink_component(path: &Path) -> bool {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if current.is_symlink() {
            return true;
        }
    }
    false
}

/// List directory entries
pub fn ls(path: &Path) -> anyhow::Result<Vec<serde_json::Value>> {
    if !path.exists() {
        return Err(WindError::PathNotFound(path.display().to_string()).into());
    }
    if !path.is_dir() {
        return Err(WindError::PathIsNotDir(path.display().to_string()).into());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_symlink = entry.path().is_symlink();

        entries.push(serde_json::json!({
            "name": name,
            "type": if file_type.is_dir() {
                "dir"
            } else if file_type.is_symlink() {
                "symlink"
            } else {
                "file"
            },
            "symlink": is_symlink
        }));
    }

    Ok(entries)
}

/// Read a file, enforcing size limit
pub fn cat(path: &Path, size_limit: u64) -> anyhow::Result<String> {
    if !path.exists() {
        return Err(WindError::PathNotFound(path.display().to_string()).into());
    }
    if path.is_dir() {
        return Err(WindError::PathIsDir(path.display().to_string()).into());
    }
    if path.is_symlink() {
        return Err(WindError::SymlinkNotSupported.into());
    }

    let metadata = fs::metadata(path)?;
    if metadata.len() > size_limit {
        return Err(WindError::FileTooLarge {
            limit: size_limit,
            path: path.display().to_string(),
        }
        .into());
    }

    fs::read_to_string(path).map_err(|e| WindError::PermissionDenied(e.to_string()).into())
}

/// Write a file using target-directory temp file + rename (atomic)
pub fn put(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Temp file in same directory as target → same filesystem → atomic rename
    let temp_name = format!(
        ".wind.tmp.{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let temp_path = path.with_file_name(temp_name);

    fs::write(&temp_path, content)?;

    // Rename to target (atomic on same filesystem)
    match fs::rename(&temp_path, path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices
            || e.raw_os_error() == Some(18 /* EXDEV on Linux */) => {
            // Cross-device rename failed: fail explicitly, do NOT fallback to copy+delete
            let _ = fs::remove_file(&temp_path);
            Err(WindError::AtomicRenameFailed(
                format!("cross-filesystem atomic rename not supported: {}", e)
            ).into())
        }
        Err(e) => {
            let _ = fs::remove_file(&temp_path);
            Err(e.into())
        }
    }
}

/// Create a directory
pub fn mkdir(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        return Err(WindError::PathExists(path.display().to_string()).into());
    }
    fs::create_dir_all(path)
        .map_err(|e| WindError::PermissionDenied(e.to_string()).into())
}

/// Remove a file or directory
pub fn rm(path: &Path, recursive: bool, yes: bool, dry_run: bool) -> anyhow::Result<()> {
    if path.is_symlink() {
        return Err(WindError::SymlinkNotSupported.into());
    }

    if path.is_dir() {
        // Check if directory is empty
        let is_empty = path.read_dir().map(|mut i| i.next().is_none()).unwrap_or(true);
        if !is_empty && (!recursive || !yes) {
            return Err(WindError::DirNotEmpty(path.display().to_string()).into());
        }
        if !yes {
            return Err(WindError::Usage(
                "directory not empty: use --recursive --yes to confirm deletion".to_string(),
            )
            .into());
        }
    }

    if dry_run {
        return Ok(());
    }

    if path.is_dir() {
        if recursive {
            fs::remove_dir_all(path).map_err(|e| WindError::PermissionDenied(e.to_string()))?;
        } else {
            fs::remove_dir(path).map_err(|e| WindError::PermissionDenied(e.to_string()))?;
        }
    } else {
        fs::remove_file(path).map_err(|e| WindError::PermissionDenied(e.to_string()))?;
    }

    Ok(())
}
