# windcli v0.2.0 Phase 1 Release

## New Features

### Agent Protocol - Tools Subcommand
- `windcli tools list`: List all available tools (simplified metadata)
- `windcli tools describe <name>`: Show single tool schema with full parameters
- `windcli tools call <name> --params <json>`: Call tool with optional --force flag
- `windcli tools --help`: Help information

### RiskLevel System
- Four levels: None, Low, Medium, High
- High risk operations (rm, write with overwrite) require `--force` flag
- Schema validation for all tool parameters

### open Command - windlocal Protocol
- `windcli open --file <path>`: Open workspace file with system default app
- `windcli open --search <query>`: Open windlocal search page
- `windcli open --app`: Open windlocal application
- `windcli open --settings`: Open windlocal settings

### upgrade --check - GitHub API Integration
- Fetches latest version from GitHub releases
- Returns `update_available` flag when new version exists
- Semantic version comparison

## Bug Fixes
- Fixed `tools call` not executing actual commands
- Fixed `FILE_EXISTS` error not returned when overwrite=false
- FileExists error no longer exposes internal workspace paths
- `install.ps1`: fixed executable name to `windcli.exe`
- `install.ps1`: replaced the old piped installer with a download-and-run command
- `install.ps1`: defaults to the latest release, verifies SHA256, updates user PATH, and prints a copyable `-ExecutionPolicy Bypass` command when PowerShell blocks script execution

## Installation

Open PowerShell and run:
```powershell
$p = "$env:TEMP\windcli-install.ps1"; irm https://github.com/wbyanclaw/wind-cli/releases/latest/download/install.ps1 -OutFile $p; powershell -NoProfile -ExecutionPolicy Bypass -File $p -NoPause
```

If PowerShell blocks a downloaded script, copy the command printed by the installer. It will look like:
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\install.ps1"
```

## Verification
```powershell
windcli --version
```

## Version
This release: v0.2.0 Phase 1
