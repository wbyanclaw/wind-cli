//! Command dispatcher
#![allow(clippy::ptr_arg)]

use crate::cli::{Cli, Command, ToolsCommand, WftAction, WikiAction};
use crate::config::get_workspace_root;
use crate::errors::{exit_with_error, WindError};
use crate::tools;
use crate::windlocal;

pub fn run(cli: Cli) -> anyhow::Result<()> {
    let json_mode = cli.json;

    let result = match &cli.command {
        Command::Version => cmd_version(),
        Command::Init { path } => cmd_init(path),
        Command::Ls { path } => cmd_ls(path),
        Command::Read { path } => cmd_read(path),
        Command::Write { path, stdin, content, overwrite } => cmd_write(path, *stdin, content.as_ref(), *overwrite),
        Command::Mkdir { path } => cmd_mkdir(path),
        Command::Rm { path, recursive, yes, dry_run, force } => {
            cmd_rm(path, *recursive, *yes, *dry_run, *force)
        }
        Command::Open { file, search, app, settings } => {
            // Deprecated - show warning
            eprintln!("warning: 'wind open' is deprecated; use 'wind wft' instead");
            cmd_open(file.as_ref(), search.as_ref(), *app, *settings)
        }
        Command::Upgrade { check } => cmd_upgrade(*check),
        Command::Tools { subcommand } => cmd_tools(subcommand),
        Command::Extract { path, format, include_base64, tabular } => {
            cmd_extract(path, format.as_deref(), *include_base64, *tabular)
        }
        Command::Wft { action } => cmd_wft(action),
        Command::Wiki { action } => cmd_wiki(action, json_mode),
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

fn cmd_version() -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::json!({
        "ok": true,
        "version": env!("CARGO_PKG_VERSION"),
        "name": "windcli"
    }))
}

fn cmd_init(path: &std::path::PathBuf) -> anyhow::Result<serde_json::Value> {
    let root = if path == &std::path::PathBuf::from(".") {
        std::env::current_dir()?
    } else {
        path.clone()
    };

    if root.exists() && !root.is_dir() {
        return Err(WindError::InitFailed(format!(
            "workspace path is not a directory: {}",
            root.display()
        )).into());
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

pub(crate) fn cmd_ls(path: &std::path::PathBuf) -> anyhow::Result<serde_json::Value> {
    let root = get_workspace_root()?;
    let safe = crate::workspace::safe_path(&root, path)?;
    let listing = crate::workspace::ls(&safe)?;
    Ok(serde_json::json!({
        "ok": true,
        "root": root,
        "entries": listing
    }))
}

pub(crate) fn cmd_read(path: &std::path::PathBuf) -> anyhow::Result<serde_json::Value> {
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

pub(crate) fn cmd_write(
    path: &std::path::PathBuf,
    stdin: bool,
    content: Option<&String>,
    overwrite: bool,
) -> anyhow::Result<serde_json::Value> {
    let root = get_workspace_root()?;
    let safe = crate::workspace::safe_path_for_create(&root, path)?;

    // Default deny: check if file exists before writing
    if safe.exists() && !overwrite {
        return Err(WindError::FileExists("<file> already exists".to_string()).into());
    }

    let bytes = if stdin {
        use std::io::Read;
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        buf
    } else if let Some(text) = content {
        text.as_bytes().to_vec()
    } else {
        return Err(WindError::Usage(
            "write requires either --stdin or --content".to_string(),
        ).into());
    };

    crate::workspace::put(&safe, &bytes)?;
    let size_bytes = bytes.len();
    Ok(serde_json::json!({
        "ok": true,
        "message": format!("file written: {} ({} bytes)", safe.display(), size_bytes),
        "file": safe.display().to_string(),
        "size_bytes": size_bytes
    }))
}

pub(crate) fn cmd_mkdir(path: &std::path::PathBuf) -> anyhow::Result<serde_json::Value> {
    let root = get_workspace_root()?;
    let safe = crate::workspace::safe_path_for_create(&root, path)?;
    crate::workspace::mkdir(&safe)?;
    Ok(serde_json::json!({
        "ok": true,
        "message": format!("directory created: {}", safe.display()),
        "path": safe.display().to_string()
    }))
}

pub(crate) fn cmd_rm(
    path: &std::path::PathBuf,
    recursive: bool,
    yes: bool,
    dry_run: bool,
    force: bool,
) -> anyhow::Result<serde_json::Value> {
    let path_text = path.to_string_lossy();
    if path_text.contains('*') || path_text.contains('?') || path_text.contains('[') {
        return Err(WindError::GlobNotAllowed.into());
    }

    // --force is shorthand for --recursive --yes (AI Agent friendly)
    let recursive = recursive || force;
    let yes = yes || force;

    let root = get_workspace_root()?;
    let safe = crate::workspace::safe_path(&root, path)?;

    if dry_run {
        Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "message": format!("dry run — would delete: {}", safe.display()),
            "path": safe.display().to_string()
        }))
    } else {
        crate::workspace::rm(&safe, recursive, yes, dry_run)?;
        Ok(serde_json::json!({
            "ok": true,
            "message": "Deleted",
            "path": safe.display().to_string()
        }))
    }
}

fn cmd_open(
    file: Option<&std::path::PathBuf>,
    search: Option<&String>,
    app: bool,
    settings: bool,
) -> anyhow::Result<serde_json::Value> {
    // Encapsulate windlocal details — user doesn't see URI scheme
    let action = match (file, search, app, settings) {
        (Some(path), _, _, _) => {
            let target = path.to_string_lossy();
            if target.contains("..") || target.starts_with('/') {
                return Err(WindError::ActionBlocked("path traversal attempt".to_string()).into());
            }
            crate::windlocal::WindAction::Page {
                kind: crate::windlocal::PageKind::File,
                target: target.to_string(),
            }
        }
        (_, Some(query), _, _) => crate::windlocal::WindAction::Page {
            kind: crate::windlocal::PageKind::Search,
            target: query.to_string(),
        },
        (_, _, true, _) => crate::windlocal::WindAction::Command {
            id: crate::windlocal::CommandId::ShowApp,
        },
        (_, _, _, true) => crate::windlocal::WindAction::Command {
            id: crate::windlocal::CommandId::ShowSettings,
        },
        _ => {
            return Err(WindError::Usage(
                "open requires --file <path>, --search <query>, --app, or --settings".to_string(),
            ).into())
        }
    };

    crate::windlocal::validate(&action)?;

    // Execute based on action type
    let root = crate::config::get_workspace_root()?;
    let action_json = crate::windlocal::action_to_json(&action);

    match &action {
        crate::windlocal::WindAction::Page { kind: crate::windlocal::PageKind::File, target } => {
            // Open file with system default app (use file:// URI)
            let safe = crate::workspace::safe_path(&root, &std::path::PathBuf::from(target))?;
            open_file_with_default_app(&safe)?;
        }
        crate::windlocal::WindAction::Page { kind: crate::windlocal::PageKind::Search, .. } => {
            // Open windlocal search page
            let uri = build_windlocal_uri(&action)?;
            crate::platform::open_uri(&uri)?;
        }
        crate::windlocal::WindAction::Page { kind: crate::windlocal::PageKind::App, .. } => {
            // Open windlocal app page
            let uri = build_windlocal_uri(&action)?;
            crate::platform::open_uri(&uri)?;
        }
        crate::windlocal::WindAction::Page { kind: crate::windlocal::PageKind::Settings, .. } => {
            // Open windlocal settings page
            let uri = build_windlocal_uri(&action)?;
            crate::platform::open_uri(&uri)?;
        }
        crate::windlocal::WindAction::Command { .. } => {
            // Open windlocal command (app/settings)
            let uri = build_windlocal_uri(&action)?;
            crate::platform::open_uri(&uri)?;
        }
    }

    Ok(serde_json::json!({
        "ok": true,
        "message": "opened",
        "action": action_json
    }))
}

/// Build a windlocal URI from a WindAction
fn build_windlocal_uri(action: &crate::windlocal::WindAction) -> anyhow::Result<String> {
    match action {
        crate::windlocal::WindAction::Page { kind, target } => {
            let kind_str = match kind {
                crate::windlocal::PageKind::File => "file",
                crate::windlocal::PageKind::Search => "search",
                crate::windlocal::PageKind::App => "app",
                crate::windlocal::PageKind::Settings => "settings",
            };
            let encoded_target = urlencoding::encode(target);
            Ok(format!("windlocal://page?kind={}&target={}", kind_str, encoded_target))
        }
        crate::windlocal::WindAction::Command { id } => {
            let id_str = match id {
                crate::windlocal::CommandId::ShowWorkspace => "show_workspace",
                crate::windlocal::CommandId::ShowApp => "show_app",
                crate::windlocal::CommandId::ShowSettings => "show_settings",
                crate::windlocal::CommandId::CheckUpgrade => "check_upgrade",
            };
            Ok(format!("windlocal://command?id={}", id_str))
        }
    }
}

/// Open file with system default handler (does not require Wind Terminal)
fn open_file_with_default_app(path: &std::path::Path) -> anyhow::Result<()> {
    let absolute_path = path.canonicalize()
        .map_err(|e| anyhow::anyhow!("cannot resolve path: {}", e))?;

    let uri = format!("file://{}", absolute_path.to_string_lossy());
    crate::platform::open_uri(&uri)
}

fn cmd_upgrade(check: bool) -> anyhow::Result<serde_json::Value> {
    if !check {
        return Ok(serde_json::json!({
            "ok": true,
            "upgrade_supported": false,
            "message": "windcli currently supports checking for updates only. Run `windcli upgrade --check` to check the latest release.",
            "next_command": "windcli upgrade --check"
        }));
    }

    let current = env!("CARGO_PKG_VERSION");
    let repo = "wbyanclaw/wind-cli";
    let release_url = format!("https://github.com/{}/releases/latest", repo);
    let install_command = recommended_install_command();

    // Fetch latest release from GitHub API
    let latest = fetch_latest_version(repo)?;

    let update_available = compare_versions(&latest, current) > 0;

    Ok(serde_json::json!({
        "ok": true,
        "update_available": update_available,
        "current_version": current,
        "latest_version": latest,
        "release_url": release_url,
        "install_command": install_command,
        "message": if update_available {
            format!("new version {} available. Download it from {} or run the recommended install command.", latest, release_url)
        } else {
            "you are using the latest version. windcli upgrade --check only checks releases; it does not install updates automatically.".to_string()
        }
    }))
}

fn recommended_install_command() -> &'static str {
    "$p = \"$env:TEMP\\windcli-install.ps1\"; irm https://github.com/wbyanclaw/wind-cli/releases/latest/download/install.ps1 -OutFile $p; powershell -NoProfile -ExecutionPolicy Bypass -File $p -NoPause"
}

fn upgrade_network_error_message() -> String {
    format!(
        "Could not reach GitHub releases over HTTPS. `windcli upgrade --check` needs access to https://api.github.com. Check these items: 1) run `curl https://github.com` and `curl https://api.github.com`; 2) check company proxy, firewall, or VPN settings; 3) check system date/time and Windows root certificates; 4) check WinHTTP proxy or HTTPS proxy environment variables. Manual download: https://github.com/wbyanclaw/wind-cli/releases/latest. Recommended install command: {}",
        recommended_install_command()
    )
}

/// Fetch latest version from GitHub releases
fn fetch_latest_version(repo: &str) -> anyhow::Result<String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);

    let response = ureq::get(&url)
        .set("User-Agent", "wind-cli")
        .call()
        .map_err(|_| WindError::NetworkFailed(upgrade_network_error_message()))?;

    let json: serde_json::Value = serde_json::from_reader(response.into_reader())
        .map_err(|_| WindError::UpgradeResponseInvalid)?;

    // Extract tag_name from response (e.g., "v0.1.10" -> "0.1.10")
    let tag = json.get("tag_name")
        .and_then(|t| t.as_str())
        .ok_or(WindError::UpgradeResponseInvalid)?;

    // Strip leading 'v' if present
    let version = tag.strip_prefix('v').unwrap_or(tag);
    Ok(version.to_string())
}

/// Compare two semver versions: returns positive if v1 > v2
fn compare_versions(v1: &str, v2: &str) -> i32 {
    use std::cmp::Ordering;

    fn parse_ver(s: &str) -> Vec<u32> {
        s.split('.')
            .filter_map(|p| p.parse().ok())
            .collect()
    }

    let v1_parts = parse_ver(v1);
    let v2_parts = parse_ver(v2);

    for (a, b) in v1_parts.iter().zip(v2_parts.iter()) {
        match a.cmp(b) {
            Ordering::Greater => return 1,
            Ordering::Less => return -1,
            Ordering::Equal => continue,
        }
    }

    // If all compared parts are equal, longer version is greater
    match v1_parts.len().cmp(&v2_parts.len()) {
        Ordering::Greater => 1,
        Ordering::Less => -1,
        Ordering::Equal => 0,
    }
}

/// Agent Protocol tools dispatcher
fn cmd_tools(subcommand: &ToolsCommand) -> anyhow::Result<serde_json::Value> {
    tools::run_tools(subcommand.clone())
}

/// Extract content from documents (PDF, Excel, PPTX, Markdown, HTML, images)
fn cmd_extract(
    path: &std::path::PathBuf,
    format: Option<&str>,
    include_base64: bool,
    tabular: bool,
) -> anyhow::Result<serde_json::Value> {
    use crate::extract::ExtractFormat;

    // Validate workspace path
    let root = crate::config::get_workspace_root()?;
    let safe = crate::workspace::safe_path(&root, path)?;

    // Check file exists and size
    let metadata = std::fs::metadata(&safe)?;
    const SIZE_LIMIT: u64 = 10 * 1024 * 1024; // 10MB
    if metadata.len() > SIZE_LIMIT {
        Err(WindError::Usage(format!(
            "file too large: {} bytes (max 10MB)",
            metadata.len()
        )))?;
    }

    // Parse format option
    let force_format = match format {
        Some(f) => {
            let f_lower = f.to_lowercase();
            Some(match f_lower.as_str() {
                "md" | "markdown" => ExtractFormat::Md,
                "html" | "htm" => ExtractFormat::Html,
                "pdf" => ExtractFormat::Pdf,
                "xlsx" | "xls" | "excel" => ExtractFormat::Xlsx,
                "pptx" | "ppt" => ExtractFormat::Pptx,
                "img" | "png" | "jpg" | "jpeg" | "image" => ExtractFormat::Img,
                _ => anyhow::bail!("unsupported format: {}", f),
            })
        }
        None => None,
    };

    let output = crate::extract::extract(&safe, force_format, include_base64, tabular)?;
    Ok(serde_json::to_value(output)?)
}

fn cmd_wft(action: &WftAction) -> anyhow::Result<serde_json::Value> {
    let wind_action = match action {
        WftAction::File { path } => {
            let target = path.to_string_lossy();
            if target.contains("..") || target.starts_with('/') {
                return Err(WindError::ActionBlocked("path traversal attempt".to_string()).into());
            }
            windlocal::WindAction::Page {
                kind: windlocal::PageKind::File,
                target: target.to_string(),
            }
        }
        WftAction::Search { query } => windlocal::WindAction::Page {
            kind: windlocal::PageKind::Search,
            target: query.to_string(),
        },
        WftAction::App => windlocal::WindAction::Command {
            id: windlocal::CommandId::ShowApp,
        },
        WftAction::Settings => windlocal::WindAction::Command {
            id: windlocal::CommandId::ShowSettings,
        },
        WftAction::Workspace => windlocal::WindAction::Command {
            id: windlocal::CommandId::ShowWorkspace,
        },
        WftAction::Upgrade => windlocal::WindAction::Command {
            id: windlocal::CommandId::CheckUpgrade,
        },
        WftAction::Url { uri } => {
            let parsed = windlocal::parse(uri)?;
            windlocal::validate(&parsed)?;
            return Ok(serde_json::json!({
                "ok": true,
                "message": "windlocal URI processed",
                "action": windlocal::action_to_json(&parsed)
            }));
        }
    };

    windlocal::validate(&wind_action)?;
    let action_json = windlocal::action_to_json(&wind_action);
    Ok(serde_json::json!({
        "ok": true,
        "message": "windlocal action dispatched to WFT",
        "action": action_json
    }))
}

fn cmd_wiki(action: &WikiAction, _json_mode: bool) -> anyhow::Result<serde_json::Value> {
    use crate::cli::WikiAction;
    use tokio::runtime::Runtime;

    let rt = Runtime::new()?;

    match action {
        WikiAction::Ingest { file } => {
            let result = rt.block_on(async {
                let config = wind_wiki::Config::load().unwrap_or_default();
                let wiki = wind_wiki::Wiki::new(config).await?;
                wiki.ingest(file.to_string_lossy().as_ref()).await
            })?;
            Ok(serde_json::to_value(result)?)
        }
        WikiAction::Query { question } => {
            let result = rt.block_on(async {
                let config = wind_wiki::Config::load().unwrap_or_default();
                let wiki = wind_wiki::Wiki::new(config).await?;
                wiki.query(question).await
            })?;
            Ok(serde_json::to_value(result)?)
        }
        WikiAction::Lint => {
            let result = rt.block_on(async {
                let config = wind_wiki::Config::load().unwrap_or_default();
                let wiki = wind_wiki::Wiki::new(config).await?;
                wiki.lint().await
            })?;
            Ok(serde_json::to_value(result)?)
        }
        WikiAction::Status => {
            let config = wind_wiki::Config::load().unwrap_or_default();
            let wiki = wind_wiki::Wiki::new_local(config)?;
            let status = wiki.status()?;
            Ok(serde_json::to_value(status)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::upgrade_network_error_message;

    #[test]
    fn upgrade_network_error_is_actionable() {
        let message = upgrade_network_error_message();

        assert!(message.contains("curl https://github.com"));
        assert!(message.contains("curl https://api.github.com"));
        assert!(message.contains("proxy"));
        assert!(message.contains("firewall"));
        assert!(message.contains("system date/time"));
        assert!(message.contains("Windows root certificates"));
        assert!(message.contains("https://github.com/wbyanclaw/wind-cli/releases/latest"));
        assert!(message.contains("ExecutionPolicy Bypass"));
        assert!(!message.contains("P0"));
        assert!(!message.contains("ffailed"));
    }
}
