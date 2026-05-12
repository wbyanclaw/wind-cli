# wind CLI — AI Agent File Workspace

A safe file management CLI designed for AI agents. All operations are scoped to a controlled workspace directory.

## Download

Latest release: https://github.com/wbyanclaw/wind-cli/releases/latest

### Windows — One-Click Install (Recommended)

Open PowerShell and run:

```powershell
irm https://github.com/wbyanclaw/wind-cli/releases/latest/download/install.ps1 | iex
```

This downloads `wind.exe`, places it in your user directory, adds it to `PATH`, and verifies the installation. No administrator rights required.

### Manual Download

- `wind.exe`: standalone Windows executable
- `wind-windows-x86_64.zip`: zipped package
- `install.ps1`: one-click installer
- `SHA256SUMS.txt`: checksums for verification

## Quick Start

```bash
# Initialize workspace (run once)
wind init ~/my-workspace

# Write a file
echo "hello world" | wind write notes/hello.txt --stdin

# Read a file
wind read notes/hello.txt

# List files
wind list notes

# Delete a file (force = --recursive --yes, AI agent friendly)
wind delete notes/hello.txt --force
```

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `init <path>` | Create/switch workspace | `wind init ~/workspace` |
| `list [path]` | List files in workspace | `wind list notes` |
| `read <file>` | Read file content (max 10MB) | `wind read notes/todo.txt` |
| `write <file> --stdin` | Write file from pipe | `echo 'hello' \| wind write notes/a.txt --stdin` |
| `write <file> --content "text"` | Write file from argument | `wind write notes/a.txt --content "hello"` |
| `mkdir <path>` | Create directory | `wind mkdir notes` |
| `delete <path>` | Delete file | `wind delete notes/old.txt` |
| `delete <path> --force` | Force delete (file or directory, no confirm) | `wind delete docs -f` |
| `open --file <path>` | Validate workspace file via windlocal | `wind open --file docs/readme.md` |
| `open --search <query>` | Search workspace content | `wind open --search "TODO"` |
| `version` | Show version | `wind version` |
| `upgrade --check` | Check for updates | `wind upgrade --check` |

All commands support `--json` for machine-readable output.

## JSON Output

```bash
wind --json list notes
```

```json
{
  "ok": true,
  "root": "/home/user/my-workspace",
  "entries": [...]
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Usage/argument error |
| 3 | Workspace / path error |
| 4 | windlocal protocol error |
| 5 | IO / permission error |

## Security Rules

- All paths are resolved relative to workspace root
- Path traversal (`..`) is blocked
- Symlink/reparse-point following is blocked (ls shows but does not follow)
- File reads are capped at 10MB
- Glob/wildcard patterns are blocked in delete

## Installation from Source

```bash
git clone git@github.com:wbyanclaw/wind-cli.git
cd wind-cli
cargo install --path .
wind version
```

`cargo install --path .` installs the binary to `~/.cargo/bin`. Add it to `PATH` if needed:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

## Build

```bash
cargo build --release
cargo test
```
