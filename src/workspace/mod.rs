//! Workspace file operations — all paths go through safe_path()

use crate::errors::WindError;
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

/// Validate and resolve a path against workspace root (P0: no-follow symlink)
pub fn safe_path(workspace_root: &Path, user_path: &Path) -> anyhow::Result<PathBuf> {
    resolve_workspace_path(workspace_root, user_path, true)
}

/// Validate a workspace path whose final component may not exist yet.
pub fn safe_path_for_create(workspace_root: &Path, user_path: &Path) -> anyhow::Result<PathBuf> {
    resolve_workspace_path(workspace_root, user_path, false)
}

fn resolve_workspace_path(
    workspace_root: &Path,
    user_path: &Path,
    must_exist: bool,
) -> anyhow::Result<PathBuf> {
    if user_path.is_absolute() {
        return Err(WindError::PathTraversal.into());
    }

    let root_canonical = workspace_root
        .canonicalize()
        .map_err(|_| WindError::PathNotFound(workspace_root.display().to_string()))?;

    let mut current = root_canonical.clone();
    let components: Vec<_> = user_path.components().collect();
    let mut saw_normal = false;

    for (index, component) in components.iter().enumerate() {
        match component {
            Component::CurDir => continue,
            Component::Normal(part) => {
                saw_normal = true;
                current.push(part);
            }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(WindError::PathTraversal.into());
            }
        }

        let is_last = index == components.len() - 1;
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if is_symlink_like(&metadata) {
                    return Err(WindError::SymlinkNotSupported.into());
                }
                if !is_last && !metadata.is_dir() {
                    return Err(WindError::PathIsNotDir(current.display().to_string()).into());
                }
            }
            Err(_) if must_exist || !is_last => {
                return Err(WindError::PathNotFound(current.display().to_string()).into());
            }
            Err(_) => {}
        }
    }

    if !saw_normal {
        return Ok(root_canonical);
    }

    if current.exists() {
        let canonical = current
            .canonicalize()
            .map_err(|_| WindError::PathNotFound(current.display().to_string()))?;
        if !canonical.starts_with(&root_canonical) {
            return Err(WindError::PathOutsideWorkspace(user_path.display().to_string()).into());
        }
        Ok(canonical)
    } else {
        let parent = current
            .parent()
            .ok_or_else(|| WindError::PathTraversal)?;
        let parent_canonical = parent
            .canonicalize()
            .map_err(|_| WindError::PathNotFound(parent.display().to_string()))?;
        if !parent_canonical.starts_with(&root_canonical) {
            return Err(WindError::PathOutsideWorkspace(user_path.display().to_string()).into());
        }
        Ok(current)
    }
}

fn is_symlink_like(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }

    #[cfg(windows)]
    {
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
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
        let metadata = fs::symlink_metadata(entry.path())?;
        let file_type = metadata.file_type();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_symlink = is_symlink_like(&metadata);

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
    let metadata = fs::symlink_metadata(path)?;
    if is_symlink_like(&metadata) {
        return Err(WindError::SymlinkNotSupported.into());
    }

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
    let metadata = fs::symlink_metadata(path)?;
    if is_symlink_like(&metadata) {
        return Err(WindError::SymlinkNotSupported.into());
    }

    if path.is_dir() {
        // Check if directory is empty
        let is_empty = path.read_dir().map(|mut i| i.next().is_none()).unwrap_or(true);
        if !is_empty && (!recursive || !yes) {
            return Err(WindError::DirNotEmpty(path.display().to_string()).into());
        }
        if recursive && !yes {
            return Err(WindError::Usage(
                "recursive directory deletion requires --yes".to_string(),
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
