# windcli

A controlled workspace CLI for AI agents and developers to safely manage local files.

## Quick Setup (AI Agent)

```bash
# Download and run
curl -L https://github.com/wbyanclaw/wind-cli/releases/download/v0.1.6/windcli.exe -o windcli.exe

# Initialize workspace (one-time)
windcli.exe init C:\agent-workspace

# Use from any directory
cd C:\agent-workspace
windcli.exe put notes/todo.md --stdin
windcli.exe cat notes/todo.md
windcli.exe ls notes
```

## Commands

| Command | Description |
|---------|-------------|
| `windcli init <path>` | Initialize/create workspace |
| `windcli ls [path]` | List files |
| `windcli cat <path>` | Read file (≤10MB) |
| `windcli put <path> --stdin` | Write file from stdin |
| `windcli mkdir <path>` | Create directory |
| `windcli rm <path>` | Delete file |
| `windcli rm <path> --recursive --yes` | Delete directory |
| `windcli version` | Show version |
| `windcli upgrade --check` | Check for updates |

## Security

- All paths are relative to workspace root
- No `..` path traversal allowed
- No symlink following
- 10MB file size limit for reads

## Windows Quick Start

```powershell
# Download
Invoke-WebRequest -Uri "https://github.com/wbyanclaw/wind-cli/releases/download/v0.1.6/windcli.exe" -OutFile "windcli.exe"

# Initialize workspace
.\windcli.exe init $env:USERPROFILE\wind-workspace

# Write and read files
"hello world" | .\windcli.exe put notes\test.txt --stdin
.\windcli.exe cat notes\test.txt
```

## Installation from Source

```bash
git clone git@github.com:wbyanclaw/wind-cli.git
cd wind-cli
cargo build --release
./target/release/windcli.exe version
```

## Architecture

- `src/cli.rs` — CLI argument definitions
- `src/app.rs` — Command handlers
- `src/workspace/` — Safe file operations
- `src/config.rs` — Workspace configuration
- `src/errors.rs` — Error codes and messages