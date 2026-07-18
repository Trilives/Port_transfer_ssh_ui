param(
    [Parameter(Mandatory = $true)]
    [string]$Executable,
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [Parameter(Mandatory = $true)]
    [string]$OutputDirectory
)

$ErrorActionPreference = "Stop"
$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$sourceExe = [System.IO.Path]::GetFullPath($Executable)
$portableReadme = Join-Path $repoRoot "packaging\PORTABLE-README.txt"
$license = Join-Path $repoRoot "LICENSE"
$outputRoot = [System.IO.Path]::GetFullPath($OutputDirectory)

if (-not (Test-Path -LiteralPath $sourceExe -PathType Leaf)) {
    throw "Executable not found: $sourceExe"
}
if (-not (Test-Path -LiteralPath $portableReadme -PathType Leaf)) {
    throw "Portable README not found: $portableReadme"
}
if (-not (Test-Path -LiteralPath $license -PathType Leaf)) {
    throw "License not found: $license"
}

$tempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
$staging = [System.IO.Path]::GetFullPath((Join-Path $tempRoot ("sshdeck-portable-" + [guid]::NewGuid().ToString("N"))))
if (-not $staging.StartsWith($tempRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Unsafe portable staging path: $staging"
}

New-Item -ItemType Directory -Path $staging | Out-Null
try {
    Copy-Item -LiteralPath $sourceExe -Destination (Join-Path $staging "SSHDeck.exe")
    Copy-Item -LiteralPath $portableReadme -Destination (Join-Path $staging "README.txt")
    Copy-Item -LiteralPath $license -Destination (Join-Path $staging "LICENSE.txt")
    New-Item -ItemType File -Path (Join-Path $staging "portable.flag") | Out-Null

    New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
    $archive = Join-Path $outputRoot "SSHDeck_${Version}_x64-portable.zip"
    if (Test-Path -LiteralPath $archive) {
        Remove-Item -LiteralPath $archive
    }
    Compress-Archive -Path (Join-Path $staging "*") -DestinationPath $archive -CompressionLevel Optimal
    Write-Output $archive
}
finally {
    if (Test-Path -LiteralPath $staging) {
        Remove-Item -LiteralPath $staging -Recurse -Force
    }
}
