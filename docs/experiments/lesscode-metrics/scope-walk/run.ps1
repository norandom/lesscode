# R169: generate the walk over the renderer of drift-rdp and run it, as generated.
#   ./run.ps1 -Gen <scope-walk.exe> -Out <new directory under D:\BuildArtifacts\lesscode> [-Tree <copy made by an earlier attempt>]
# drift-rdp is only read. With -Tree the copy's renderer sources are put back as they are in
# drift-rdp before anything is generated, and its build output is used again.
param([Parameter(Mandatory)][string]$Gen, [Parameter(Mandatory)][string]$Out, [string]$Tree, [int]$LimitSeconds = 420, [int]$Retries = 8)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $Out) { throw "$Out exists" }
if ($Out -notmatch '^D:\\BuildArtifacts\\lesscode\\') { throw 'output must be under D:\BuildArtifacts\lesscode' }
$rules = Join-Path $PSScriptRoot 'rust.scope.json'
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
$log = [ordered]@{ started = (Get-Date -Format o); gen = (Get-FileHash $Gen).Hash; rules = (Get-FileHash $rules).Hash
    subject = (git -C $subject rev-parse HEAD); subject_dirty = @(git -C $subject status --porcelain -- src-tauri); tree = $Tree }
$env:CARGO_TARGET_DIR = Join-Path $Tree 'target'
$manifest = Join-Path $Tree 'src-tauri\Cargo.toml'
function Finish([string]$Outcome) {
    $log.outcome = $Outcome; $log.ended = (Get-Date -Format o)
    $log | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $Out 'run.json') -Encoding utf8NoBOM
    $Outcome
}

# 1. Generate.
$report = Join-Path $Out 'gen.json'
& $Gen $rules $renderer 'crate::renderer' @files > $report 2> (Join-Path $Out 'gen.err')
if ($LASTEXITCODE -ne 0) { Finish 'not generated'; exit 2 }
$made = Get-Content $report -Raw | ConvertFrom-Json
New-Item -ItemType Directory -Force (Join-Path $Out 'modules') | Out-Null
foreach ($part in $made.parts) {
    Copy-Item $part.module (Join-Path $Out ('modules\' + ($part.file -replace '[/\\]', '_') + '.state_walk_part.rs'))
    Add-Content (Join-Path $renderer $part.declare_in) ("`n" + $part.declaration) -Encoding utf8NoBOM
}
Copy-Item $made.walk_module.module (Join-Path $Out 'modules\state_walk.rs')
Add-Content (Join-Path $renderer 'mod.rs') ("`n" + $made.walk_module.declaration) -Encoding utf8NoBOM

# 2. Build the tests with the modules in.
Push-Location $Tree
try { cargo test --locked --manifest-path src-tauri/Cargo.toml --lib --no-run *> (Join-Path $Out 'build.log') } finally { Pop-Location }
$log.build_exit = $LASTEXITCODE
if ($LASTEXITCODE -ne 0) { Finish 'does not compile'; exit 1 }

# 3. Run; a call that does not return is named by the log and left out of the next run.
$results = Join-Path $Out 'reached'
New-Item -ItemType Directory -Force $results | Out-Null
$skip = Join-Path $Out 'skip.txt'
Set-Content $skip '' -NoNewline -Encoding utf8NoBOM
$env:LESSCODE_WALK_OUT = $results
$env:LESSCODE_WALK_SKIP = $skip
$log.not_returning = @()
for ($try = 1; $try -le $Retries; $try++) {
    $p = Start-Process cargo -ArgumentList 'test', '--locked', '--manifest-path', $manifest, '--lib', 'state_walk::walk_the_scope' `
        -WorkingDirectory $Tree -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $Out "run-$try.out") -RedirectStandardError (Join-Path $Out "run-$try.err")
    if ($p.WaitForExit($LimitSeconds * 1000)) {
        $log.test_exit = $p.ExitCode; $log.tries = $try
        if ($p.ExitCode -eq 0 -and (Test-Path (Join-Path $results 'scope.json'))) { Finish 'walked'; exit 0 }
        Finish 'the walk failed'; exit 1
    }
    taskkill /PID $p.Id /T /F | Out-Null
    Get-Process | Where-Object { $_.Path -like "$Tree\target\*" } | Stop-Process -Force -ErrorAction SilentlyContinue
    $lines = @(Get-Content (Join-Path $results 'probe.log') -ErrorAction SilentlyContinue)
    Copy-Item (Join-Path $results 'probe.log') (Join-Path $Out "probe-$try.log") -ErrorAction SilentlyContinue
    if ($lines.Count -eq 0 -or $lines[-1] -eq '(every call returned)') { $log.tries = $try; Finish 'the walk did not end in time'; exit 1 }
    $log.not_returning += $lines[-1]
    Add-Content $skip $lines[-1] -Encoding utf8NoBOM
}
Finish 'too many calls did not return'; exit 1
