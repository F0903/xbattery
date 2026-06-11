param(
    [Parameter(Mandatory = $true)]
    [string] $FilePath,

    [string] $ArgumentList = ''
)

$ErrorActionPreference = 'Stop'

$principal = [Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
$isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($isAdmin) {
    $process = Start-Process -FilePath $FilePath -ArgumentList $ArgumentList -Wait -PassThru
} else {
    $process = Start-Process -FilePath $FilePath -ArgumentList $ArgumentList -Verb RunAs -Wait -PassThru
}

if ($process.ExitCode -ne 0) {
    throw "Elevated command failed with exit code $($process.ExitCode): $FilePath $ArgumentList"
}
