# windcli Windows Release

This release provides a Windows x86_64 executable for the P0 wind CLI MVP.

## Download

- `windcli.exe`: standalone Windows executable (CLI command: `wind`, project: `windcli`).
- `windcli-windows-x86_64.zip`: zipped Windows executable.
- `SHA256SUMS.txt`: SHA256 checksums for verification.

Verify the executable checksum in PowerShell:

```powershell
Get-FileHash .\windcli.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

## Quick Start

Run directly from the download directory:

```powershell
.\windcli.exe version
.\windcli.exe init $env:USERPROFILE\wind-workspace
"hello wind" | .\windcli.exe put notes\hello.md --stdin
.\windcli.exe ls notes
.\windcli.exe cat notes\hello.md
.\windcli.exe open --file notes\hello.md
.\windcli.exe upgrade --check
```

To use `windcli.exe` from any terminal, place it in a directory that is already on
`PATH`, or add the download directory to your Windows `PATH`.

## P0 Scope

- Controlled workspace file operations: `init`, `ls`, `cat`, `put`, `mkdir`, `rm`.
- `wind open --file <path>` / `--search <query>` / `--app` / `--settings`: windlocal protocol encapsulated, user uses flags not raw URIs.
- Single active workspace.
- No-follow symlink/reparse-point policy.
- `upgrade --check` reports capability only; automatic self-update is not included in this release.

## Not Included In This Release

- macOS artifacts.
- Full automatic self-update.
- Arbitrary shell/program launch.
- Multi-workspace switching.
- Metadata synchronization.