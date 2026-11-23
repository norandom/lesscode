# R168: generate a walk for each struct with holders in the renderer of drift-rdp and run it.
#   ./run-renderer.ps1 -Gen <walk-gen.exe> -Out <new directory under D:\BuildArtifacts\lesscode>
# Nothing generated is edited. drift-rdp is only read; everything runs in a copy.
param([Parameter(Mandatory)][string]$Gen, [Parameter(Mandatory)][string]$Out, [int]$LimitSeconds = 180)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $Out) { throw "$Out exists" }
if ($Out -notmatch '^D:\\BuildArtifacts\\lesscode\\') { throw 'output must be under D:\BuildArtifacts\lesscode' }
$rules = Join-Path $PSScriptRoot 'rust.walk.json'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
$subject = (Resolve-Path (Join-Path $repo '..\drift-rdp')).Path
$types = @(
    @('Composition', 'native_layer/dcomp.rs'), @('Damage', 'shared_frame.rs'), @('DisplayPathInfo', 'display_path.rs'),
    @('Facts', 'display_path.rs'), @('FrameStores', 'frame_stores.rs'), @('Layer', 'native_layer.rs'),
    @('LayerTarget', 'native_layer.rs'), @('Params', 'gpu.rs'), @('RenderShared', 'gpu.rs'), @('Renderer', 'gpu.rs'),
    @('SharedFrame', 'shared_frame.rs'), @('SharedFrameInner', 'shared_frame.rs'), @('State', 'native_layer.rs'),
    @('View', 'gpu.rs'), @('Wake', 'native_layer.rs'), @('YuvFrame', 'yuv.rs'))

$tree = Join-Path $Out 'tree'
$results = Join-Path $Out 'reached'
New-Item -ItemType Directory -Force $tree, $results, (Join-Path $Out 'modules'), (Join-Path $Out 'logs') | Out-Null
robocopy $subject $tree /E /XD .git node_modules target /NFL /NDL /NJH /NJS /NP | Out-Null
if ($LASTEXITCODE -ge 8) { throw "copy failed: robocopy $LASTEXITCODE" }
$started = Get-Date -Format o
$env:CARGO_TARGET_DIR = Join-Path $tree 'target'
$env:LESSCODE_WALK_OUT = $results
$manifest = Join-Path $tree 'src-tauri\Cargo.toml'
$renderer = Join-Path $tree 'src-tauri\src\renderer'

function Build([string]$Log) {
    Push-Location $tree
    try { cargo test --locked --manifest-path src-tauri/Cargo.toml --lib --no-run *> $Log } finally { Pop-Location }
    return $LASTEXITCODE
}

# The project as it is must build first; its time is the cost of every later build.
$clock = [Diagnostics.Stopwatch]::StartNew()
$base = Build (Join-Path $Out 'logs\build-unchanged.log')
$baseSeconds = [int]$clock.Elapsed.TotalSeconds
if ($base -ne 0) { throw 'the unchanged copy does not build' }

$rows = foreach ($t in $types) {
    $name, $relative = $t
    $module = 'state_walk_' + $name.ToLower()
    $file = Join-Path $renderer $relative
    $beside = Join-Path (Split-Path $file) ([IO.Path]::GetFileNameWithoutExtension($file))
    $target = Join-Path $beside "$module.rs"
    New-Item -ItemType Directory -Force $beside | Out-Null
    $row = [ordered]@{ type = $name; file = $relative; outcome = ''; detail = '' }
    $report = Join-Path $Out "logs\$name-gen.json"
    & $Gen $rules $file $name $target > $report 2> (Join-Path $Out "logs\$name-gen.err")
    if ($LASTEXITCODE -ne 0) {
        $row.outcome = 'not generated'
        $row.detail = (Get-Content (Join-Path $Out "logs\$name-gen.err") -Raw).Trim()
        [pscustomobject]$row; continue
    }
    Copy-Item $target (Join-Path $Out "modules\$module.rs")
    $gen = Get-Content $report -Raw | ConvertFrom-Json
    $row.calls = $gen.calls.own + $gen.calls.of_what_they_return + $gen.calls.test_only
    $row.holders = @($gen.holders).Count
    $row.not_called = @($gen.methods_not_called).Count
    $row.length = $gen.length
    $original = Get-Content $file -Raw
    Add-Content $file "`n#[cfg(test)]`nmod $module;" -Encoding utf8NoBOM
    $log = Join-Path $Out "logs\$name-build.log"
    if ((Build $log) -ne 0) {
        Set-Content $file $original -NoNewline -Encoding utf8NoBOM
        Remove-Item -LiteralPath $target
        $row.outcome = 'does not compile'
        $row.detail = (Select-String -Path $log -Pattern '^error' | Select-Object -First 3 | ForEach-Object Line) -join ' | '
        [pscustomobject]$row; continue
    }
    $row.outcome = 'compiled'
    [pscustomobject]$row
}

# Each walk by itself, with a limit.
foreach ($row in $rows | Where-Object outcome -eq 'compiled') {
    $module = 'state_walk_' + $row.type.ToLower()
    $log = Join-Path $Out "logs\$($row.type)-run"
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process cargo -ArgumentList 'test', '--locked', '--manifest-path', $manifest, '--lib', "$($module)::" `
        -WorkingDirectory $tree -NoNewWindow -PassThru -RedirectStandardOutput "$log.out" -RedirectStandardError "$log.err"
    if (-not $p.WaitForExit($LimitSeconds * 1000)) {
        taskkill /PID $p.Id /T /F | Out-Null
        $row.outcome = 'does not finish'; $row.detail = "stopped after $LimitSeconds seconds"
        continue
    }
    $row | Add-Member seconds ([int]$clock.Elapsed.TotalSeconds) -Force
    $result = Join-Path $results "$($row.type).json"
    if ($p.ExitCode -ne 0 -or -not (Test-Path $result)) {
        $row.outcome = 'does not finish'
        $row.detail = "test exit $($p.ExitCode): " + ((Select-String -Path "$log.out", "$log.err" -Pattern 'panicked|error' | Select-Object -First 2 | ForEach-Object Line) -join ' | ')
        continue
    }
    $sets = (Get-Content $result -Raw | ConvertFrom-Json).sets
    $row | Add-Member reached (($sets | ForEach-Object reached) -join ', ') -Force
    $row | Add-Member sequences $sets[-1].sequences -Force
    $row.outcome = if ($row.calls -gt 0) { 'walked' } else { 'walked, no calls' }
}

@{
    started = $started; ended = (Get-Date -Format o)
    gen = (Get-FileHash $Gen).Hash; rules = (Get-FileHash $rules).Hash
    subject = (git -C $subject rev-parse HEAD); subject_dirty = @(git -C $subject status --porcelain -- src-tauri)
    seconds_to_build_unchanged = $baseSeconds; limit_seconds = $LimitSeconds
    rows = @($rows)
} | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Out 'run.json') -Encoding utf8NoBOM
