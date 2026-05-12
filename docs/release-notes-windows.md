# windcli Windows Release

This release provides a Windows x86_64 executable for the windcli P0 MVP.

## Download

- `wind.exe`: standalone Windows executable.
- `wind-windows-x86_64.zip`: zipped Windows executable.
- `install.ps1`: one-click installer (recommended).
- `SHA256SUMS.txt`: SHA256 checksums for verification.

## One-Click Install (Recommended)

Open PowerShell and run:

```powershell
irm https://github.com/wbyanclaw/wind-cli/releases/latest/download/install.ps1 | iex
```

This downloads the latest `wind.exe`, places it in your user directory, adds it to your `PATH`, and verifies the installation. No administrator rights required.

## Verify Download

```powershell
Get-FileHash .\wind.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

## Quick Start

```powershell
.\wind.exe version
.\wind.exe init $env:USERPROFILE\wind-workspace
"hello wind" | .\wind.exe put notes\hello.md --stdin
.\wind.exe ls notes
.\wind.exe cat notes\hello.md
.\wind.exe open --file notes\hello.md
.\wind.exe upgrade --check
```

To use `wind` from any terminal, place `wind.exe` in a directory on your `PATH`, or use the installer above which handles this automatically.

## P0 Scope

- Controlled workspace file operations: `init`, `ls`, `cat`, `put`, `mkdir`, `rm`.
- `wind open --file <path>` / `--search <query>` / `--app` / `--settings`: windlocal protocol encapsulated, user uses CLI flags.
- Single active workspace.
- No-follow symlink/reparse-point policy.
- `upgrade --check` reports capability only; automatic self-update is not included in this release.

## Not Included In This Release

- macOS artifacts.
- Full automatic self-update.
- Arbitrary shell/program launch.
- Multi-workspace switching.
- Metadata synchronization.