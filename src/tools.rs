//! Agent Protocol: tools registry and execution
//!
//! Phase 1 implementation:
//! - wind tools --list: 列出所有工具（简化信息）
//! - wind tools describe <name>: 查看单工具详情（含完整 Schema）
//! - wind tools call <name> --params <json>: 调用工具（含 --force 门控）

use crate::cli::ToolsCommand;
use crate::errors::{exit_with_error, WindError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 风险等级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
}

/// 参数 Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSchema {
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

/// 工具 Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    #[serde(rename = "risk_level")]
    pub risk_level: RiskLevel,
    pub params: ToolParams,
}

/// 工具参数 Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    #[serde(rename = "properties")]
    pub properties: HashMap<String, ParamSchema>,
    #[serde(default)]
    pub required: Vec<String>,
}

/// 工具元数据（用于 --list 简化输出）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMeta {
    pub name: String,
    pub description: String,
    #[serde(rename = "risk_level")]
    pub risk_level: RiskLevel,
}

/// 工具注册表
pub struct ToolRegistry;

impl ToolRegistry {
    /// 获取所有工具的完整 Schema
    pub fn all_schemas() -> Vec<ToolSchema> {
        vec![
            // None risk: information queries
            Self::ls_schema(),
            Self::describe_schema(),
            Self::help_schema(),
            Self::version_schema(),
            // Low risk: read operations
            Self::read_schema(),
            // Medium risk: write operations (new files)
            Self::write_schema(),
            Self::mkdir_schema(),
            // High risk: destructive operations
            Self::rm_schema(),
        ]
    }

    /// 获取所有工具的简化元数据（用于 --list）
    pub fn all_meta() -> Vec<ToolMeta> {
        Self::all_schemas()
            .into_iter()
            .map(|s| ToolMeta {
                name: s.name,
                description: s.description,
                risk_level: s.risk_level,
            })
            .collect()
    }

    /// 获取单工具 Schema（用于 describe）
    pub fn get_schema(name: &str) -> Option<ToolSchema> {
        Self::all_schemas().into_iter().find(|s| s.name == name)
    }

    // === Tool Schema Definitions ===

    fn ls_schema() -> ToolSchema {
        use self::RiskLevel::*;
        ToolSchema {
            name: "ls".to_string(),
            description: "列出 workspace 目录内容".to_string(),
            risk_level: None,
            params: ToolParams {
                properties: {
                    let mut m = HashMap::new();
                    m.insert(
                        "path".to_string(),
                        ParamSchema {
                            param_type: "string".to_string(),
                            description: "目录路径，默认 '.'".to_string(),
                            required: false,
                            default: Some(serde_json::json!(".")),
                        },
                    );
                    m
                },
                required: vec![],
            },
        }
    }

    fn describe_schema() -> ToolSchema {
        use self::RiskLevel::*;
        ToolSchema {
            name: "describe".to_string(),
            description: "查看工具详情".to_string(),
            risk_level: None,
            params: ToolParams {
                properties: {
                    let mut m = HashMap::new();
                    m.insert(
                        "name".to_string(),
                        ParamSchema {
                            param_type: "string".to_string(),
                            description: "工具名称".to_string(),
                            required: true,
                            default: None,
                        },
                    );
                    m
                },
                required: vec!["name".to_string()],
            },
        }
    }

    fn help_schema() -> ToolSchema {
        use self::RiskLevel::*;
        ToolSchema {
            name: "help".to_string(),
            description: "显示帮助信息".to_string(),
            risk_level: None,
            params: ToolParams {
                properties: HashMap::new(),
                required: vec![],
            },
        }
    }

    fn version_schema() -> ToolSchema {
        use self::RiskLevel::*;
        ToolSchema {
            name: "version".to_string(),
            description: "输出版本信息".to_string(),
            risk_level: None,
            params: ToolParams {
                properties: HashMap::new(),
                required: vec![],
            },
        }
    }

    fn read_schema() -> ToolSchema {
        use self::RiskLevel::*;
        ToolSchema {
            name: "read".to_string(),
            description: "读取文件内容".to_string(),
            risk_level: Low,
            params: ToolParams {
                properties: {
                    let mut m = HashMap::new();
                    m.insert(
                        "path".to_string(),
                        ParamSchema {
                            param_type: "string".to_string(),
                            description: "文件路径".to_string(),
                            required: true,
                            default: None,
                        },
                    );
                    m
                },
                required: vec!["path".to_string()],
            },
        }
    }

    fn write_schema() -> ToolSchema {
        use self::RiskLevel::*;
        ToolSchema {
            name: "write".to_string(),
            description: "写入文件内容（新建或覆盖）".to_string(),
            risk_level: Medium,
            params: ToolParams {
                properties: {
                    let mut m = HashMap::new();
                    m.insert(
                        "path".to_string(),
                        ParamSchema {
                            param_type: "string".to_string(),
                            description: "目标路径".to_string(),
                            required: true,
                            default: None,
                        },
                    );
                    m.insert(
                        "content".to_string(),
                        ParamSchema {
                            param_type: "string".to_string(),
                            description: "文件内容".to_string(),
                            required: true,
                            default: None,
                        },
                    );
                    m.insert(
                        "overwrite".to_string(),
                        ParamSchema {
                            param_type: "boolean".to_string(),
                            description: "覆盖已存在文件（高危，需 --force）".to_string(),
                            required: false,
                            default: Some(serde_json::json!(false)),
                        },
                    );
                    m
                },
                required: vec!["path".to_string(), "content".to_string()],
            },
        }
    }

    fn mkdir_schema() -> ToolSchema {
        use self::RiskLevel::*;
        ToolSchema {
            name: "mkdir".to_string(),
            description: "创建目录".to_string(),
            risk_level: Medium,
            params: ToolParams {
                properties: {
                    let mut m = HashMap::new();
                    m.insert(
                        "path".to_string(),
                        ParamSchema {
                            param_type: "string".to_string(),
                            description: "目录路径".to_string(),
                            required: true,
                            default: None,
                        },
                    );
                    m
                },
                required: vec!["path".to_string()],
            },
        }
    }

    fn rm_schema() -> ToolSchema {
        use self::RiskLevel::*;
        ToolSchema {
            name: "rm".to_string(),
            description: "删除文件或目录".to_string(),
            risk_level: High,
            params: ToolParams {
                properties: {
                    let mut m = HashMap::new();
                    m.insert(
                        "path".to_string(),
                        ParamSchema {
                            param_type: "string".to_string(),
                            description: "目标路径".to_string(),
                            required: true,
                            default: None,
                        },
                    );
                    m.insert(
                        "recursive".to_string(),
                        ParamSchema {
                            param_type: "boolean".to_string(),
                            description: "递归删除目录".to_string(),
                            required: false,
                            default: Some(serde_json::json!(false)),
                        },
                    );
                    m
                },
                required: vec!["path".to_string()],
            },
        }
    }
}

/// 执行 tools 子命令
pub fn run_tools(cmd: ToolsCommand) -> anyhow::Result<serde_json::Value> {
    match cmd {
        ToolsCommand::List => cmd_tools_list(),
        ToolsCommand::Describe { name } => cmd_tools_describe(&name),
        ToolsCommand::Call { name, params, force } => cmd_tools_call(&name, params.as_deref(), force),
    }
}

/// wind tools --list
fn cmd_tools_list() -> anyhow::Result<serde_json::Value> {
    let tools = ToolRegistry::all_meta();
    Ok(serde_json::json!({
        "ok": true,
        "tools": tools
    }))
}

/// wind tools describe <name>
fn cmd_tools_describe(name: &str) -> anyhow::Result<serde_json::Value> {
    let schema = ToolRegistry::get_schema(name).ok_or_else(|| {
        WindError::Usage(format!("tool not found: {}", name))
    })?;

    Ok(serde_json::json!({
        "ok": true,
        "tool": schema
    }))
}

/// wind tools call <name> --params <json> --force
fn cmd_tools_call(name: &str, params_json: Option<&str>, force: bool) -> anyhow::Result<serde_json::Value> {
    let schema = ToolRegistry::get_schema(name).ok_or_else(|| {
        WindError::Usage(format!("tool not found: {}", name))
    })?;

    // 解析参数
    let params: HashMap<String, serde_json::Value> = if let Some(json_str) = params_json {
        serde_json::from_str(json_str).map_err(|e| {
            WindError::Usage(format!("invalid JSON params: {}", e))
        })?
    } else {
        HashMap::new()
    };

    // Schema 校验
    validate_params(&schema, &params)?;

    // --force 门控：高危操作必须显式授权
    // High risk: rm（直接 High）
    // write with overwrite: true（动态升级为 High）
    let is_high_risk = match (schema.name.as_str(), params.get("overwrite")) {
        ("write", Some(serde_json::Value::Bool(true))) => true,
        _ => schema.risk_level == RiskLevel::High,
    };

    if is_high_risk && !force {
        return Err(WindError::HighRiskRequiresForce(name.to_string()).into());
    }

    // Bug 2 修复：FILE_EXISTS 检查
    if name == "write" {
        let root = crate::config::get_workspace_root()?;
        let path = params.get("path")
            .and_then(|v| v.as_str())
            .map(|p| std::path::PathBuf::from(p))
            .unwrap_or_default();
        let safe = crate::workspace::safe_path(&root, &path)?;

        if safe.exists() {
            // overwrite=false + 文件存在 → 返回 FILE_EXISTS
            let overwrite = params.get("overwrite")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !overwrite {
                return Err(WindError::FileExists(format!("file already exists")).into());
            }
        }
    }

    // 审计日志
    log_audit(name, &params, force);

    // Bug 1 修复：实际执行工具
    match name {
        "ls" => {
            let path = params.get("path")
                .and_then(|v| v.as_str())
                .map(|p| std::path::PathBuf::from(p))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            crate::app::cmd_ls(&path)
        }
        "read" => {
            let path = params.get("path")
                .ok_or_else(|| WindError::Usage("missing required parameter: path".to_string()))?;
            let path = path.as_str()
                .ok_or_else(|| WindError::Usage("path must be a string".to_string()))?;
            crate::app::cmd_read(&std::path::PathBuf::from(path))
        }
        "write" => {
            let path = params.get("path")
                .ok_or_else(|| WindError::Usage("missing required parameter: path".to_string()))?;
            let content = params.get("content")
                .ok_or_else(|| WindError::Usage("missing required parameter: content".to_string()))?;
            let path = path.as_str()
                .ok_or_else(|| WindError::Usage("path must be a string".to_string()))?;
            let content = content.as_str()
                .ok_or_else(|| WindError::Usage("content must be a string".to_string()))?;
            crate::app::cmd_write(
                &std::path::PathBuf::from(path),
                false,
                Some(&content.to_string()),
            )
        }
        "mkdir" => {
            let path = params.get("path")
                .ok_or_else(|| WindError::Usage("missing required parameter: path".to_string()))?;
            let path = path.as_str()
                .ok_or_else(|| WindError::Usage("path must be a string".to_string()))?;
            crate::app::cmd_mkdir(&std::path::PathBuf::from(path))
        }
        "rm" => {
            let path = params.get("path")
                .ok_or_else(|| WindError::Usage("missing required parameter: path".to_string()))?;
            let path = path.as_str()
                .ok_or_else(|| WindError::Usage("path must be a string".to_string()))?;
            let recursive = params.get("recursive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            // rm 本身是 High risk，所以 --force 已经在上面检查过
            // 这里用 recursive=true, yes=true, dry_run=false, force=true 简化
            crate::app::cmd_rm(
                &std::path::PathBuf::from(path),
                recursive,
                true,  // yes=true（--force 已授权）
                false, // dry_run=false
                true,  // force=true（--force 已授权）
            )
        }
        _ => Err(WindError::Usage(format!("tool not implemented: {}", name)).into()),
    }
}

/// 校验参数是否符合 Schema
fn validate_params(
    schema: &ToolSchema,
    params: &HashMap<String, serde_json::Value>,
) -> anyhow::Result<()> {
    // 检查必填参数
    for required in &schema.params.required {
        if !params.contains_key(required) {
            return Err(WindError::Usage(format!(
                "missing required parameter: {}",
                required
            )).into());
        }
    }

    // 检查参数类型
    for (key, value) in params {
        if let Some(param_schema) = schema.params.properties.get(key) {
            let expected_type = &param_schema.param_type;
            let actual_type = match value {
                serde_json::Value::Null => "null",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::String(_) => "string",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
            };

            if expected_type != actual_type {
                return Err(WindError::Usage(format!(
                    "parameter type mismatch for '{}': expected {}, got {}",
                    key, expected_type, actual_type
                )).into());
            }
        }
    }

    Ok(())
}

/// 审计日志
fn log_audit(tool: &str, params: &HashMap<String, serde_json::Value>, force: bool) {
    if force {
        // 仅记录 --force 调用
        log::info!(
            target: "audit",
            "TOOL_CALL: tool={}, params={}, force={}, ts={}",
            tool,
            serde_json::to_string(params).unwrap_or_default(),
            force,
            chrono::Local::now().to_rfc3339()
        );
    }
}
