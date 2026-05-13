# Zero-dependency Windows installer for windcli
# Usage: .\install.ps1
# Or:   irm https://github.com/wbyanclaw/wind-cli/releases/latest/download/install.ps1 | iex

param(
    [string]$Version = "v0.1.10"
)

$ErrorActionPreference = "Stop"

if ($PSVersionTable.PSVersion.Major -lt 5) {
    Write-Error "PowerShell 5.0+ required. https://aka.ms/powershell"
    exit 1
}

$InstallDir = "$env:LOCALAPPDATA\wind-cli"
$ExeName = "windcli.exe"

Write-Host "Installing windcli to $InstallDir ..."

# Direct download — no GitHub API, no rate limit
$tag = $Version
$downloadUrl = "https://github.com/wbyanclaw/wind-cli/releases/download/$tag/$ExeName"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$destPath = Join-Path $InstallDir $ExeName

Write-Host "Downloading v$tag from GitHub ..."
Invoke-WebRequest -Uri $downloadUrl -OutFile $destPath -UserAgent "windcli-install"

# Add to user PATH
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$InstallDir*") {
    Write-Host "Adding $InstallDir to user PATH ..."
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
} else {
    Write-Host "PATH already contains $InstallDir"
}

# Verify
if (Test-Path $destPath) {
    $sha = (Get-FileHash $destPath -Algorithm SHA256).Hash
    Write-Host ""
    Write-Host "Installed:  $destPath"
    Write-Host "SHA256:     $sha"
    Write-Host ""
    Write-Host "Run 'windcli version' to verify."
    Write-Host "(Restart terminal if PATH was updated.)"
} else {
    Write-Error "Download failed."
    exit 1
}
