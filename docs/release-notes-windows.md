# wind CLI Windows Release

This release provides a Windows x86_64 executable for the P0 wind CLI MVP.

## Download

- `wind.exe`: standalone Windows executable.
- `wind-windows-x86_64.zip`: zipped Windows executable.
- `SHA256SUMS.txt`: SHA256 checksums for verification.

Verify the executable checksum in PowerShell:

```powershell
Get-FileHash .\wind.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

## Quick Start

Run directly from the download directory:

```powershell
.\wind.exe version
.\wind.exe init $env:USERPROFILE\wind-workspace
"hello wind" | .\wind.exe put notes\hello.md --stdin
.\wind.exe ls notes
.\wind.exe cat notes\hello.md
.\wind.exe upgrade --check
```

To use `wind.exe` from any terminal, place it in a directory that is already on
`PATH`, or add the download directory to your Windows `PATH`.

## P0 Scope

- Controlled workspace file operations.
- Single active workspace.
- No-follow symlink/reparse-point policy.
- `upgrade --check` reports capability only; automatic self-update is not
  included in this release.

## Not Included In This Release

- macOS artifacts.
- Public `windlocal://` command entry. Protocol integration must be wrapped by
  an upper-layer product before it is exposed to users.
- Full automatic self-update.
- Arbitrary shell/program launch.
- Multi-workspace switching.
- Metadata synchronization.
