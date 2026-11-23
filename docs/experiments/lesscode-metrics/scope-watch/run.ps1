# R170: add the watching statements to a copy of drift-rdp's renderer, run the project's own
# tests, and collect what they noted.
#   ./run.ps1 -Watch <scope-watch.exe> -Out <new directory under D:\BuildArtifacts\lesscode> [-Tree <existing copy>]
# drift-rdp is only read. With -Tree the copy's renderer sources are put back first.
param([Parameter(Mandatory)][string]$Watch, [Parameter(Mandatory)][string]$Out, [string]$Tree, [int]$LimitSeconds = 900)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $Out) { throw "$Out exists" }
if ($Out -notmatch '^D:\\BuildArtifacts\\lesscode\\') { throw 'output must be under D:\BuildArtifacts\lesscode' }
$rules = Join-Path $PSScriptRoot 'rust.watch.json'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
$subject = (Resolve-Path (Join-Path $repo '..\drift-rdp')).Path
$files = 'display_path.rs', 'frame_stores.rs', 'gpu.rs', 'native_layer.rs', 'native_layer/dcomp.rs', 'shared_frame.rs', 'yuv.rs'
New-Item -ItemType Directory -Force $Out | Out-Null
if (-not $Tree) {
    $Tree = Join-Path $Out 'tree'
    New-Item -ItemType Directory -Force $Tree | Out-Null
    robocopy $subject $Tree /E /XD .git node_modules target /NFL /NDL /NJH /NJS /NP | Out-Null
    if ($LASTEXITCODE -ge 8) { throw "copy failed: robocopy $LASTEXITCODE" }
}
elseif ($Tree -notmatch '^D:\\BuildArtifacts\\lesscode\\') { throw 'the copy must be under D:\BuildArtifacts\lesscode' }
$renderer = Join-Path $Tree 'src-tauri\src\renderer'
if (Test-Path -LiteralPath $renderer) { Remove-Item -LiteralPath $renderer -Recurse -Force -Confirm:$false }
Copy-Item (Join-Path $subject 'src-tauri\src\renderer') $renderer -Recurse
$log = [ordered]@{ started = (Get-Date -Format o); watch = (Get-FileHash $Watch).Hash; rules = (Get-FileHash $rules).Hash
    subject = (git -C $subject rev-parse HEAD); subject_dirty = @(git -C $subject status --porcelain -- src-tauri); tree = $Tree }
$env:CARGO_TARGET_DIR = Join-Path $Tree 'target'
$manifest = Join-Path $Tree 'src-tauri\Cargo.toml'
function Finish([string]$Outcome) {
    $log.outcome = $Outcome; $log.ended = (Get-Date -Format o)
    $log | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Out 'run.json') -Encoding utf8NoBOM
    $Outcome
}
function Tests([string]$Name) {
    $p = Start-Process cargo -ArgumentList 'nextest', 'run', '--locked', '--manifest-path', $manifest, '--lib', '--no-fail-fast' `
        -WorkingDirectory $Tree -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $Out "$Name.out") -RedirectStandardError (Join-Path $Out "$Name.err")
    if (-not $p.WaitForExit($LimitSeconds * 1000)) { taskkill /PID $p.Id /T /F | Out-Null; return 'did not end' }
    $summary = Select-String -Path (Join-Path $Out "$Name.err"), (Join-Path $Out "$Name.out") -Pattern '^\s*Summary ' | Select-Object -Last 1
    return "exit $($p.ExitCode): " + $(if ($summary) { $summary.Line.Trim() } else { 'no summary line' })
}

# 1. The tests as they are, in this copy.
$log.tests_unchanged = Tests 'tests-unchanged'

# 2. The statements and the modules.
$report = Join-Path $Out 'gen.json'
& $Watch instrument $rules $renderer 'crate::renderer' @files > $report 2> (Join-Path $Out 'gen.err')
if ($LASTEXITCODE -ne 0) { Finish 'not generated'; exit 2 }
$made = Get-Content $report -Raw | ConvertFrom-Json
Add-Content (Join-Path $renderer 'mod.rs') ("`n" + $made.collector.declaration) -Encoding utf8NoBOM
Copy-Item $renderer (Join-Path $Out 'renderer-with-statements') -Recurse

# 3. The same tests, watched.
$noted = Join-Path $Out 'noted'
New-Item -ItemType Directory -Force $noted | Out-Null
$env:LESSCODE_WATCH_OUT = $noted
$log.tests_watched = Tests 'tests-watched'
Remove-Item Env:\LESSCODE_WATCH_OUT
$log.build_failed = [bool](Select-String -Path (Join-Path $Out 'tests-watched.err') -Pattern 'could not compile' -Quiet)
if ($log.build_failed) { Finish 'does not compile'; exit 1 }

# 4. What was noted.
& $Watch collect $report $noted > (Join-Path $Out 'seen.json') 2> (Join-Path $Out 'collect.err')
$log.collect_exit = $LASTEXITCODE
Finish 'watched'
