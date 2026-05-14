# windcli

`windcli` is a controlled workspace file CLI for AI agents and local automation. All file operations are scoped to one initialized workspace directory.

## Windows Install

Open PowerShell and run this recommended command:

```powershell
$p = "$env:TEMP\windcli-install.ps1"; irm https://github.com/wbyanclaw/wind-cli/releases/latest/download/install.ps1 -OutFile $p; powershell -NoProfile -ExecutionPolicy Bypass -File $p -NoPause
```

The installer downloads the latest `windcli.exe`, verifies its SHA256 checksum, installs it to `%LOCALAPPDATA%\wind-cli`, and adds that directory to your user `PATH`.

If PowerShell blocks a downloaded script, run the command printed by the installer:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\install.ps1"
```

After installation, open a new terminal and verify:

```powershell
windcli --version
```

Latest release: <https://github.com/wbyanclaw/wind-cli/releases/latest>

## Quick Start

```bash
# Initialize workspace once
windcli init ~/my-workspace

# Write a file
echo "hello world" | windcli write notes/hello.txt --stdin

# Read a file
windcli read notes/hello.txt

# List files
windcli ls notes

# Delete a file
windcli rm notes/hello.txt --yes
```

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `init <path>` | Create the active workspace | `windcli init ~/workspace` |
| `ls [path]` | List files in workspace | `windcli ls notes` |
| `read <file>` | Read file content, capped at 10MB | `windcli read notes/todo.txt` |
| `write <file> --stdin` | Write file from stdin | `echo 'hello' \| windcli write notes/a.txt --stdin` |
| `write <file> --content "text"` | Write file from an argument | `windcli write notes/a.txt --content "hello"` |
| `mkdir <path>` | Create a directory | `windcli mkdir notes` |
| `rm <path> --yes` | Delete a file | `windcli rm notes/old.txt --yes` |
| `rm <path> --force` | Delete recursively without prompts | `windcli rm docs --force` |
| `open --file <path>` | Validate and open a workspace file | `windcli open --file docs/readme.md` |
| `open --search <query>` | Search workspace content | `windcli open --search "TODO"` |
| `tools list` | List Agent Protocol tools | `windcli tools list` |
| `tools describe <name>` | Describe one Agent Protocol tool | `windcli tools describe read` |
| `tools call <name> --params <json>` | Call one Agent Protocol tool | `windcli tools call read --params '{"path":"notes/a.txt"}'` |
| `version` | Show version as JSON-like output | `windcli version` |
| `upgrade --check` | Check GitHub releases for updates | `windcli upgrade --check` |

All commands support `--json` for machine-readable output.

```bash
windcli --json ls notes
```

## Security Rules

- All paths are resolved relative to the active workspace root
- Path traversal (`..`) is blocked
- Symlink and reparse-point following is blocked
- File reads are capped at 10MB
- Glob and wildcard patterns are blocked in delete operations
- High-risk tool calls require explicit `--force`

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `WIND_CONFIG_PATH` | `~/.wind/config.json` | Path to config file, useful for test isolation |

## Build From Source

```bash
git clone git@github.com:wbyanclaw/wind-cli.git
cd wind-cli
cargo build --release
cargo test
```
