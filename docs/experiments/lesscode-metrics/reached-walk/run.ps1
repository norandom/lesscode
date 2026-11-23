# R166: copy drift-rdp, place the walk beside shared_frame.rs in the copy, run it as a test.
#   ./run.ps1 -Out <new directory under D:\BuildArtifacts\lesscode>
# The project itself is only read.
param([Parameter(Mandatory)][string]$Out)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $Out) { throw "$Out exists" }
if ($Out -notmatch '^D:\\BuildArtifacts\\lesscode\\') { throw 'output must be under D:\BuildArtifacts\lesscode' }
$subject = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..\..\drift-rdp')).Path
$tree = Join-Path $Out 'tree'
New-Item -ItemType Directory -Force $tree | Out-Null
robocopy $subject $tree /E /XD .git node_modules target /NFL /NDL /NJH /NJS /NP | Out-Null
if ($LASTEXITCODE -ge 8) { throw "copy failed: robocopy $LASTEXITCODE" }
$file = Join-Path $tree 'src-tauri\src\renderer\shared_frame.rs'
$before = (Get-FileHash $file).Hash
New-Item -ItemType Directory -Force (Join-Path $tree 'src-tauri\src\renderer\shared_frame') | Out-Null
Copy-Item (Join-Path $PSScriptRoot 'state_walk.rs') (Join-Path $tree 'src-tauri\src\renderer\shared_frame\state_walk.rs')
Add-Content $file "`n#[cfg(test)]`nmod state_walk;" -Encoding utf8NoBOM
@{
    started = (Get-Date -Format o)
    subject = (git -C $subject rev-parse HEAD)
    dirty   = @(git -C $subject status --porcelain -- src-tauri)
    shared_frame_sha256_before_the_added_line = $before
    walk_sha256 = (Get-FileHash (Join-Path $PSScriptRoot 'state_walk.rs')).Hash
} | ConvertTo-Json | Set-Content (Join-Path $Out 'start.json') -Encoding utf8NoBOM
$env:CARGO_TARGET_DIR = Join-Path $tree 'target'
$env:LESSCODE_WALK_OUT = Join-Path $Out 'reached.json'
Push-Location $tree
try { cargo test --locked --manifest-path src-tauri/Cargo.toml --lib state_walk -- --nocapture *> (Join-Path $Out 'walk.log') }
finally { Pop-Location }
$code = $LASTEXITCODE
@{ ended = (Get-Date -Format o); exit_code = $code } | ConvertTo-Json | Set-Content (Join-Path $Out 'end.json') -Encoding utf8NoBOM
exit $code
