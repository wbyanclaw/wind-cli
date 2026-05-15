# wind

`wind` is a controlled workspace file CLI for AI agents and local automation. All file operations are scoped to one initialized workspace directory.

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
wind --version
```

Latest release: <https://github.com/wbyanclaw/wind-cli/releases/latest>

## Quick Start

```bash
# Initialize workspace once
wind init ~/my-workspace

# Write a file
echo "hello world" | wind write notes/hello.txt --stdin

# Read a file
wind read notes/hello.txt

# List files
wind ls notes

# Delete a file
wind rm notes/hello.txt --yes
```

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `init <path>` | Create the active workspace | `wind init ~/workspace` |
| `ls [path]` | List files in workspace | `wind ls notes` |
| `read <file>` | Read file content, capped at 10MB | `wind read notes/todo.txt` |
| `write <file> --stdin` | Write file from stdin | `echo 'hello' \| wind write notes/a.txt --stdin` |
| `write <file> --content "text"` | Write file from an argument | `wind write notes/a.txt --content "hello"` |
| `write <file> --overwrite` | Allow overwriting existing files | `wind write notes/a.txt --overwrite` |
| `mkdir <path>` | Create a directory | `wind mkdir notes` |
| `rm <path> --yes` | Delete a file | `wind rm notes/old.txt --yes` |
| `rm <path> --force` | Delete recursively without prompts | `wind rm docs --force` |
| `wft file <path>` | Open workspace file (WFT) | `wind wft file docs/readme.md` |
| `wft search <query>` | Search workspace content | `wind wft search "TODO"` |
| `wft app` | Open app view | `wind wft app` |
| `wft settings` | Open settings | `wind wft settings` |
| `wft workspace` | Show workspace info | `wind wft workspace` |
| `tools list` | List Agent Protocol tools | `wind tools list` |
| `tools describe <name>` | Describe one Agent Protocol tool | `wind tools describe read` |
| `tools call <name> --params <json>` | Call one Agent Protocol tool | `wind tools call read --params '{"path":"notes/a.txt"}'` |
| `version` | Show version | `wind version` |
| `upgrade --check` | Check GitHub releases for updates | `wind upgrade --check` |

All commands support `--json` for machine-readable output.

```bash
wind --json ls notes
```

## Security Rules

- All paths are resolved relative to the active workspace root
- Path traversal (`..`) is blocked
- Symlink and reparse-point following is blocked
- File reads are capped at 10MB
- Glob and wildcard patterns are blocked in delete operations
- Write operations require `--overwrite` flag to overwrite existing files
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
