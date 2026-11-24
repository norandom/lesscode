# Task 8.1 of run-dashboard: `step plan --page --open` on the renderer of drift-rdp, watched by the owner.
#   ./run.ps1 -Core <lesscode-core.exe> -Out <new directory under D:\BuildArtifacts\lesscode>
# Same scope and settings as R164 (../../lesscode-metrics/renderer-baseline). Starts the Rust command
# and keeps its output. It measures nothing itself.
param([Parameter(Mandatory)][string]$Core, [Parameter(Mandatory)][string]$Out)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $Out) { throw "$Out exists" }
if ($Out -notmatch '^D:\\BuildArtifacts\\lesscode\\') { throw 'output must be under D:\BuildArtifacts\lesscode' }
New-Item -ItemType Directory -Force $Out | Out-Null
$config = Join-Path $PSScriptRoot 'config.json'
$subject = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..\..\drift-rdp')).Path
@{
    started   = (Get-Date -Format o)
    core      = (Get-FileHash $Core).Hash
    config    = (Get-FileHash $config).Hash
    subject   = (git -C $subject rev-parse HEAD)
    dirty     = @(git -C $subject status --porcelain -- src-tauri)
    free_gib  = [math]::Round((Get-PSDrive D).Free / 1GB, 1)
} | ConvertTo-Json | Set-Content (Join-Path $Out 'start.json') -Encoding utf8NoBOM
& $Core step plan --config $config --work (Join-Path $Out 'work') --out (Join-Path $Out 'plan.json') --page --open *> (Join-Path $Out 'plan.log')
$code = $LASTEXITCODE
@{ ended = (Get-Date -Format o); exit_code = $code; free_gib = [math]::Round((Get-PSDrive D).Free / 1GB, 1) } |
    ConvertTo-Json | Set-Content (Join-Path $Out 'end.json') -Encoding utf8NoBOM
exit $code
