//! Agent Protocol tools for AI Agent integration
//!
//! This module provides a JSON-based interface for AI agents to interact
//! with windcli commands through the `tools` subcommand.

use crate::config::get_workspace_root;
use crate::errors::WindError;
use crate::workspace;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// =============================================================================
// Tool definitions
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ToolResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ToolError>,
}

#[derive(Debug, Serialize)]
pub struct ToolError {
    pub code: String,
    pub message: String,
}

// =============================================================================
// Tool handlers
// =============================================================================

pub fn call_tool(call: &ToolCall) -> anyhow::Result<serde_json::Value> {
    match call.name.as_str() {
        // File operations
        "ls" => tool_ls(&call.arguments),
        "read" | "cat" => tool_read(&call.arguments),
        "write" | "put" => tool_write(&call.arguments),
        "mkdir" => tool_mkdir(&call.arguments),
        "rm" => tool_rm(&call.arguments),

        // Workspace tools
        "describe" | "workspace_info" => tool_workspace_info(),
        "version" | "version_check" => tool_version(),

        // Utility
        "help" => tool_help(&call.arguments),

        _ => Err(WindError::Usage(format!("unknown tool: {}", call.name)).into()),
    }
}

// D-2: write risk escalation fix
// overwrite=true + new file = no --force needed
// overwrite=true + existing file + --force = write
fn tool_write(args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct WriteArgs {
        path: String,
        content: Option<String>,
        #[serde(rename = "overwrite")]
        do_overwrite: Option<bool>,
        #[serde(rename = "force")]
        force: Option<bool>,
    }

    let args: WriteArgs = serde_json::from_value(args.clone())
        .map_err(|e| WindError::Usage(format!("invalid write arguments: {}", e)))?;

    let root = get_workspace_root()?;
    let path = PathBuf::from(&args.path);
    let safe = workspace::safe_path_for_create(&root, &path)?;

    // D-2: Check if file exists
    let file_exists = safe.exists();

    // Risk assessment
    let do_overwrite = args.do_overwrite.unwrap_or(false);
    let force = args.force.unwrap_or(false);

    if file_exists {
        // D-2: Existing file requires overwrite=true AND force=true
        if !do_overwrite {
            return Ok(serde_json::json!({
                "ok": false,
                "error": {
                    "code": "FILE_EXISTS",
                    "message": format!("file '{}' already exists; use overwrite=true to overwrite", args.path),
                    "exitCode": 3
                }
            }));
        }
        if !force {
            return Ok(serde_json::json!({
                "ok": false,
                "error": {
                    "code": "HIGH_RISK_REQUIRED_FORCE",
                    "message": format!("file '{}' already exists; use force=true to confirm overwrite", args.path),
                    "exitCode": 3
                }
            }));
        }
    }
    // D-2: New file with overwrite=true = allowed, no force needed

    let content = args.content.unwrap_or_default();
    let bytes = content.as_bytes();

    workspace::put(&safe, bytes)?;

    Ok(serde_json::json!({
        "ok": true,
        "result": {
            "path": safe.to_string_lossy(),
            "bytes_written": bytes.len()
        }
    }))
}

fn tool_ls(args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct LsArgs {
        #[serde(rename = "path")]
        #[serde(default = "default_path")]
        path: String,
    }

    fn default_path() -> String {
        ".".to_string()
    }

    let args: LsArgs = serde_json::from_value(args.clone())
        .map_err(|e| WindError::Usage(format!("invalid ls arguments: {}", e)))?;

    let root = get_workspace_root()?;
    let path = PathBuf::from(&args.path);
    let safe = workspace::safe_path(&root, &path)?;
    let entries = workspace::ls(&safe)?;

    Ok(serde_json::json!({
        "ok": true,
        "result": {
            "root": root.to_string_lossy(),
            "entries": entries
        }
    }))
}

fn tool_read(args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct ReadArgs {
        path: String,
    }

    let args: ReadArgs = serde_json::from_value(args.clone())
        .map_err(|e| WindError::Usage(format!("invalid read arguments: {}", e)))?;

    const CAT_SIZE_LIMIT: u64 = 10 * 1024 * 1024; // 10MB
    let root = get_workspace_root()?;
    let path = PathBuf::from(&args.path);
    let safe = workspace::safe_path(&root, &path)?;
    let content = workspace::cat(&safe, CAT_SIZE_LIMIT)?;

    Ok(serde_json::json!({
        "ok": true,
        "result": {
            "content": content
        }
    }))
}

fn tool_mkdir(args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct MkdirArgs {
        path: String,
    }

    let args: MkdirArgs = serde_json::from_value(args.clone())
        .map_err(|e| WindError::Usage(format!("invalid mkdir arguments: {}", e)))?;

    let root = get_workspace_root()?;
    let path = PathBuf::from(&args.path);
    let safe = workspace::safe_path_for_create(&root, &path)?;
    workspace::mkdir(&safe)?;

    Ok(serde_json::json!({
        "ok": true,
        "result": {
            "path": safe.to_string_lossy()
        }
    }))
}

fn tool_rm(args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    #[derive(Deserialize)]
    struct RmArgs {
        path: String,
        #[serde(rename = "recursive")]
        #[serde(default)]
        recursive: bool,
        #[serde(rename = "yes")]
        #[serde(default)]
        yes: bool,
        #[serde(rename = "dryRun")]
        #[serde(default)]
        dry_run: bool,
    }

    let args: RmArgs = serde_json::from_value(args.clone())
        .map_err(|e| WindError::Usage(format!("invalid rm arguments: {}", e)))?;

    let root = get_workspace_root()?;
    let path = PathBuf::from(&args.path);
    let safe = workspace::safe_path(&root, &path)?;
    workspace::rm(&safe, args.recursive, args.yes, args.dry_run)?;

    Ok(serde_json::json!({
        "ok": true,
        "result": {
            "message": if args.dry_run { "would delete" } else { "deleted" },
            "path": safe.to_string_lossy()
        }
    }))
}

fn tool_workspace_info() -> anyhow::Result<serde_json::Value> {
    let root = get_workspace_root()?;

    Ok(serde_json::json!({
        "ok": true,
        "result": {
            "workspace_root": root.to_string_lossy(),
            "version": env!("CARGO_PKG_VERSION")
        }
    }))
}

fn tool_version() -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::json!({
        "ok": true,
        "result": {
            "version": env!("CARGO_PKG_VERSION"),
            "name": "windcli"
        }
    }))
}

fn tool_help(args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let tool_name = args.get("tool").and_then(|v| v.as_str());

    let tools = serde_json::json!([
        {
            "name": "ls",
            "description": "List directory contents",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path (default: .)"
                    }
                }
            }
        },
        {
            "name": "read",
            "description": "Read file content (max 10MB)",
            "input_schema": {
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path"
                    }
                }
            }
        },
        {
            "name": "write",
            "description": "Write file content",
            "input_schema": {
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path"
                    },
                    "content": {
                        "type": "string",
                        "description": "File content"
                    },
                    "overwrite": {
                        "type": "boolean",
                        "description": "Allow overwriting existing files"
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Confirm overwrite for existing files"
                    }
                }
            }
        },
        {
            "name": "mkdir",
            "description": "Create directory",
            "input_schema": {
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path"
                    }
                }
            }
        },
        {
            "name": "rm",
            "description": "Delete file or directory",
            "input_schema": {
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File or directory path"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Delete non-empty directories"
                    },
                    "yes": {
                        "type": "boolean",
                        "description": "Confirm deletion"
                    },
                    "dryRun": {
                        "type": "boolean",
                        "description": "Preview without deleting"
                    }
                }
            }
        },
        {
            "name": "workspace_info",
            "description": "Get current workspace information",
            "input_schema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "version",
            "description": "Get windcli version",
            "input_schema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "help",
            "description": "Show available tools",
            "input_schema": {
                "type": "object",
                "properties": {
                    "tool": {
                        "type": "string",
                        "description": "Tool name for specific help"
                    }
                }
            }
        }
    ]);

    if let Some(name) = tool_name {
        if let Some(tool) = tools.as_array().and_then(|arr| {
            arr.iter().find(|t| t.get("name").and_then(|n| n.as_str()) == Some(name))
        }) {
            Ok(serde_json::json!({
                "ok": true,
                "result": tool
            }))
        } else {
            Ok(serde_json::json!({
                "ok": false,
                "error": {
                    "code": "TOOL_NOT_FOUND",
                    "message": format!("tool '{}' not found", name)
                }
            }))
        }
    } else {
        Ok(serde_json::json!({
            "ok": true,
            "result": {
                "tools": tools
            }
        }))
    }
}

// =============================================================================
// CLI integration
// =============================================================================

pub fn handle_tools_call(call: &ToolCall) -> ToolResult {
    match call_tool(call) {
        Ok(result) => ToolResult {
            ok: true,
            result: Some(result),
            error: None,
        },
        Err(e) => {
            let code = e
                .downcast_ref::<WindError>()
                .map(|we| we.code().to_string())
                .unwrap_or_else(|| "GENERAL_ERROR".to_string());

            let message = e.to_string();

            ToolResult {
                ok: false,
                result: None,
                error: Some(ToolError { code, message }),
            }
        }
    }
}
