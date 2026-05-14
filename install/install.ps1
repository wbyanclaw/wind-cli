# Windows installer for windcli.
#
# Recommended command:
#   $p = "$env:TEMP\windcli-install.ps1"; irm https://github.com/wbyanclaw/wind-cli/releases/latest/download/install.ps1 -OutFile $p; powershell -NoProfile -ExecutionPolicy Bypass -File $p -NoPause
#
# If PowerShell blocks this script, copy and run:
#   powershell -NoProfile -ExecutionPolicy Bypass -File ".\install.ps1"

param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:LOCALAPPDATA\wind-cli",
    [switch]$NoPause
)

$ErrorActionPreference = "Stop"
$Repo = "wbyanclaw/wind-cli"
$ExeName = "windcli.exe"
$UserAgent = "windcli-install"

function Write-Title($Text) {
    Write-Host ""
    Write-Host "== $Text ==" -ForegroundColor Cyan
}

function Write-Step($Text) {
    Write-Host "[*] $Text" -ForegroundColor Yellow
}

function Write-Ok($Text) {
    Write-Host "[OK] $Text" -ForegroundColor Green
}

function Write-Fail($Text) {
    Write-Host "[ERROR] $Text" -ForegroundColor Red
}

function Pause-IfNeeded {
    if (-not $NoPause) {
        Write-Host ""
        Read-Host "Press Enter to close"
    }
}

function Exit-Installer($Code) {
    Pause-IfNeeded
    exit $Code
}

function Show-BypassCommand {
    $scriptPath = $PSCommandPath
    if (-not $scriptPath) {
        $scriptPath = ".\install.ps1"
    }

    Write-Host ""
    Write-Host "If PowerShell execution policy blocks this script, copy and run:" -ForegroundColor Yellow
    Write-Host "powershell -NoProfile -ExecutionPolicy Bypass -File `"$scriptPath`"" -ForegroundColor Green
}

try {
    Write-Title "windcli installer"

    if ($PSVersionTable.PSVersion.Major -lt 5) {
        throw "PowerShell 5.0 or newer is required. Install from https://aka.ms/powershell"
    }

    if ($InstallDir -notlike "$env:LOCALAPPDATA*") {
        throw "InstallDir must be under LOCALAPPDATA: $env:LOCALAPPDATA"
    }

    Write-Step "Checking version"
    if ($Version -eq "latest") {
        $releaseApi = "https://api.github.com/repos/$Repo/releases/latest"
        $release = Invoke-RestMethod -Uri $releaseApi -Headers @{ "User-Agent" = $UserAgent }
        $tag = [string]$release.tag_name
        if (-not $tag -or $tag -notmatch '^v\d+\.\d+\.\d+$') {
            throw "GitHub latest release tag is invalid. Expected format like v0.2.1, got '$tag'."
        }
    } else {
        $tag = $Version
        if ($tag -notmatch '^v\d+\.\d+\.\d+$') {
            throw "Version must use format v0.2.1, got '$tag'."
        }
    }
    Write-Ok "Selected $tag"

    $downloadBase = "https://github.com/$Repo/releases/download/$tag"
    $exeUrl = "$downloadBase/$ExeName"
    $sumsUrl = "$downloadBase/SHA256SUMS.txt"
    $destPath = Join-Path $InstallDir $ExeName
    $tempExe = Join-Path $env:TEMP "$ExeName.download"
    $tempSums = Join-Path $env:TEMP "windcli-SHA256SUMS.txt"

    Write-Step "Preparing install path"
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Write-Ok $InstallDir

    Write-Step "Downloading $ExeName"
    Invoke-WebRequest -Uri $exeUrl -OutFile $tempExe -UserAgent $UserAgent
    if (-not (Test-Path $tempExe)) {
        throw "Download failed: $exeUrl"
    }
    Write-Ok "Downloaded $ExeName"

    Write-Step "Downloading checksum"
    Invoke-WebRequest -Uri $sumsUrl -OutFile $tempSums -UserAgent $UserAgent
    $expectedLine = Get-Content $tempSums | Where-Object { $_ -match "\s+$([regex]::Escape($ExeName))$" } | Select-Object -First 1
    if (-not $expectedLine) {
        throw "SHA256SUMS.txt does not contain $ExeName"
    }
    $expectedHash = ($expectedLine -split '\s+')[0].ToLowerInvariant()
    $actualHash = (Get-FileHash $tempExe -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $expectedHash) {
        throw "Checksum mismatch for $ExeName. Expected $expectedHash, got $actualHash."
    }
    Write-Ok "SHA256 verified"

    Write-Step "Installing binary"
    Move-Item -Force -Path $tempExe -Destination $destPath
    Write-Ok $destPath

    Write-Step "Updating user PATH"
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $pathEntries = @()
    if ($currentPath) {
        $pathEntries = $currentPath -split ';' | Where-Object { $_ }
    }
    if ($pathEntries -notcontains $InstallDir) {
        $newPath = if ($currentPath) { "$currentPath;$InstallDir" } else { $InstallDir }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        $env:Path = "$env:Path;$InstallDir"
        Write-Ok "Added $InstallDir to user PATH"
    } else {
        Write-Ok "PATH already contains $InstallDir"
    }

    Write-Step "Verifying installed version"
    $versionOutput = & $destPath --version
    Write-Ok $versionOutput

    Write-Title "Install complete"
    Write-Host "Version: $tag"
    Write-Host "Installed to: $destPath"
    Write-Host "Verify now: `"$destPath`" --version"
    Write-Host "After opening a new terminal: windcli --version"
    Show-BypassCommand
    Exit-Installer 0
} catch {
    Write-Fail $_.Exception.Message
    Show-BypassCommand
    Write-Host ""
    Write-Host "Next step: copy the green command above and run it in PowerShell." -ForegroundColor Yellow
    Exit-Installer 1
}
