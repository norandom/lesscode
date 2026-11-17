param([string]$Controls, [string]$Output)
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath($Output)
$allowed = 'D:\BuildArtifacts\lesscode\model-simplification\ms1\'
if (-not $root.StartsWith($allowed) -or (Test-Path -LiteralPath $root)) { throw 'Require new experiment output' }
$data = Get-Content -LiteralPath $Controls -Raw | ConvertFrom-Json
New-Item -ItemType Directory -Path $root | Out-Null
Copy-Item -LiteralPath $Controls -Destination "$root\controls.json"
$archive = Join-Path $root 'baseline.tar'
git -C $data.repository archive --format=tar "--output=$archive" $data.revision
if ($LASTEXITCODE -ne 0) { throw 'Cannot archive frozen private revision' }
foreach ($control in $data.controls) {
    if ($control.id -notmatch '^[a-z0-9-]+$') { throw 'Invalid control ID' }
    $copy = Join-Path $root $control.id
    New-Item -ItemType Directory -Path $copy | Out-Null
    tar.exe -xf $archive -C $copy
    if ($LASTEXITCODE -ne 0) { throw 'Cannot extract snapshot' }
    if ($control.file) {
        $path = [IO.Path]::GetFullPath((Join-Path $copy $control.file))
        if (-not $path.StartsWith($copy + '\')) { throw 'Control escapes snapshot' }
        $source = [IO.File]::ReadAllText($path)
        $newline = if ($source.Contains("`r`n")) { "`r`n" } else { "`n" }
        $before = $control.before.Replace("`r`n", "`n").Replace("`n", $newline)
        $after = $control.after.Replace("`r`n", "`n").Replace("`n", $newline)
        if (($source.Split([string[]]@($before), [StringSplitOptions]::None).Count - 1) -ne 1) {
            throw "Control $($control.id) needs exactly one matching location"
        }
        [IO.File]::WriteAllText($path, $source.Replace($before, $after))
    }
    Push-Location -LiteralPath $copy
    try {
        & pwsh.exe -NoProfile -File "$PSScriptRoot\run-tests.ps1" -LimitSeconds 1200 2>&1 |
            Tee-Object -FilePath (Join-Path $root "$($control.id).log")
        $code = $LASTEXITCODE
    } finally { Pop-Location }
    if ($code -ne $control.expected_exit) { throw "Unexpected control $($control.id) exit $code" }
    "$($control.id): expected exit $code" | Add-Content -LiteralPath "$root\results.txt"
}
