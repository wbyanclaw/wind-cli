# wind CLI

**受控 workspace 文件管理 + windlocal 安全解析 MVP**

## 定位

wind 不是通用本地执行器，不是全盘文件管理器，也不是元数据同步工具。

一期目标：让开发者、AI Agent 和自动化脚本可以在一个明确、安全、可验收的 workspace 内读写文件，并能安全解析 `windlocal://` 入口。

## 核心命令

```bash
wind version                      # 输出版本
wind init [path]                  # 初始化 workspace
wind ls [path]                    # 列文件
wind cat <path>                   # 读取文件（≤10MB）
wind put <path> --stdin           # 从 stdin 写文件
wind put <path> --file <src>      # 从本地文件写
wind mkdir <path>                  # 创建目录
wind rm <path>                     # 删除文件/空目录
wind rm <path> --recursive --yes  # 删除非空目录（需确认）
wind open <uri>                   # 解析 windlocal URI
wind upgrade --check              # 检查更新（不替换二进制）
```

## 安全约束（P0）

- 所有路径必须通过 `safe_path()` 校验，workspace 外路径一律拒绝
- P0 默认 no-follow：遇到 symlink/reparse point 返回 `SYMLINK_NOT_SUPPORTED`
- `put` 使用目标同目录临时文件 + rename；跨分区无法原子则失败
- `rm` 对目录/非空目录强制 `--recursive --yes`
- `wind open` P0 只做 parse/validate，不执行任意外部程序
- 任意 `Platform::launch(cmd)` 在 P0 中禁用

## windlocal URI Grammar

```text
windlocal://page?kind=file&target=docs/readme.md
windlocal://command?id=show_workspace
```

- `kind`: file / search / app / settings
- `id`: show_workspace / show_settings / check_upgrade

## 输出格式

所有命令支持 `--json`，返回结构化结果：

```json
{
  "ok": true,
  "content": "..."
}
```

错误格式：

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

## 退出码

| 退出码 | 含义 |
|--------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 参数/用法错误 |
| 3 | workspace / 路径错误 |
| 4 | windlocal 协议错误 |
| 5 | IO / 权限错误 |
| 6 | 平台 / 环境错误 |
| 7 | 网络 / 版本检查错误 |

## 构建

```bash
cargo build --release
```

## 测试

```bash
cargo test
cargo test -- --test-threads=1   # 避免并发竞争
```