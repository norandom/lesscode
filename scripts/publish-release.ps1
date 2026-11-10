#Requires -Version 5.1
<#
.SYNOPSIS
Signs the built release archives and writes the checksum files.

.DESCRIPTION
Called by .github/workflows/release.yml in the approval-gated `release`
environment. -Directory holds the platform archives (lesscode-core-*.tar.gz,
lesscode-core-*.zip) and the install scripts, and nothing else that should
not be published.

Order (the same as mmhmem):
  1. sign every archive in place (context = its published file name) and
     verify each signature;
  2. write <archive>.sha256 for the installers and SHA256SUMS over every
     published file, computed after signing;
  3. sign SHA256SUMS as SHA256SUMS.sig and verify it.

Fails closed: no key, a wrong zipsign version, or a key that is not listed in
scripts/release-public-keys.txt stops the release. The private key comes from
LESSCODE_RELEASE_SIGNING_KEY and is never printed.
#>
param(
    [Parameter(Mandatory = $true)][string]$Directory,
    [string[]]$ExpectedArchives = @(
        'lesscode-core-aarch64-apple-darwin.tar.gz',
        'lesscode-core-aarch64-unknown-linux-gnu.tar.gz',
        'lesscode-core-x86_64-apple-darwin.tar.gz',
        'lesscode-core-x86_64-pc-windows-msvc.zip',
        'lesscode-core-x86_64-unknown-linux-gnu.tar.gz'
    ),
    [string]$KeyList = (Join-Path $PSScriptRoot 'release-public-keys.txt')
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'sign-release-assets.ps1')

$Directory = (Resolve-Path -LiteralPath $Directory).Path
$archives = @(foreach ($name in $ExpectedArchives) {
        $path = Join-Path $Directory $name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing release archive: $name" }
        $path
    })
foreach ($stale in 'SHA256SUMS', 'SHA256SUMS.sig') {
    $path = Join-Path $Directory $stale
    if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
}
Get-ChildItem -LiteralPath $Directory -Filter '*.sha256' -File | Remove-Item -Force

function Get-Sha256Hex([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}
function Write-LfText([string]$Path, [string[]]$Lines) {
    [IO.File]::WriteAllText($Path, (($Lines -join "`n") + "`n"), (New-Object Text.UTF8Encoding $false))
}

$keyDir = Join-Path ([IO.Path]::GetTempPath()) ('lesscode-signing-' + [guid]::NewGuid().ToString('N'))
$signing = $null
try {
    $signing = Open-ReleaseSigning -Directory $keyDir -EmbeddedSource $KeyList
    Invoke-ReleaseSigning $archives $signing
    Write-Host "Signed $($archives.Count) archives with key $($signing.PublicKeyHex)"

    foreach ($archive in $archives) {
        $name = Split-Path -Leaf $archive
        Write-LfText "$archive.sha256" @("$(Get-Sha256Hex $archive)  $name")
    }
    $published = @(Get-ChildItem -LiteralPath $Directory -File | Sort-Object Name)
    Write-LfText (Join-Path $Directory 'SHA256SUMS') @($published | ForEach-Object { "$(Get-Sha256Hex $_.FullName)  $($_.Name)" })
    $null = Write-ReleaseChecksumSignature $Directory $signing
    Write-Host "Wrote SHA256SUMS over $($published.Count) files and SHA256SUMS.sig"
} finally {
    Close-ReleaseSigning $signing
    Remove-Item -LiteralPath $keyDir -Recurse -Force -ErrorAction SilentlyContinue
}
