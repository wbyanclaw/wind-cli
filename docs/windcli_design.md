# wind-cli v0.2 DRI 规格草案

状态：DRI 合并草案，待产品、架构、QA、IT 最终签字。

来源：
- v0.1 附件：`attachments/wind-cli/windcli_full_spec.md`
- 产品侧草案：`attachments/wind-cli/windcli_v0.2_product_sections.md`
- 评审线程：`#wind-cli:4acb9cb3`

## 1. 一期定位

一期定位为：受控 workspace 文件能力 + `windlocal://` 安全解析 MVP。

`wind` 不是通用本地执行器，不是全盘文件管理器，也不是跨平台元数据同步工具。一期目标是让开发者、AI Agent 和自动化脚本可以在一个明确、安全、可验收的 workspace 内读写文件，并能安全解析 `windlocal://` 入口。

## 2. 目标用户

- 开发者/内部用户：下载单文件后快速初始化 workspace，并用 CLI 管理其中的文件。
- AI Agent/自动化脚本：通过稳定命令、结构化错误码、exit code 和 `--json` 输出读写 workspace。
- 后续客户端/网页入口：生成 `windlocal://` 链接，由本地 `wind open` 安全解析为受控 action。

## 3. P0 范围

- `wind version`
- `wind init`
- `wind ls [path]`
- `wind cat <path>`
- `wind put <path> --stdin`
- `wind put <path> --file <src>`
- `wind mkdir <path>`
- `wind rm <path>`
- `wind open --file <path>` | `wind open --search <query>` | `wind open --app` | `wind open --settings`
- `wind upgrade --check`

## 4. P0 非目标

- 不支持任意 shell command 或 `Platform::launch(cmd)`。
- 不支持完整自更新、自动替换、回滚、签名链和 Windows 自替换。
- 不承诺保留 `mtime/atime/owner/group/ACL/xattr/executable bit`。
- 不默认启用遥测、匿名心跳或自动错误上报。
- 不跟随 symlink/reparse point。
- 不支持 RPC、MCP、个人知识库、通用外部程序执行。

## 5. 模块边界

```text
src/
  main.rs
  cli.rs          # clap 参数定义，不放业务逻辑
  app.rs          # 命令调度、统一输出、错误转换、middleware
  config.rs       # 配置和标准路径
  errors.rs       # 错误码、exit code、错误输出结构
  workspace/      # safe_path、文件操作、原子写入
  windlocal/      # URI parse、schema、validate、action
  platform/       # 平台差异封装，不暴露任意 launch
  util/
tests/
```

`app.rs` 调度层必须统一经过 validation middleware：路径校验、权限/安全边界、结构化日志字段、错误转换。

## 6. Workspace 安全模型

### 6.1 workspace root

P0 采用单 active workspace 模型：
- `wind init [path]` 默认使用当前目录作为 workspace root。
- 初始化成功后，将 active workspace root 写入平台标准配置。
- 重复 init 同一路径是幂等操作。
- 对不同路径执行 init 默认失败并提示当前 active workspace。
- P0 不支持 `--switch`；多 workspace / workspace 切换能力放到 P1 设计。
- `wind open` 和所有未显式传 root 的命令都使用 active workspace root。

建议配置路径：
- Linux: `$XDG_CONFIG_HOME/wind/config.json` 或 `~/.config/wind/config.json`
- macOS: `~/Library/Application Support/wind/config.json`
- Windows: `%APPDATA%\wind\config.json`

### 6.2 `safe_path()` 语义

所有接受 workspace path 的命令必须调用 `safe_path()`。

P0 语义：
- 输入必须是 workspace-relative path。
- 拒绝 `../` 越界。
- 拒绝绝对路径。
- 拒绝 symlink/reparse point 目标或任一中间组件。
- 对普通路径归一化后，结果必须位于 workspace root 内。
- 缺失路径、权限失败、非 UTF-8/非法编码等情况返回稳定错误码。

P0 默认 no-follow：
- workspace 可以存在 symlink/reparse point。
- `ls` 可以将其标识为 `symlink`。
- `cat` / `put` / `rm` / `open` 遇到目标或任一中间组件是 symlink/reparse point，返回 `SYMLINK_NOT_SUPPORTED`。
- P0 不做“跟随后再二次校验”。
- P1/P2 如支持 `--follow-links`，必须单独设计并配套测试。

## 7. 文件操作行为

### 7.1 `cat`

- P0 hard limit: 10MB。
- 超过限制返回稳定错误码，不无上限读入内存。
- 是否支持二进制输出需在 v0.2 正文明确；如不支持，二进制文件返回结构化错误。

### 7.2 `put`

- `--stdin` 和 `--file <src>` 是一等输入方式。
- 不使用长 `<content>` 作为主要输入；如保留短文本便利模式，必须有长度限制。
- 写入采用目标同目录临时文件 + rename。
- 如果无法保证同文件系统原子 rename，返回明确错误，不做 copy+delete 降级。
- 磁盘满、权限失败、父目录不存在、临时文件创建失败都必须有稳定错误码。

### 7.3 `rm`

- 禁止 glob/wildcard。
- 默认只删除单个文件或空目录。
- 删除目录或非空目录必须显式 `--recursive --yes`。
- AI/脚本调用无确认参数时直接失败，不做交互式静默删除。
- P0 不做回收站。
- 建议支持 `--dry-run`，输出将删除对象，不执行删除。

### 7.4 并发

P0 不做文件锁，但文档必须明确：workspace 操作无多进程并发写保护，调用方负责协调。

P0 仍必须实现目标同目录临时文件 + rename 的基础原子写入。

P1/P2 可设计 workspace-level advisory lock 或文件级 lock。

## 8. windlocal URI grammar

### 8.1 基本规则

- Scheme 固定为 `windlocal://`。
- P0 action 只支持 `page` 和 `command`。
- 参数必须 URL decode 后校验。
- 多余参数默认拒绝。
- 任意非 `windlocal` scheme 拒绝。
- `wind open` P0 只 parse/validate 并输出 action/结果，不执行任意外部程序。
- 默认人类输出只说明 URI 已通过校验并展示受控 action；`--json` 输出稳定 action 结构。

### 8.2 Action schema

```rust
enum WindAction {
    Page { kind: PageKind, target: String },
    Command { id: CommandId },
}

enum PageKind {
    File,
    Search,
    App,
    Settings,
}

enum CommandId {
    ShowWorkspace,
    ShowApp,
    ShowSettings,
    CheckUpgrade,
}
```

P0 删除 `CommandStr` 和 `CommandParam`。

### 8.3 Page

```text
windlocal://page?kind=<PageKind>&target=<workspace-relative-path>
```

`PageKind` 白名单：
- `file`
- `search`
- `app`
- `settings`

### 8.4 Command

```text
windlocal://command?id=<CommandId>
```

`CommandId` 只允许非破坏性命令：
- `show_workspace`
- `show_settings`
- `check_upgrade`

### 8.5 合法样例

- `windlocal://page?kind=file&target=docs/readme.md`
- `windlocal://page?kind=app&target=.`
- `windlocal://command?id=show_workspace`
- `windlocal://command?id=check_upgrade`

### 8.6 必须拒绝的样例

- `windlocal://page?kind=file&target=../secret.txt`
- `windlocal://page?kind=file&target=/etc/passwd`
- `windlocal://command?id=rm_all`
- `windlocal://command?cmd=calc.exe`
- `windlocal://page?kind=file&target=docs/a.md&cmd=launch`
- `https://example.com`

### 8.7 `open --json` 输出示例

Page action:

```json
{
  "ok": true,
  "action": {
    "type": "page",
    "kind": "file",
    "target": "docs/readme.md"
  }
}
```

Command action:

```json
{
  "ok": true,
  "action": {
    "type": "command",
    "id": "check_upgrade"
  }
}
```

## 9. Platform 层边界

P0 删除任意命令执行能力。

```rust
trait Platform {
    fn open_uri(&self, uri: &str) -> Result<()>;
    fn get_workspace_root(&self) -> Result<PathBuf>;
}
```

`open_uri` 必须受 sandbox 约束；不得从 `windlocal` action 映射到任意 shell 或外部程序执行。

## 10. 输出、错误码和 exit code

所有命令必须支持：
- 默认人类可读输出。
- `--json` 机器可读输出。
- 稳定错误码。
- 标准 exit code。

所有 CLI 错误必须同时包含：
- 人类可读 `message`。
- 稳定 `error_code` 字符串。
- 标准 `exitCode`。
- 可选 `traceId`，用于用户主动反馈问题时关联本地日志/上下文。

建议 exit code：
- `0`: 成功
- `1`: 一般错误
- `2`: 参数/用法错误
- `3`: workspace/path 错误
- `4`: windlocal 协议错误
- `5`: IO/权限错误
- `6`: 平台/环境错误
- `7`: 网络/版本检查错误

建议错误码分段：
- `1-99`: 通用 CLI / 参数 / 配置
- `100-199`: workspace / path / 文件操作
- `200-299`: windlocal / URI / action schema
- `300-399`: platform / OS / 环境兼容
- `400-499`: upgrade check / network / release metadata

JSON 错误形态示例：

```json
{
  "ok": false,
  "error": {
    "code": "SYMLINK_NOT_SUPPORTED",
    "exitCode": 3,
    "message": "symlink paths are not supported in P0",
    "traceId": "..."
  }
}
```

## 11. Upgrade

P0 仅支持 `wind upgrade --check`：
- 返回当前版本、新版本、下载地址、发布时间等元数据。
- 不下载替换当前二进制。
- 网络失败、版本源不可达、响应格式错误必须有稳定错误码。

完整自更新放 P1/P2，必须包含：
- HTTPS。
- SHA-256 / signature verification。
- 临时下载文件。
- 备份回滚。
- Windows 影子脚本替换方案。
- macOS/Windows 代码签名远期规划。

## 12. Release Matrix

P0 发布目标：
- Linux: `x86_64-unknown-linux-musl`
- macOS: `x86_64-apple-darwin`
- macOS: `aarch64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`

最低 OS：
- Windows 10 Version 1809+
- macOS 12 Monterey+
- Linux kernel 3.10+

CI 必须覆盖核心命令和路径行为。

## 12.1 测试框架建议

P0 测试结构：
- `src/**` 模块单元测试：覆盖 `workspace/safe_path`、`windlocal/parse`、`errors`。
- `tests/cli.rs` 或等价集成测试：通过真实 CLI 调用覆盖 happy path 和失败场景。
- CLI 测试工具建议使用 `assert_cmd` + `predicates`，或项目确认的等价方案。
- 临时目录建议使用 `tempfile`，避免污染用户真实 workspace。
- GitHub Actions matrix 覆盖 Linux/macOS/Windows，并包含中文路径、空格路径、权限失败、symlink/reparse point 拒绝、10MB `cat` 限制、非法 URI、`rm --recursive --yes` 等关键场景。

## 13. 验收表

| 命令 | Happy Path | 失败/极端场景 |
| --- | --- | --- |
| `version` | 输出版本；`--json` 返回版本字段 | 无法读取版本元数据时返回结构化错误 |
| `init` | 首次初始化 workspace 成功 | 重复初始化、配置路径不可写、权限不足 |
| `ls` | 列出普通目录；支持中文/空格路径；symlink 标识为 `symlink` | 不存在路径、越界路径、权限失败 |
| `cat` | 读取小于等于 10MB 文本文件 | 超过 10MB、二进制/不可读、越界、symlink、权限失败 |
| `put --stdin` | 写入多行文本和特殊字符 | 目标越界、父目录不存在、磁盘满、同目录临时文件创建失败 |
| `put --file` | 从本地文件写入 workspace | 源文件不存在、源文件超限、目标 symlink、rename 失败 |
| `mkdir` | 创建目录 | 已存在、父目录不存在、越界、权限失败 |
| `rm` | 删除单个文件或空目录 | 非空目录无 `--recursive --yes`、glob、越界、symlink |
| `open` | `open --file <path>`, `open --search <query>`, `open --app`, `open --settings` | 缺少参数、路径穿越、非法参数 |
| `upgrade --check` | 返回当前版本、新版本和下载地址 | 网络失败、版本源不可达、响应格式错误 |

## 14. 进入实现前置条件

一期进入实现前，v0.2 规格书必须覆盖以下阻断项：

1. `WindAction` 使用枚举型 `PageKind` 和受控 `CommandId`。
2. `Platform::launch` 一期删除。
3. 所有路径经过 `safe_path()`，并明确 P0 no-follow。
4. `put` 支持 `--stdin` 和 `--file`。
5. 文件写入采用目标同目录临时文件 + rename。
6. `rm` 对目录和非空目录强制确认。
7. `upgrade` 一期只做 `--check`。
8. 所有命令支持 `--json`。
9. 错误码体系和标准 exit code 入规格。
10. `cat` 明确 10MB size limit。
11. 跨平台 CI matrix 明确。

## 15. 签字状态

- 产品 ✅ @首席产品-需求君：已签字确认一期定位、P0/P1 边界、非目标、用户场景、windlocal grammar、验收表。
- 架构 ✅ @首席架构-架构狮：已签字确认 `safe_path()`、middleware、windlocal schema、Platform 边界、原子写入策略、单 active workspace。
- QA ✅ @首席QA-找茬王：已签字确认安全硬约束、极端输入、路径逃逸、OOM、删除风险、11 条阻断项。
- IT ✅ @首席IT-稳上线：已签字确认 release matrix、最低 OS、CI/CD、发布检查、环境/部署侧硬指标。
- @kevinywb：P0 范围最终确认 / 范围变更入口。当前草案明确完整自更新移出 P0，仅保留 `upgrade --check`；如需改动，按范围变更处理。
