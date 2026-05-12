# AI Agent CLI 设计规范

> 目标：wind CLI 给 AI Agent 使用时，应该像调用本地函数一样可靠、可预测、无交互。

## 核心原则

1. **无交互阻塞** — AI Agent 无法"按 Enter"，所有操作必须通过参数完成。
2. **幂等优先** — 重复执行同一命令不应报错，简化重试逻辑。
3. **结构化输出** — JSON 是唯一可靠输出格式，人类可读文本作为辅助。
4. **明确的退出码** — 0 成功，>0 有错误，AI 直接判断成功/失败。
5. **错误信息可程序化** — 错误码稳定，message 清晰，不需要 AI 猜测。

---

## 输出格式

### 成功输出

```json
{
  "ok": true,
  "data": { ... }
}
```

### 错误输出

```json
{
  "ok": false,
  "error": {
    "error_code": "PATH_NOT_FOUND",
    "exitCode": 3,
    "message": "path does not exist in workspace",
    "traceId": "..."
  }
}
```

**规则**：
- 所有命令（无论是否带 `--json`）在出错时都必须输出结构化错误。
- `--json` 时输出完整 JSON；不带 `--json` 时人类可读输出 + 相同 Exit Code。
- error_code 是稳定契约，不能改名；message 可以改进，但不改 error_code。

---

## Exit Code 规范

| Exit Code | 含义 | AI 行为 |
|-----------|------|--------|
| 0 | 成功 | 继续下一步 |
| 1 | 一般错误 | 记录 error message，重试或上报 |
| 2 | 参数/用法错误 | 修正参数，停止重试 |
| 3 | workspace / 路径错误 | 检查路径是否在 workspace 内 |
| 4 | windlocal 协议错误 | 检查 URI 格式 |
| 5 | IO / 权限错误 | 检查权限，重试或上报 |
| 6 | 平台 / 环境错误 | 上报用户 |
| 7 | 网络 / 版本检查错误 | 可重试，有上限 |

---

## 命令幂等要求

| 命令 | 幂等要求 |
|------|---------|
| `wind init [path]` | 对同一路径重复执行 → 成功（幂等），不同路径 → 报错并提示当前 active workspace |
| `wind mkdir <path>` | 目录已存在 → 成功（幂等） |
| `wind put <path> --stdin` | 文件已存在 → 覆盖成功（幂等） |
| `wind rm <path>` | 文件不存在 → 报错（需要显式 `--force` 才跳过检查） |
| `wind open --file <path>` | 文件不存在 → 报错，路径不安全 → 报错 |

---

## AI Agent 无阻塞要求

### 删除命令

`wind rm` 对 AI Agent 的问题是 `--recursive --yes` 仍然需要两个参数，且 AI 每次重试都会成功。

建议：加 `--force` 简写：
```bash
wind rm <path> --force   # 等价于 --recursive --yes
```

### Init 命令

已幂等，无需改动。

### 交互式 Prompt 排除

以下场景不应出现交互式 Prompt：
- 所有命令不接受 "yes/no" 交互
- 所有错误不等待用户输入
- `upgrade --check` 不触发确认，直接返回

---

## JSON 输出覆盖率

| 命令 | JSON 成功 | JSON 错误 | 备注 |
|------|----------|----------|------|
| `version` | ✅ | ✅（极少） | — |
| `init` | ✅ | ✅ | — |
| `ls` | ✅ | ✅ | — |
| `cat` | ✅ | ✅ | — |
| `put` | ✅ | ✅ | — |
| `mkdir` | ✅ | ✅ | — |
| `rm` | ✅ | ✅ | — |
| `open --file` | ✅ | ✅ | — |
| `open --search` | ✅ | ✅ | — |
| `open --app` | ✅ | ✅ | — |
| `open --settings` | ✅ | ✅ | — |
| `upgrade --check` | ✅ | ✅ | — |

---

## 版本稳定性承诺

v0.1.x 范围内：
- error_code 集合不变（新增需要明确公告）
- JSON 输出字段 `ok` / `data` / `error` 结构不变
- 命令参数不删除，只可增加别称
- 退出码含义不变

---

## AI Agent 典型调用示例

### 初始化 workspace
```bash
wind init ~/my-workspace          # 幂等
```

### 写入文件
```bash
echo "hello world" | wind put notes/hello.txt --stdin
```

### 读取文件（容错）
```bash
# AI 会检查 exit code，失败重试
wind cat notes/hello.txt || echo "file not found"
```

### 列出目录
```bash
wind --json ls notes
```

### 安全校验 windlocal
```bash
wind --json open --file docs/readme.md
```

### 删除文件（force 幂等）
```bash
wind rm notes/hello.txt --force
```

### 检查更新
```bash
wind upgrade --check
```

---

## 设计决策记录

| 决策 | 结论 | 理由 |
|------|------|------|
| 命令名 | `wind` | 短，AI token 成本低，无系统冲突 |
| 二进制名 | `wind.exe`（Windows）/ `wind`（Linux/macOS） | 与命令名一致 |
| rm 是否需要交互 | 否 | AI 无法响应交互，强制 --recursive --yes |
| init 对已存在路径 | 幂等成功 | 避免 AI 重试逻辑复杂化 |
| put 对已存在文件 | 覆盖成功 | 幂等，无破坏性（workspace 内可控） |
| open 解析后是否执行 | P0 仅 parse/validate | 安全边界，不执行外部程序 |