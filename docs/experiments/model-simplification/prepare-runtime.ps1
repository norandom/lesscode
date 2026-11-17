param(
    [string]$Specimen = 'D:\BuildArtifacts\lesscode\steps-foreign\x_likes_scraper',
    [string]$Output = 'D:\BuildArtifacts\lesscode\model-simplification\ms1\runtime',
    [string]$Container = 'lesscode-ms1-prep'
)
$ErrorActionPreference = 'Stop'
$sourceRoot = (Resolve-Path -LiteralPath $Specimen).Path
$outputRoot = [IO.Path]::GetFullPath($Output)
$allowedRoot = 'D:\BuildArtifacts\lesscode\model-simplification\'
if (-not $outputRoot.StartsWith($allowedRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Output must remain under the experiment artifact directory'
}
if ($outputRoot.StartsWith($sourceRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Output overlaps source'
}
function Invoke-Checked([string]$Program, [string[]]$Arguments) {
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed ($LASTEXITCODE)" }
}
$revision = '3a214ff13ee768535481f71258ac2b3529306870'
$image = 'ghcr.io/astral-sh/uv@sha256:60936acc2dfbd8db1ebdc2e841000aeee1a86394df6a09f96eb99d45a36038c1'
New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
New-Item -ItemType Directory -Force -Path "$outputRoot\work" | Out-Null
Invoke-Checked git @('-C', $sourceRoot, 'archive', '--format=tar', "--output=$outputRoot\source.tar", $revision)
# An existing container is deliberately an error: preserve earlier evidence.
Invoke-Checked podman @('create', '--name', $Container, '--workdir', '/work', $image, 'sleep', 'infinity')
Invoke-Checked podman @('cp', "$outputRoot\work", "${Container}:/work")
Invoke-Checked podman @('cp', "$outputRoot\source.tar", "${Container}:/tmp/source.tar")
Invoke-Checked podman @('start', $Container)
Invoke-Checked podman @('exec', $Container, 'tar', '-xf', '/tmp/source.tar', '-C', '/work')
Invoke-Checked podman @('exec', $Container, 'uv', 'sync', '--frozen', '--group', 'dev')
Invoke-Checked podman @('exec', $Container, 'uv', 'pip', 'install', '--python', '/work/.venv/bin/python', 'pytest-xdist==3.8.0', 'execnet==2.1.2')
Invoke-Checked podman @('exec', $Container, 'uv', 'pip', 'freeze', '--python', '/work/.venv/bin/python') |
    Set-Content -LiteralPath "$outputRoot\installed.txt" -Encoding utf8
Invoke-Checked podman @('exec', $Container, 'sha256sum', '/work/uv.lock', '/work/pyproject.toml', '/usr/local/bin/python3.12', '/usr/local/bin/uv') |
    Set-Content -LiteralPath "$outputRoot\runtime-hashes.txt" -Encoding utf8
Invoke-Checked podman @('commit', $Container, 'localhost/lesscode-ms1-runner:prepared') |
    Set-Content -LiteralPath "$outputRoot\image-id.txt" -Encoding utf8
Invoke-Checked podman @('stop', $Container)
# This prepares dependencies only. Later tests require --network none and the full freeze.
