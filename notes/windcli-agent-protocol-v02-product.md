# wind-cli v0.2 Agent Protocol — 产品验收标准

> 状态：v0.2 Phase 1 实现完成，待 Kevin 最终验收
> Owner: @首席产品-需求君

---

## 1. Phase 1 验收目标

Phase 1 最小集：CLI 提供 `tools` 子命令，作为 AI Agent 的标准化工具调用接口。

---

## 2. CLI 接口验收

### 2.1 `wind tools --list`

**成功场景**：
- 无参数调用，返回 JSON 数组
- 每个工具包含 `name` 和 `risk_level`，不包含参数 Schema

**验收用例**：

| 输入 | 期望输出 |
|------|---------|
| `wind tools --list` | JSON 数组，所有工具的 name + risk_level |
| `wind tools --list --json` | 纯 JSON，无人类可读前缀 |

**边界场景**：
- workspace 未初始化 → 返回错误，提示先运行 `wind init`
- 无任何工具注册 → 返回空数组 `[]`，不报错

**JSON 格式**：
```json
[
  {"name": "file_ls", "description": "列出目录内容", "risk_level": "None"},
  {"name": "file_cat", "description": "读取文件内容", "risk_level": "Low"},
  {"name": "file_mkdir", "description": "创建目录", "risk_level": "Medium"},
  {"name": "file_rm", "description": "删除文件", "risk_level": "High"}
]
```

---

### 2.2 `wind tools describe <name>`

**成功场景**：
- 指定合法工具名，返回完整 Schema JSON

**验收用例**：

| 输入 | 期望输出 |
|------|---------|
| `wind tools describe file_rm` | 完整 schema，含 params 定义 |
| `wind tools describe nonexistent` | 错误码 `TOOL_NOT_FOUND`，exit code 2 |

**JSON 格式**：
```json
{
  "name": "file_rm",
  "description": "删除文件或目录",
  "risk_level": "High",
  "params": {
    "type": "object",
    "properties": {
      "path": {
        "type": "string",
        "description": "workspace 相对路径",
        "required": true
      },
      "recursive": {
        "type": "boolean",
        "description": "递归删除目录",
        "default": false
      },
      "force": {
        "type": "boolean",
        "description": "跳过确认，直接执行",
        "default": false
      }
    },
    "required": ["path"]
  }
}
```

---

### 2.3 `wind tools call <name> --params <json> [--force]`

**成功场景**：
- params 为合法 JSON，工具存在，参数符合 schema → 执行成功

**验收用例**：

| 输入 | 期望输出 |
|------|---------|
| `wind tools call file_ls --params '{}'` | 执行成功，返回 ls 结果 |
| `wind tools call file_rm --params '{"path":"notes/t.txt"}'` | High 级别无 --force → 拒绝执行，返回错误 |
| `wind tools call file_rm --params '{"path":"notes/t.txt"}' --force` | 执行成功，返回删除结果 |
| `wind tools call nonexistent --params '{}'` | 错误码 `TOOL_NOT_FOUND`，exit code 2 |

**High 级别拒绝行为**：
```
$ wind tools call file_rm --params '{"path":"notes/t.txt"}'
ERROR: HIGH_RISK_OPERATION
Hint: This operation requires explicit --force confirmation.
       Add --force to execute.
exit code: 1
```

---

### 2.4 `wind tools --help`

**成功场景**：
- 无需参数，返回人类可读帮助信息

**验收用例**：

| 输入 | 期望输出 |
|------|---------|
| `wind tools --help` | 显示四子命令用法说明 |
| `wind tools --help --json` | 返回纯 JSON 格式帮助 |

---

## 3. 工具注册清单（Phase 1）

| 工具名 | 描述 | risk_level | 备注 |
|--------|------|------------|------|
| `file_ls` | 列出目录 | None | 无任何风险 |
| `file_cat` | 读取文件 | Low | 可恢复读取 |
| `file_mkdir` | 创建目录 | Medium | 有副作用 |
| `file_put` | 写入文件 | Medium/High | 新建=Medium，覆盖=High；overwrite 参数区分 |
| `file_rm` | 删除文件 | High | 不可逆 |
| `workspace_info` | 获取 workspace 信息 | None | 仅信息查询 |
| `version_check` | 检查 CLI 版本 | None | 无风险 |

---

## 4. 用户场景（AI Agent 典型调用）

### 场景 A：Agent 不知道有哪些工具
```
Agent → wind tools --list
→ 根据返回的 risk_level 过滤出可用操作
→ 选择低风险工具先执行
```

### 场景 B：Agent 准备调用高危工具
```
Agent → wind tools describe file_rm
→ 看到 risk_level: High，params 需要 --force
→ Agent 决策：是否传 --force（受 system prompt 约束）
→ wind tools call file_rm --params '...' --force
```

### 场景 C：Agent 遇到未知工具
```
Agent → wind tools describe unknown_tool
→ 收到 TOOL_NOT_FOUND，Agent 改用其他方式
```

---

## 5. 错误信息验收标准

**所有错误必须同时满足**：
1. 退出码非 0
2. 含稳定 error_code（不在后续版本中改名）
3. 含人类可读 message
4. 含 JSON 结构输出（加 --json 时）

**错误信息不得暴露**：
- 系统内部路径结构
- workspace root 完整路径
- 配置敏感信息

**错误信息示例**（Human 可读）：
```
ERROR: HIGH_RISK_OPERATION
Hint: This operation requires explicit --force confirmation. Add --force to execute.
```

**错误信息示例**（JSON）：
```json
{
  "ok": false,
  "error": {
    "code": "HIGH_RISK_OPERATION",
    "exitCode": 1,
    "message": "this operation requires explicit --force confirmation"
  }
}
```

---

## 6. 产品验收检查表

| # | 检查项 | 验收标准 |
|---|--------|---------|
| P1 | --list 返回所有工具 | 工具列表完整，不遗漏 |
| P2 | --list 无参数时成功 | workspace 未初始化不崩溃 |
| P3 | describe 返回完整 Schema | 含 params、required、type |
| P4 | describe 未知工具返回错误 | error_code = TOOL_NOT_FOUND |
| P5 | call High 工具无 --force 被拒 | 提示明确，给出解决方向 |
| P5b | file_put overwrite=true 无 --force 被拒 | 提示加 --force |
| P6 | call 有 --force 执行成功 | 返回操作结果 |
| P7 | --help 给出正确用法 | 人类可读，不含内部细节 |
| P8 | 所有输出支持 --json | 人类输出和 JSON 输出并存 |
| P9 | error_code 跨版本稳定 | 不改名，不删除 |
| P10 | 错误信息不泄露系统路径 | 不暴露 workspace root 或配置路径 |

---

## 7. 优先级说明

**P1-P3**：核心接口完整性，AI Agent 能否正确调用取决于此
**P4-P6**：执行层，安全性关键路径
**P7-P8**：用户体验，辅助功能
**P9-P10**：长期稳定性承诺

Phase 1 通过验收后，再推进 Phase 2（Client 中转层、并发场景）。

---

## 8. 待产品确认的开放点

~~1. `file_put` 覆盖行为：已解决（方案 B，参数区分，见下方行为矩阵）~~
2. `workspace_info` 工具：Agent 需要知道当前 workspace root，用于日志和调试。应返回什么字段？
3. 日志审计保留期：--force 执行日志保留多久？超出存储限制时如何处理？

---

## 9. file_put 行为矩阵（产品+QA+IT 三方确认）

| 场景 | 结果 |
|------|------|
| overwrite=false（默认）+ 文件不存在 | 新建成功 |
| overwrite=false + 文件已存在 | 返回 FILE_EXISTS，提示可设 overwrite=true |
| overwrite=true + 文件不存在 | 新建成功（无风险升级） |
| overwrite=true + 文件存在 + 无 --force | 拒绝，提示加 --force |
| overwrite=true + 文件存在 + --force | 执行，写入，审计日志 |

---

## 10. 实现状态（截至 2026-05-13）

| 功能 | 状态 | 备注 |
|------|------|------|
| `tools --list` | ✅ 完成 | commit 1368dee |
| `tools describe` | ✅ 完成 | 含完整 Schema |
| `tools call` | ✅ 完成 | 已修复静默失败 Bug |
| `FILE_EXISTS` | ✅ 完成 | 已修复未返回 Bug |
| RiskLevel 枚举 | ✅ 完成 | None/Low/Medium/High |
| --force 门控 | ✅ 完成 | High 级别强制 |
| overwrite 参数 | ✅ 完成 | 三方确认行为矩阵 |
| Rust 编译测试 | ⏳ CI | 本地无 toolchain |
| 架构狮评审 | ⏳ 待 | 未响应 |