param([string]$Manifest, [string]$ExpectedHash)
$ErrorActionPreference = 'Stop'
$path = (Resolve-Path -LiteralPath $Manifest).Path
if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $ExpectedHash) { throw 'Freeze digest mismatch' }
$root = Split-Path -Parent $path
$data = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
if ($data.schema -ne 'h-ms1-freeze-v1' -or $data.inputs.Count -eq 0) { throw 'Missing freeze inputs' }
foreach ($inputFile in $data.inputs) {
    $target = [IO.Path]::GetFullPath((Join-Path $root $inputFile.path))
    if (-not $target.StartsWith($root + '\')) { throw 'Input path escapes freeze' }
    if ((Get-Item -LiteralPath $target).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Reparse input' }
    if ((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash -ne $inputFile.sha256) {
        throw "Frozen input changed: $($inputFile.path)"
    }
}
foreach ($item in (Get-Content -LiteralPath "$root\metadata\evidence-index.json" -Raw | ConvertFrom-Json)) {
    if (-not $item.path.StartsWith('D:\BuildArtifacts\lesscode\model-simplification\ms1\')) { throw 'Evidence path escapes experiment' }
    if ((Get-FileHash -LiteralPath $item.path -Algorithm SHA256).Hash -ne $item.sha256) {
        throw "Retained evidence changed: $($item.path)"
    }
}
"Verified $($data.inputs.Count) frozen inputs"
