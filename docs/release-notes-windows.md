# windcli v0.2 Phase 1 Release

## New Features

### Agent Protocol - Tools Subcommand
- `wind tools --list`: List all available tools (simplified metadata)
- `wind tools describe <name>`: Show single tool schema with full parameters
- `wind tools call <name> --params <json>`: Call tool with optional --force flag
- `wind tools --help`: Help information

### RiskLevel System
- Four levels: None, Low, Medium, High
- High risk operations (rm, write with overwrite) require `--force` flag
- Schema validation for all tool parameters

### open Command - windlocal Protocol
- `wind open --file <path>`: Open workspace file with system default app
- `wind open --search <query>`: Open windlocal search page
- `wind open --app`: Open windlocal application
- `wind open --settings`: Open windlocal settings

### upgrade --check - GitHub API Integration
- Fetches latest version from GitHub releases
- Returns `update_available` flag when new version exists
- Semantic version comparison

## Bug Fixes
- Fixed `tools call` not executing actual commands
- Fixed `FILE_EXISTS` error not returned when overwrite=false
- FileExists error no longer exposes internal workspace paths
- install.ps1: Fixed exe name (wind.exe → windcli.exe)

## Installation

Open PowerShell and run:
```powershell
irm https://github.com/wbyanclaw/wind-cli/releases/latest/download/install.ps1 | iex
```

## Verification
```powershell
Get-FileHash .\windcli.exe -Algorithm SHA256
```

## Version
This release: v0.2 Phase 1