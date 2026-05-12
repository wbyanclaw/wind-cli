# Zero-dependency Windows installer for windcli
# Usage: irm https://github.com/wbyanclaw/wind-cli/releases/latest/download/install.ps1 | iex
# Or:   .\install.ps1

param(
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"

# Pre-check
if ($PSVersionTable.PSVersion.Major -lt 5) {
    Write-Error "PowerShell 5.0+ required. Please upgrade: https://aka.ms/powershell"
    exit 1
}

# Determine install dir (user-local, no admin required)
$InstallDir = "$env:LOCALAPPDATA\wind-cli"
$ExeName = "wind.exe"

Write-Host "Installing windcli to $InstallDir ..."

# Download latest release
if ($Version -eq "latest") {
    $apiUrl = "https://api.github.com/repos/wbyanclaw/wind-cli/releases/latest"
    Write-Host "Fetching latest release info from GitHub API..."
    $release = Invoke-RestMethod -Uri $apiUrl -UserAgent "windcli-install"
    $tag = $release.tag_name
    $downloadUrl = $release.assets | Where-Object { $_.name -eq "$ExeName" } | Select-Object -First 1 -ExpandProperty browser_download_url
} else {
    $tag = $Version
    $downloadUrl = "https://github.com/wbyanclaw/wind-cli/releases/download/$tag/$ExeName"
}

if (-not $downloadUrl) {
    Write-Error "Could not find windcli.exe asset for tag '$tag'. Please check https://github.com/wbyanclaw/wind-cli/releases"
    exit 1
}

Write-Host "Downloading $tag from $downloadUrl ..."
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$destPath = Join-Path $InstallDir $ExeName
Invoke-WebRequest -Uri $downloadUrl -OutFile $destPath -UserAgent "windcli-install"

# Add to user PATH
$pathEntry = $InstallDir
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$InstallDir*") {
    Write-Host "Adding $InstallDir to user PATH ..."
    $newPath = "$currentPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    $env:Path = "$env:Path;$InstallDir"  # update current session
} else {
    Write-Host "PATH already contains $InstallDir"
}

# Verify
Write-Host ""
Write-Host "Verifying installation ..."
$installed = Join-Path $InstallDir $ExeName
if (Test-Path $installed) {
    $sha = (Get-FileHash $installed -Algorithm SHA256).Hash
    Write-Host "Installed: $installed"
    Write-Host "SHA256:  $sha"
    Write-Host ""
    Write-Host "Run 'wind version' to verify PATH is working."
    Write-Host "(You may need to restart your terminal for PATH changes to take effect)"
} else {
    Write-Error "Installation verification failed."
    exit 1
}
