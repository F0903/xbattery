param(
    [switch] $Update,
    [string] $Version
)

$ErrorActionPreference = 'Stop'

$packageId = 'Microsoft.GameInput'
$nugetSource = 'https://api.nuget.org/v3/index.json'
$root = Split-Path -Parent $PSScriptRoot
$packagesDir = Join-Path $root 'packages'
$packagesConfig = Join-Path $root 'packages.config'
$nugetConfig = Join-Path $root 'nuget.config'

function Get-NuGetExe {
    $machinePath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $env:Path = @($env:Path, $machinePath, $userPath) -join ';'

    $nuget = Get-Command nuget.exe -ErrorAction SilentlyContinue
    if ($null -ne $nuget) {
        return $nuget.Source
    }

    if (Test-Path -LiteralPath 'C:\nuget\nuget.exe') {
        return 'C:\nuget\nuget.exe'
    }

    throw "nuget.exe was not found on PATH. Install the NuGet CLI from https://www.nuget.org/downloads and rerun this script."
}

function Get-PackagesConfig {
    [xml] $xml = Get-Content -Raw -LiteralPath $packagesConfig
    return $xml
}

function Get-GameInputPackageNode {
    param([xml] $PackagesXml)

    $node = $PackagesXml.packages.package |
        Where-Object { $_.id -eq $packageId } |
        Select-Object -First 1

    if ($null -eq $node) {
        throw "$packagesConfig must contain package id $packageId"
    }

    return $node
}

function Get-LatestGameInputVersion {
    param([string] $NuGetExe)

    $output = & $NuGetExe search $packageId `
        -Source $nugetSource `
        -Take 10 `
        -NonInteractive

    if ($LASTEXITCODE -ne 0) {
        throw "nuget.exe search failed with exit code $LASTEXITCODE"
    }

    foreach ($line in $output) {
        if ($line -match '^>\s+Microsoft\.GameInput\s+\|\s+([^| ]+)') {
            return $Matches[1]
        }
    }

    throw "Could not find latest $packageId version in NuGet search output."
}

$nugetExe = Get-NuGetExe
$packagesXml = Get-PackagesConfig
$packageNode = Get-GameInputPackageNode -PackagesXml $packagesXml
$currentVersion = $packageNode.version
$targetVersion = $currentVersion

if ($Version) {
    $targetVersion = $Version
} elseif ($Update) {
    $targetVersion = Get-LatestGameInputVersion -NuGetExe $nugetExe
}

if ($targetVersion -ne $currentVersion) {
    $packageNode.SetAttribute('version', $targetVersion)
    $packagesXml.Save($packagesConfig)
    Write-Host "Updated $packageId from $currentVersion to $targetVersion"
}

& $nugetExe restore $packagesConfig `
    -PackagesDirectory $packagesDir `
    -ConfigFile $nugetConfig `
    -NonInteractive `
    -Verbosity quiet

if ($LASTEXITCODE -ne 0) {
    throw "nuget.exe restore failed with exit code $LASTEXITCODE"
}

$packageDir = Join-Path $packagesDir "$packageId.$targetVersion"
$lib = Join-Path $packageDir 'native\lib\x64\GameInput.lib'
$redist = Join-Path $packageDir 'redist\GameInputRedist.msi'

if (-not (Test-Path -LiteralPath $lib)) {
    throw "Expected native library was not found: $lib"
}

if (-not (Test-Path -LiteralPath $redist)) {
    throw "Expected redistributable was not found: $redist"
}

Write-Host "Restored $packageId $targetVersion"
Write-Host "Native lib: $lib"
Write-Host "Redist MSI: $redist"
