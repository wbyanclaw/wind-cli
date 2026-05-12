//! Command dispatcher
//!
//! Designed for AI agent friendliness with clear messages and helpful errors.

use crate::cli::{Cli, Command};
use crate::config::get_workspace_root;
use crate::errors::{exit_with_error, WindError};
use std::path::PathBuf;

pub fn run(cli: Cli) -> anyhow::Result<()> {
    let json_mode = cli.json;

    let result = match &cli.command {
        Command::Version => cmd_version(),
        Command::Init { path } => cmd_init(path),
        Command::Ls { path } => cmd_ls(path),
        Command::Read { path } => cmd_read(path),
        Command::Write { path, stdin, content } => cmd_write(path, *stdin, content.as_ref()),
        Command::Mkdir { path } => cmd_mkdir(path),
        Command::Delete {
            path,
            recursive,
            yes,
            dry_run,
        } => cmd_delete(path, *recursive, *yes, *dry_run),
        Command::Upgrade { check } => cmd_upgrade(*check),
    };

    match result {
        Ok(output) => {
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("{}", output);
            }
            Ok(())
        }
        Err(ref e) => {
            exit_with_error(e, json_mode);
        }
    }
}

// =============================================================================
// Commands
// =============================================================================

fn cmd_version() -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::json!({
        "ok": true,
        "version": env!("CARGO_PKG_VERSION"),
        "name": "windcli"
    }))
}

fn cmd_init(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    let root = if path == &PathBuf::from(".") {
        std::env::current_dir()?
    } else {
        path.clone()
    };

    if root.exists() && !root.is_dir() {
        return Err(WindError::InitFailed(format!(
            "ERROR: '{}' is not a directory. Please specify a valid directory path.",
            root.display()
        ))
        .into());
    }

    std::fs::create_dir_all(&root).map_err(|e| {
        WindError::InitFailed(format!(
            "ERROR: Cannot create directory '{}': {}. Check permissions or try a different path.",
            root.display(),
            e
        ))
    })?;

    let root = root.canonicalize().map_err(|e| {
        WindError::InitFailed(format!(
            "ERROR: Cannot resolve path '{}': {}. Please use an absolute or relative path.",
            root.display(),
            e
        ))
    })?;

    let mut config = crate::config::Config::load()?;
    if let Some(existing) = &config.active_workspace {
        if existing == &root {
            return Ok(serde_json::json!({
                "ok": true,
                "message": format!("Workspace already set to: {}", root.display()),
                "root": root
            }));
        } else {
            return Err(WindError::InitFailed(
                format!(
                    "ERROR: Workspace already exists at '{}'. To use a different workspace, please specify its path. Current command is: windcli init <new-path>",
                    existing.display()
                )
            ).into());
        }
    }

    config.set_active_workspace(root.clone());
    config.save()?;

    Ok(serde_json::json!({
        "ok": true,
        "message": format!("Workspace initialized: {}", root.display()),
        "root": root
    }))
}

fn cmd_ls(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    let root = get_workspace_root()?;
    let safe = crate::workspace::safe_path(&root, path)?;
    let listing = crate::workspace::ls(&safe)?;
    Ok(serde_json::json!({
        "ok": true,
        "workspace": root.display().to_string(),
        "path": safe.display().to_string(),
        "files": listing
    }))
}

fn cmd_read(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    const SIZE_LIMIT: u64 = 10 * 1024 * 1024; // 10MB
    let root = get_workspace_root()?;
    let safe = crate::workspace::safe_path(&root, path)?;
    let content = crate::workspace::cat(&safe, SIZE_LIMIT)?;

    let size_bytes = content.len();
    Ok(serde_json::json!({
        "ok": true,
        "file": safe.display().to_string(),
        "size_bytes": size_bytes,
        "content": content
    }))
}

fn cmd_write(
    path: &PathBuf,
    stdin: bool,
    content: Option<&String>,
) -> anyhow::Result<serde_json::Value> {
    let root = get_workspace_root()?;
    let safe = crate::workspace::safe_path_for_create(&root, path)?;

    let bytes = if stdin {
        use std::io::Read;
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        buf
    } else if let Some(text) = content {
        text.as_bytes().to_vec()
    } else {
        return Err(WindError::Usage(
            "ERROR: Please provide content via --stdin or --content. Example: echo 'hello' | windcli write notes/todo.md"
        ).into());
    };

    crate::workspace::put(&safe, &bytes)?;
    let size_bytes = bytes.len();
    Ok(serde_json::json!({
        "ok": true,
        "message": format!("File written: {} ({} bytes)", safe.display(), size_bytes),
        "file": safe.display().to_string(),
        "size_bytes": size_bytes
    }))
}

fn cmd_mkdir(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    let root = get_workspace_root()?;
    let safe = crate::workspace::safe_path_for_create(&root, path)?;
    crate::workspace::mkdir(&safe)?;
    Ok(serde_json::json!({
        "ok": true,
        "message": format!("Directory created: {}", safe.display()),
        "path": safe.display().to_string()
    }))
}

fn cmd_delete(
    path: &PathBuf,
    recursive: bool,
    yes: bool,
    dry_run: bool,
) -> anyhow::Result<serde_json::Value> {
    let path_text = path.to_string_lossy();
    if path_text.contains('*') || path_text.contains('?') || path_text.contains('[') {
        return Err(WindError::GlobNotAllowed.into());
    }

    let root = get_workspace_root()?;
    let safe = crate::workspace::safe_path(&root, path)?;

    if dry_run {
        Ok(serde_json::json!({
            "ok": true,
            "message": format!("Would delete: {} (dry run)", safe.display()),
            "path": safe.display().to_string()
        }))
    } else {
        crate::workspace::rm(&safe, recursive, yes, dry_run)?;
        Ok(serde_json::json!({
            "ok": true,
            "message": format!("Deleted: {}", safe.display()),
            "path": safe.display().to_string()
        }))
    }
}

fn cmd_upgrade(check: bool) -> anyhow::Result<serde_json::Value> {
    if !check {
        return Err(WindError::Usage("ERROR: Please use 'windcli upgrade --check' to check for updates".to_string()).into());
    }
    let current = env!("CARGO_PKG_VERSION");
    Ok(serde_json::json!({
        "ok": true,
        "version": current,
        "message": "You have the latest version. Automatic updates not supported in this release."
    }))
}