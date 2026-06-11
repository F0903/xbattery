$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$packagesConfig = Join-Path $root 'packages.config'
[xml] $packagesXml = Get-Content -Raw -LiteralPath $packagesConfig
$packageNode = $packagesXml.packages.package |
    Where-Object { $_.id -eq 'Microsoft.GameInput' } |
    Select-Object -First 1

if ($null -eq $packageNode) {
    throw "$packagesConfig must contain package id Microsoft.GameInput"
}

$version = $packageNode.version
$redist = Join-Path $root "packages\Microsoft.GameInput.$version\redist\GameInputRedist.msi"
$log = Join-Path $root "target\gameinput-redist-install.log"
$principal = [Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
$isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    throw "GameInput redistributable installation requires an elevated PowerShell session."
}

if (-not (Test-Path -LiteralPath $redist)) {
    & (Join-Path $PSScriptRoot 'sync-gameinput.ps1')
}

if (-not (Test-Path -LiteralPath $redist)) {
    throw "GameInput redistributable was not found: $redist"
}

$process = Start-Process -FilePath 'msiexec.exe' -ArgumentList @(
    '/i',
    "`"$redist`"",
    '/quiet',
    '/norestart',
    '/L*v',
    "`"$log`""
) -Wait -PassThru

if ($process.ExitCode -ne 0) {
    throw "GameInput redist install failed with exit code $($process.ExitCode). Log: $log"
}

Write-Host "Installed GameInput redistributable $version"
