<#
.SYNOPSIS
Write LICENSES.md and THIRD_PARTY_NOTICES.txt of one crate from its locked
Cargo metadata.

.DESCRIPTION
Every package of the lock file that comes from a registry gets a row in
LICENSES.md and a section in THIRD_PARTY_NOTICES.txt with the license files
its published archive holds. A package that publishes no license file takes
the texts kept under licenses/ of the crate, or of a crate of this repository
it depends on, where a file `<prefix>-<WORD>.txt` serves every package whose
name starts with <prefix>; the longest prefix decides. A package with neither
stops the run, and so does one that states no license.

A file of another project that is kept in the repository and taken into the
build is listed in bundled.json of its crate, with its source address, its
SHA-256 and its license files. It gets a row and a section too. A file whose
hash is not the stated one stops the run.

With -Check nothing is written; the run fails when a file would change.

.EXAMPLE
./scripts/license-inventory.ps1 -Crate model
#>
param(
    [Parameter(Mandatory = $true)][string]$Crate,
    [string]$Toolchain = '1.98.1',
    [switch]$Check
)
$ErrorActionPreference = 'Stop'
$Crate = (Resolve-Path -LiteralPath $Crate).Path
$manifest = Join-Path $Crate 'Cargo.toml'

$metadata = cargo "+$Toolchain" metadata --locked --format-version 1 --manifest-path $manifest
if ($LASTEXITCODE -ne 0) { throw "cargo metadata failed for $manifest" }
$all = ($metadata | ConvertFrom-Json -AsHashtable).packages
$packages = @($all | Where-Object { $_.source } |
        Sort-Object -Property @{ Expression = { $_.name.ToLowerInvariant() } }, version)

# Kept texts: licenses/ of this crate and of every crate of the repository it depends on.
$kept = @($all | Where-Object { -not $_.source } | ForEach-Object {
        Get-ChildItem -LiteralPath (Join-Path (Split-Path $_.manifest_path) 'licenses') -Filter '*.txt' -File -ErrorAction SilentlyContinue
    })
function Get-LicenseFiles($package) {
    $own = @(Get-ChildItem -LiteralPath (Split-Path $package.manifest_path) -File |
            Where-Object { $_.Name -match '^(LICEN[CS]E|COPYING|COPYRIGHT|NOTICE|UNLICENSE)' } |
            Sort-Object -Property Name -Culture '')
    if ($own) { return $own }
    # The longest prefix decides: jni-sys-macros-MIT.txt before jni-MIT.txt.
    $fitting = @($kept | Where-Object { $package.name.StartsWith(($_.BaseName -replace '-[^-]+$', '')) } |
            Group-Object { ($_.BaseName -replace '-[^-]+$', '').Length } | Sort-Object { [int]$_.Name } -Descending)
    if ($fitting) { @($fitting[0].Group | Sort-Object -Property Name -Culture '') }
}

$rows = [Collections.Generic.List[string]]::new()
$notices = [Text.StringBuilder]::new()
$problems = [Collections.Generic.List[string]]::new()
foreach ($package in $packages) {
    $label = "$($package.name) $($package.version)"
    if (-not $package.license) { $problems.Add("$label states no license"); continue }
    $files = Get-LicenseFiles $package
    if (-not $files) { $problems.Add("$label publishes no license file and licenses/ has none for it"); continue }
    $rows.Add("| $($package.name) | $($package.version) | $($package.license) |")
    [void]$notices.Append("========== $label [$($package.license)] ==========`n")
    foreach ($file in $files) {
        $text = [IO.File]::ReadAllText($file.FullName) -replace "`r`n", "`n"
        [void]$notices.Append("--- $($file.Name) ---`n$text`n")
    }
}

# Files of other projects kept in the repository and taken into the build:
# bundled.json of this crate and of every crate of the repository it depends on.
$bundled = [Collections.Generic.List[string]]::new()
foreach ($local in @($all | Where-Object { -not $_.source } | Sort-Object -Property name)) {
    $dir = Split-Path $local.manifest_path
    $list = Join-Path $dir 'bundled.json'
    if (-not (Test-Path -LiteralPath $list)) { continue }
    foreach ($item in @(Get-Content -LiteralPath $list -Raw | ConvertFrom-Json)) {
        $label = "$($item.name) $($item.version)"
        $hash = (Get-FileHash -LiteralPath (Join-Path $dir $item.file) -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($hash -cne $item.sha256) { $problems.Add("${label}: $($item.file) has the hash $hash, bundled.json states $($item.sha256)"); continue }
        if (-not $item.license -or -not $item.license_files) { $problems.Add("$label states no license or no license file"); continue }
        $bundled.Add("| $($item.name) | $($item.version) | $($item.license) | $($item.file) of $($local.name) | $($item.source) | ``$hash`` |")
        [void]$notices.Append("========== $label [$($item.license)] ==========`n")
        [void]$notices.Append("File: $($item.file) of $($local.name)`nSource: $($item.source)`nSHA-256: $hash`n$($item.note)`n")
        foreach ($name in $item.license_files) {
            $text = [IO.File]::ReadAllText((Join-Path $dir $name)) -replace "`r`n", "`n"
            [void]$notices.Append("--- $(Split-Path $name -Leaf) ---`n$text`n")
        }
    }
}
if ($problems) { throw ("License inventory of ${Crate}:`n" + ($problems -join "`n")) }
if ($bundled) {
    $heading = @(
        ''
        '## Files kept in the repository and taken into the build'
        ''
        'Listed in `bundled.json` of the crate that holds them. The hash is checked when this inventory is written, and by a test of that crate.'
        ''
        '| Component | Version | SPDX expression | File | Source | SHA-256 |'
        '| --- | --- | --- | --- | --- | --- |'
    )
    foreach ($line in $heading + $bundled) { $rows.Add($line) }
}

$inventory = @(
    '# Third-party license inventory'
    ''
    'Generated from locked Cargo metadata; includes target-specific and optional locked records. Full license texts are retained in THIRD_PARTY_NOTICES.txt and licenses/. Lesscode project code is licensed under the [MIT License](../LICENSE); third-party components retain their respective licenses.'
    ''
    '| Package | Version | SPDX expression |'
    '| --- | --- | --- |'
) + $rows
$written = @{
    'LICENSES.md'             = ($inventory -join "`n") + "`n"
    'THIRD_PARTY_NOTICES.txt' = $notices.ToString()
}
$changed = @(foreach ($name in $written.Keys | Sort-Object) {
        $path = Join-Path $Crate $name
        $before = if (Test-Path -LiteralPath $path) { [IO.File]::ReadAllText($path) -replace "`r`n", "`n" } else { '' }
        if ($before -cne $written[$name]) {
            if (-not $Check) { [IO.File]::WriteAllText($path, $written[$name], [Text.UTF8Encoding]::new($false)) }
            $name
        }
    })
"$($packages.Count) packages; changed: $(if ($changed) { $changed -join ', ' } else { 'nothing' })"
if ($Check -and $changed) { throw "License inventory of $Crate is out of date: run scripts/license-inventory.ps1 -Crate $Crate" }
