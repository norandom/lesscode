# R167: generate a walk for two types and run each as written.
#   ./run.ps1 -Gen <walk-gen.exe> -Out <new directory under D:\BuildArtifacts\lesscode>
# Nothing generated is edited. drift-rdp is only read; the walk runs in a copy.
param([Parameter(Mandatory)][string]$Gen, [Parameter(Mandatory)][string]$Out)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $Out) { throw "$Out exists" }
if ($Out -notmatch '^D:\\BuildArtifacts\\lesscode\\') { throw 'output must be under D:\BuildArtifacts\lesscode' }
New-Item -ItemType Directory -Force $Out | Out-Null
$rules = Join-Path $PSScriptRoot 'rust.walk.json'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
$declare = "`n#[cfg(test)]`nmod state_walk;"
$log = @{ started = (Get-Date -Format o); gen = (Get-FileHash $Gen).Hash; rules = (Get-FileHash $rules).Hash }

# 1. Order: the example file becomes the library root of a throwaway crate.
$crate = Join-Path $Out 'order'
New-Item -ItemType Directory -Force (Join-Path $crate 'src') | Out-Null
Set-Content (Join-Path $crate 'Cargo.toml') "[package]`nname = `"order-walk`"`nversion = `"0.0.0`"`nedition = `"2021`"`n" -Encoding utf8NoBOM
$lib = Join-Path $crate 'src\lib.rs'
Copy-Item (Join-Path $repo 'docs\examples\order-flags\before.rs') $lib
& $Gen $rules $lib Order (Join-Path $crate 'src\state_walk.rs') > (Join-Path $Out 'order-gen.json')
$log.order_gen_exit = $LASTEXITCODE
Add-Content $lib $declare -Encoding utf8NoBOM
$env:CARGO_TARGET_DIR = Join-Path $crate 'target'
$env:LESSCODE_WALK_OUT = Join-Path $Out 'order-reached.json'
cargo test --manifest-path (Join-Path $crate 'Cargo.toml') --lib state_walk *> (Join-Path $Out 'order-walk.log')
$log.order_test_exit = $LASTEXITCODE

# 2. SharedFrame: a copy of drift-rdp, the module beside the file.
$subject = (Resolve-Path (Join-Path $repo '..\drift-rdp')).Path
$tree = Join-Path $Out 'tree'
New-Item -ItemType Directory -Force $tree | Out-Null
robocopy $subject $tree /E /XD .git node_modules target /NFL /NDL /NJH /NJS /NP | Out-Null
if ($LASTEXITCODE -ge 8) { throw "copy failed: robocopy $LASTEXITCODE" }
$log.subject = (git -C $subject rev-parse HEAD)
$log.subject_dirty = @(git -C $subject status --porcelain -- src-tauri)
$file = Join-Path $tree 'src-tauri\src\renderer\shared_frame.rs'
New-Item -ItemType Directory -Force (Join-Path $tree 'src-tauri\src\renderer\shared_frame') | Out-Null
& $Gen $rules $file SharedFrame (Join-Path $tree 'src-tauri\src\renderer\shared_frame\state_walk.rs') > (Join-Path $Out 'frame-gen.json')
$log.frame_gen_exit = $LASTEXITCODE
Copy-Item (Join-Path $tree 'src-tauri\src\renderer\shared_frame\state_walk.rs') (Join-Path $Out 'frame-state_walk.rs')
Copy-Item (Join-Path $crate 'src\state_walk.rs') (Join-Path $Out 'order-state_walk.rs')
Add-Content $file $declare -Encoding utf8NoBOM
$env:CARGO_TARGET_DIR = Join-Path $tree 'target'
$env:LESSCODE_WALK_OUT = Join-Path $Out 'frame-reached.json'
Push-Location $tree
try { cargo test --locked --manifest-path src-tauri/Cargo.toml --lib state_walk *> (Join-Path $Out 'frame-walk.log') }
finally { Pop-Location }
$log.frame_test_exit = $LASTEXITCODE
$log.ended = (Get-Date -Format o)
$log | ConvertTo-Json | Set-Content (Join-Path $Out 'run.json') -Encoding utf8NoBOM
