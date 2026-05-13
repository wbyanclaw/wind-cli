# wind-cli Agent Protocol v0.2 Implementation

**Author**: @首席开发-效率猿
**Last Updated**: 2026-05-13

## Phase 1: CLI Interface Design

### Commands

| Command | Purpose |
|---------|---------|
| `wind tools --list` | 工具清单（简化信息） |
| `wind tools describe <name>` | 单工具详情（含完整 Schema） |
| `wind tools call <name> --params <json> [--force]` | 调用工具 |
| `wind tools --help` | 帮助信息 |

### Parameters

- `--params`: JSON string format, e.g., `--params '{"path":"./file.txt"}'`
- `--force`: Optional flag for High risk operations

### RiskLevel Enum

```rust
pub enum RiskLevel {
    None,   // 无风险：ls, describe, --help
    Low,    // 低风险，可恢复：cat, open
    Medium, // 中风险，有副作用：put 新建
    High,   // 高风险，不可逆：rm, put 覆盖
}
```

## Implementation Details

### tools.rs Module

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub risk_level: RiskLevel,
    pub params: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub properties: HashMap<String, ParamSchema>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSchema {
    pub param_type: String,  // "string", "boolean", "number"
    pub description: String,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

// RiskLevel enforcement
fn enforce_risk_level(tool: &Tool, force: bool) -> Result<()> {
    match tool.risk_level {
        RiskLevel::High if !force => {
            eprintln!("ERROR: HIGH_RISK_OPERATION");
            eprintln!("Hint: Add --force to execute.");
            std::process::exit(1);
        }
        _ => Ok(()),
    }
}
```

### CLI Subcommands

```rust
// tools --list
fn list_tools() -> Result<String> {
    let tools = registry::get_all_tools();
    Ok(serde_json::to_string_pretty(&tools)?)
}

// tools describe <name>
fn describe_tool(name: &str) -> Result<String> {
    let tool = registry::get_tool(name)?;
    Ok(serde_json::to_string_pretty(&tool)?)
}

// tools call <name> --params <json> --force?
fn call_tool(name: &str, params: Value, force: bool) -> Result<()> {
    let tool = registry::get_tool(name)?;

    // Schema validation
    validate_params(&tool.params, &params)?;

    // Risk level enforcement
    enforce_risk_level(&tool, force)?;

    // Audit log
    log::info!(
        target: "audit",
        "TOOL_CALL: tool={}, params={}, force={}, ts={}",
        name, params, force, timestamp()
    );

    // Execute
    tool.execute(params)
}
```

### Audit Log Format

```rust
// Each --force call logs:
// TOOL_CALL: user=<system>, tool=<name>, params=<json>, force=true, ts=<timestamp>
// Stored in: ~/.wind-cli/logs/audit.log
```

## Responsibilities

| Layer | Responsibility |
|-------|----------------|
| CLI | risk_level annotation, schema validation, --force gate, audit log |
| Client/Agent | Call decision, --force passing, system prompt constraints |

## Status

- [x] Phase 1 interface design (4 commands) - **ALIGNED**
- [x] RiskLevel enum (None/Low/Medium/High) - **ALIGNED**
- [x] --force mechanism - **ALIGNED**
- [ ] Implementation code - PENDING Kevin authorization
- [ ] Architecture review - @首席架构-架构狮 (unresponsive)

## Open Points

- stdio stability (process lifecycle)
- Multi-agent concurrency (workspace file competition)
- These require @首席架构-架构狮 review