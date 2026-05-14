# wind

`wind` 是一个本地命令行工具，用来给开发者、脚本和 AI Agent 提供一个受控 workspace，让它们只能在明确的目录里读写文件。

## 解决什么问题

当你需要让自动化工具操作本地文件时，直接开放整个文件系统风险太高。`wind` 把能力收敛到一个 active workspace：

- 开发者可以快速初始化一个本地 workspace，并用 CLI 管理其中的文件。
- AI Agent / 脚本可以通过稳定命令和 `--json` 输出集成，而不是直接访问任意系统路径。
- 上层产品后续可以封装自己的本地入口；P0 不把底层协议直接暴露给终端用户。

P0 聚焦受控 workspace 文件能力。它不是通用 shell 执行器、全盘文件管理器，也不做文件元数据同步。

## 安装

### 从源码安装

前置条件：已安装 Rust stable 和 Cargo。可以先检查：

```bash
rustc --version
cargo --version
```

如果没有安装 Rust/Cargo，请先通过 rustup 或 Rust 官方安装方式安装。

```bash
git clone git@github.com:wbyanclaw/wind-cli.git
cd wind-cli
cargo install --path .
wind version
```

`cargo install --path .` 默认把二进制安装到 `~/.cargo/bin`。如果系统找不到 `wind`，把 Cargo bin 加到 `PATH`：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

需要长期生效时，把上面这一行加入 `~/.bashrc`、`~/.zshrc` 或你的 shell 配置文件。

### 二进制 Release

当前还没有正式 release 下载。P0 阶段先支持源码安装；CI 已按 Linux musl、macOS x86_64/arm64、Windows MSVC 的矩阵准备后续二进制产物。

## 3 步快速开始

```bash
# 1. 创建或选择一个 active workspace
wind init ~/my-workspace

# 后续命令里的路径都相对这个 workspace，不是相对当前 shell 目录

# 2. 在 workspace 内写入文件
printf "hello wind\n" | wind put notes/hello.md --stdin

# 3. 查看和读取文件
wind ls notes
wind cat notes/hello.md
```

## 5 分钟完整示例

```bash
# 初始化 workspace；目录不存在时会创建
wind init ~/my-workspace

# 创建嵌套目录
wind mkdir docs/getting-started

# 从 stdin 写入多行文本
cat <<'EOF' | wind put docs/getting-started/intro.md --stdin
# Intro

This file was created through wind.
EOF

# 浏览和读取
wind ls docs/getting-started
wind cat docs/getting-started/intro.md

# 给脚本/Agent 使用 JSON 输出
wind --json ls docs/getting-started

# 检查更新能力；P0 不自动替换二进制
wind upgrade --check
```

## 常用命令

```bash
wind version                           # 输出版本
wind init [path]                      # 初始化 workspace
wind ls [path]                        # 列文件
wind cat <path>                       # 读取文件（≤10MB）
wind put <path> --stdin               # 从 stdin 写文件
wind put <path> --file <local-source> # 从本地文件写
wind mkdir <path>                     # 创建目录
wind rm <path>                        # 删除文件/空目录
wind rm <path> --recursive --yes      # 删除非空目录
wind rm <path> --dry-run              # 预览删除
wind wft file <path>                  # 打开 workspace 文件（WFT 集成）
wind wft search <query>              # 搜索 workspace 内容
wind wft app                          # 打开应用视图
wind wft settings                    # 打开设置视图
wind upgrade --check                  # 检查更新（不替换二进制）

# 给脚本或 AI Agent 使用结构化输出
wind --json ls notes
```

## Workspace 模型

P0 只支持一个 active workspace。

- `wind init [path]` 会创建目录、解析为 canonical path，并写入平台标准配置文件。
- 对同一个路径重复执行 `wind init` 是幂等的。
- 对不同路径执行 `wind init` 会失败并提示当前 active workspace；P0 不支持 `--switch`。
- 文件命令只接受相对 workspace 的路径。

## 安全边界

所有文件命令在触碰文件系统前，都必须通过 workspace 安全层解析路径。

- 拒绝绝对路径和 `..` 路径逃逸。
- 拒绝 glob/wildcard 删除。
- `cat` 有 10MB hard limit，避免大文件直接打爆内存。
- `put` 使用目标同目录临时文件 + rename；如果不能保证原子 rename，则失败，不降级为 copy/delete。
- `rm` 删除非空目录必须显式传 `--recursive --yes`。
- P0 不承诺保留 `mtime`、`atime`、owner、ACL、xattr、executable bit 等元数据。

### Symlink 行为

P0 是 no-follow，但 `ls` 允许展示 symlink 条目，方便用户理解 workspace 里有什么。

| 命令 | symlink 行为 |
| --- | --- |
| `ls` | 展示条目，并标记为 `symlink`。 |
| `cat` | 返回 `SYMLINK_NOT_SUPPORTED`。 |
| `put` | 如果目标路径或已存在的父级组件是 symlink/reparse point，则失败。 |
| `rm` | 返回 `SYMLINK_NOT_SUPPORTED`。 |
这个差异是刻意设计的：允许看见 symlink，但不允许通过 symlink 读写或逃逸 workspace。

## 协议入口说明

`windlocal://` 属于上层产品的内部集成协议，通过 `wind wft` 命令封装给终端用户使用。WFT 子命令包括：file、search、app、settings、workspace、upgrade、url。

## JSON 与错误输出

所有命令都支持 `--json`。成功输出包含 `"ok": true`；错误输出包含稳定错误码和 exit code：

```json
{
  "ok": false,
  "error": {
    "error_code": "PATH_TRAVERSAL",
    "exitCode": 3,
    "message": "path traversal attempt detected",
    "traceId": "..."
  }
}
```

P0 兼容性约定：

- `ok`、`error.error_code`、`error.exitCode`、`error.message` 是稳定字段。
- `traceId` 可选，用于排查问题。
- 人类可读的 `message` 可以被澄清，但稳定 `error_code` 不应在无兼容说明时改名。

## Exit Code

| Code | 含义 |
| --- | --- |
| 0 | 成功 |
| 1 | 通用错误 |
| 2 | 参数/用法错误 |
| 3 | workspace / 路径错误 |
| 4 | 协议/内部入口错误 |
| 5 | IO / 权限错误 |
| 6 | 平台 / 环境错误 |
| 7 | 网络 / 版本检查错误 |

## 架构说明

当前代码保持小而直，但模块职责必须清楚：

| 模块 | 职责 |
| --- | --- |
| `src/cli.rs` | 只定义 clap 参数，不放业务逻辑。 |
| `src/app.rs` | 命令 handler 和调度层，把 CLI 命令转成 workspace 操作。 |
| `src/workspace/` | workspace 路径安全和文件操作。 |
| `src/windlocal/` | windlocal:// URI 解析和验证。 |
| `src/tools.rs` | Agent Protocol 工具接口。 |
| `src/config.rs` | 平台标准配置路径和 active workspace 持久化。 |
| `src/errors.rs` | 错误类型、稳定错误码、exit code 和 JSON 错误输出。 |
| `src/platform/` | OS 抽象预留边界。P0 不启动外部程序。 |

后续如果 `workspace` 继续变大，可以拆成 path security 和 file operations 两个子模块；P0 先避免重构扩大范围。

## 开发

```bash
cargo fmt
cargo test
cargo clippy
cargo build --release
```

集成测试会使用 `WIND_CONFIG_PATH` 隔离配置文件，避免污染真实用户配置。

## P0 明确不做

- 完整自更新和二进制替换。
- 任意 shell 执行或 `Platform::launch(cmd)`。
- follow symlink/reparse point。
- 文件元数据同步。
- 多 workspace 切换。
- 遥测或匿名心跳。
