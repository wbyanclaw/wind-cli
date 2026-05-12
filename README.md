# windcli - AI Agent File Workspace

A safe file management CLI designed for AI agents. All operations are scoped to a controlled workspace directory.

## Quick Start (Copy & Paste)

```bash
# 1. Download
curl -L https://github.com/wbyanclaw/wind-cli/releases/download/v0.1.8/windcli.exe -o windcli.exe

# 2. Initialize workspace (run once)
windcli.exe init C:\agent-workspace

# 3. Start using (any directory)
cd C:\agent-workspace
echo "my notes" | windcli.exe write notes/todo.txt --stdin
windcli.exe read notes/todo.txt
windcli.exe list notes
```

## Commands (AI-Friendly Names)

| Command | Description | Example |
|---------|-------------|---------|
| `init <path>` | Create/switch workspace | `windcli init ~/workspace` |
| `list [path]` | List files | `windcli list notes` |
| `read <file>` | Read file content | `windcli read notes/todo.txt` |
| `write <file> --stdin` | Write file from pipe | `echo "hello" \| windcli write notes/a.txt --stdin` |
| `mkdir <path>` | Create directory | `windcli mkdir notes` |
| `delete <path> --yes` | Delete file | `windcli delete notes/old.txt --yes` |
| `delete <path> -r -y` | Delete directory | `windcli delete docs -r -y` |
| `version` | Show version | `windcli version` |
| `upgrade --check` | Check updates | `windcli upgrade --check` |

## AI Agent Patterns

```bash
# Write multiple files
echo "content1" | windcli write project/file1.txt --stdin
echo "content2" | windcli write project/file2.txt --stdin

# Read and process
windcli read data/input.txt
windcli list results

# Organize workspace
windcli mkdir agent-tasks
windcli mkdir agent-outputs
windcli mkdir agent-logs
```

## Security Rules

- All paths are relative to workspace root
- No `..` path traversal (blocked)
- No symlink following (blocked)
- Max file read: 10MB
- No glob/wildcard in delete

## Error Messages (Helpful)

Errors include suggestions:
```
ERROR: Path traversal attempt. Paths must be within workspace.
ERROR: File too large (max 10MB). Use streaming for larger files.
ERROR: Workspace not initialized. Run: windcli init <path>
```

## Windows PowerShell

```powershell
# Download
Invoke-WebRequest -Uri "https://github.com/wbyanclaw/wind-cli/releases/download/v0.1.8/windcli.exe" -OutFile "windcli.exe"

# Use
.\windcli init $env:USERPROFILE\agent-workspace
"hello" | .\windcli write notes\test.txt --stdin
.\windcli read notes\test.txt
```

## JSON Output (For Scripts)

Add `--json` flag for machine-readable output:
```bash
windcli --json read notes/todo.txt
```
Returns:
```json
{
  "ok": true,
  "file": "notes/todo.txt",
  "size_bytes": 12,
  "content": "hello world"
}
```

## Installation from Source

```bash
git clone git@github.com:wbyanclaw/wind-cli.git
cd wind-cli
cargo build --release
./target/release/windcli version
```