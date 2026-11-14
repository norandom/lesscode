# Task 6 of the model seam: one live judgement of four trials by tools/guide-optimizer, whose calls
# now go through the seam. One call per language, two repairs allowed, on the local endpoint (GLM 5.3).
# The tool has no list price for this model and counts its fallback price; the tokens are in the summary.
# Output is kept under guide-tool\.
$repo = Split-Path (Split-Path (Split-Path $PSScriptRoot))
$exe = 'D:\BuildArtifacts\cargo-target\guide-optimizer\debug\guide-optimizer.exe'
$kept = Join-Path $PSScriptRoot 'guide-tool'
New-Item -ItemType Directory -Force $kept | Out-Null
Remove-Item Env:LESSCODE_MODEL_NAME -ErrorAction SilentlyContinue
Set-Location $repo
$t0 = Get-Date
& $exe --out-dir $kept --endpoint-env "$repo\.env" `
    --judge-only docs/experiments/annotation-driver/pack-guide-v3.md `
    --judge-list rust,java,python,typescript --repairs 2 > "$kept\run.txt" 2> "$kept\notices.txt"
$code = $LASTEXITCODE
"started $($t0.ToString('HH:mm:ss')), $([int]((Get-Date) - $t0).TotalSeconds) s, exit $code"
Get-Content "$kept\run.txt" | ForEach-Object { "    $_" }
if ((Get-Item "$kept\notices.txt").Length -eq 0) { Remove-Item "$kept\notices.txt" -Confirm:$false }
"ended $(Get-Date -Format 'HH:mm:ss')"
