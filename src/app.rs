//! Command dispatcher

use crate::cli::{Cli, Command};
use crate::config::get_workspace_root;
use crate::errors::{exit_with_error, WindError};
use crate::{windlocal, workspace};
use std::path::Path;
use std::path::PathBuf;

pub fn run(cli: Cli) -> anyhow::Result<()> {
    let json_mode = cli.json;

    let result = match &cli.command {
        Command::Version => cmd_version(),
        Command::Init { path } => cmd_init(path),
        Command::Ls { path } => cmd_ls(path),
        Command::Cat { path } => cmd_cat(path),
        Command::Put { path, stdin, file } => cmd_put(path, *stdin, file.as_ref()),
        Command::Mkdir { path } => cmd_mkdir(path),
        Command::Rm {
            path,
            recursive,
            yes,
            dry_run,
        } => cmd_rm(path, *recursive, *yes, *dry_run),
        Command::Open { op, arg } => cmd_open(op, arg.as_deref()),
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
        "name": "wind"
    }))
}

fn cmd_init(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    // P0: single active workspace
    // If path is same as existing active workspace, idempotent
    // If path is different, fail with clear message (P0: no --switch)
    let root = if path == &PathBuf::from(".") {
        std::env::current_dir()?
    } else {
        path.clone()
    };

    if root.exists() && !root.is_dir() {
        return Err(WindError::InitFailed(format!(
            "workspace path is not a directory: {}",
            root.display()
        ))
        .into());
    }

    std::fs::create_dir_all(&root).map_err(|e| {
        WindError::InitFailed(format!(
            "unable to create workspace directory '{}': {}",
            root.display(),
            e
        ))
    })?;

    let root = root.canonicalize().map_err(|e| {
        WindError::InitFailed(format!(
            "unable to resolve workspace directory '{}': {}",
            root.display(),
            e
        ))
    })?;

    let mut config = crate::config::Config::load()?;
    if let Some(existing) = &config.active_workspace {
        if existing == &root {
            // Idempotent
            return Ok(serde_json::json!({
                "ok": true,
                "message": "workspace already initialized",
                "root": root
            }));
        } else {
            return Err(WindError::InitFailed(
                format!(
                    "workspace already initialized at '{}'; run 'wind init' from that directory or change active workspace (P0 does not support multi-workspace switch)",
                    existing.display()
                )
            ).into());
        }
    }

    config.set_active_workspace(root.clone());
    config.save()?;

    Ok(serde_json::json!({
        "ok": true,
        "message": "workspace initialized",
        "root": root
    }))
}

fn cmd_ls(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    let root = get_workspace_root()?;
    let safe = workspace::safe_path(&root, path)?;
    let listing = workspace::ls(&safe)?;
    Ok(serde_json::json!({
        "ok": true,
        "root": root,
        "entries": listing
    }))
}

fn cmd_cat(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    const CAT_SIZE_LIMIT: u64 = 10 * 1024 * 1024; // 10MB
    let root = get_workspace_root()?;
    let safe = workspace::safe_path(&root, path)?;
    let content = workspace::cat(&safe, CAT_SIZE_LIMIT)?;
    Ok(serde_json::json!({
        "ok": true,
        "content": content
    }))
}

fn cmd_put(
    path: &PathBuf,
    stdin: bool,
    file: Option<&PathBuf>,
) -> anyhow::Result<serde_json::Value> {
    let root = get_workspace_root()?;
    let safe = workspace::safe_path_for_create(&root, path)?;

    let content = if stdin {
        // Read from stdin until EOF
        use std::io::Read;
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        buf
    } else if let Some(src) = file {
        std::fs::read(src).map_err(|e| WindError::PermissionDenied(e.to_string()))?
    } else {
        return Err(WindError::Usage("put requires either --stdin or --file".to_string()).into());
    };

    workspace::put(&safe, &content)?;
    Ok(serde_json::json!({
        "ok": true,
        "message": "file written",
        "path": safe
    }))
}

fn cmd_mkdir(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    let root = get_workspace_root()?;
    let safe = workspace::safe_path_for_create(&root, path)?;
    workspace::mkdir(&safe)?;
    Ok(serde_json::json!({
        "ok": true,
        "message": "directory created",
        "path": safe
    }))
}

fn cmd_rm(
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
    let safe = workspace::safe_path(&root, path)?;

    workspace::rm(&safe, recursive, yes, dry_run)?;

    if dry_run {
        Ok(serde_json::json!({
            "ok": true,
            "message": "dry run — would delete",
            "path": safe
        }))
    } else {
        Ok(serde_json::json!({
            "ok": true,
            "message": "deleted",
            "path": safe
        }))
    }
}

fn cmd_open(op: &str, arg: Option<&str>) -> anyhow::Result<serde_json::Value> {
    // P0: only parse/validate, do not execute external actions
    // Encapsulated interface — no raw windlocal:// URI exposed to user
    let uri = match (op, arg) {
        ("file", Some(target)) => {
            format!("windlocal://page?kind=file&target={}", urlencoding::encode(target))
        }
        ("search", Some(query)) => {
            format!("windlocal://page?kind=search&target={}", urlencoding::encode(query))
        }
        ("app", Some(name)) => {
            format!("windlocal://page?kind=app&target={}", urlencoding::encode(name))
        }
        ("settings", _) => "windlocal://page?kind=settings".to_string(),
        ("show-workspace", _) => "windlocal://command?id=show_workspace".to_string(),
        ("show-settings", _) => "windlocal://command?id=show_settings".to_string(),
        ("check-upgrade", _) => "windlocal://command?id=check_upgrade".to_string(),
        _ => {
            return Err(WindError::Usage(format!(
                "unknown wind open operation '{}'; valid: file <path>, search <query>, app <name>, settings, show-workspace, show-settings, check-upgrade",
                op
            ))
            .into());
        }
    };

    let action = windlocal::parse(&uri)?;
    windlocal::validate(&action)?;
    if let windlocal::WindAction::Page { target, .. } = &action {
        let root = get_workspace_root()?;
        let _ = workspace::safe_path(&root, Path::new(target))?;
    }

    let action_json = windlocal::action_to_json(&action);
    Ok(serde_json::json!({
        "ok": true,
        "message": "validated",
        "action": action_json
    }))
}

fn cmd_upgrade(check: bool) -> anyhow::Result<serde_json::Value> {
    if !check {
        return Err(WindError::Usage("upgrade P0 only supports --check".to_string()).into());
    }
    // P0: only check version, no actual binary replacement
    let current = env!("CARGO_PKG_VERSION");
    Ok(serde_json::json!({
        "ok": true,
        "current_version": current,
        "message": "upgrade check: this version can report available updates; automatic self-update is not available in this release"
    }))
}
