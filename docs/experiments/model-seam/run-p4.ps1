# Gate P4: `lesscode-core regenerate run` reaches the model through the second executable
# (LESSCODE_MODEL_BIN) on the six first cases. The core binary must be built from the branch
# model-seam-gate-p4. Endpoint files live outside the repository, except the repository's .env.
param([int[]]$Only = @(1, 2, 3, 4, 5, 6))
$w = 'D:\BuildArtifacts\lesscode\model-seam'
$core = Join-Path $env:CARGO_TARGET_DIR 'debug\lesscode-core.exe'
$env:LESSCODE_MODEL_BIN = 'D:\BuildArtifacts\cargo-target\model-seam-prototype-wait\debug\model-seam-prototype-wait.exe'
$home_env = Join-Path $env:USERPROFILE '.lesscode'
$cases = @(
    @{ id = 1; name = 'GLM 5.3, Mistral API'; file = "$home_env\mistral.env"; model = 'zai-glm-5-3' },
    @{ id = 2; name = 'Mistral Large, Mistral API'; file = "$home_env\mistral.env"; model = '' },
    @{ id = 3; name = 'gpt-6.1-sol, OpenAI API'; file = "$home_env\openai.env"; model = '' },
    @{ id = 4; name = 'Anthropic'; file = "$home_env\model.env"; model = '' },
    @{ id = 5; name = 'GLM 5.3, local endpoint'; file = 'D:\Source\lesscode\.env'; model = '' },
    @{ id = 6; name = 'Mistral Large, local endpoint'; file = 'D:\Source\lesscode\.env'; model = 'mistral-large-latest' }
)
Set-Location 'D:\Source\lesscode'
foreach ($case in $cases | Where-Object { $_.id -in $Only }) {
    $label = "p4-$($case.id)"
    $cand = "$w\cand-$label"
    New-Item -ItemType Directory -Force $cand | Out-Null
    Copy-Item "$w\frozen\*" $cand -Recurse -Force
    $env:LESSCODE_MODEL_ENV = $case.file
    if ($case.model) { $env:LESSCODE_MODEL_NAME = $case.model }
    else { Remove-Item Env:LESSCODE_MODEL_NAME -ErrorAction SilentlyContinue }
    $receipt = "$w\receipt-$label.json"
    if (Test-Path -LiteralPath $receipt) { Remove-Item -LiteralPath $receipt -Confirm:$false }
    $t0 = Get-Date
    $said = (& $core regenerate run --request "$w\request.json" --cand $cand --receipt $receipt `
        --label $label --frozen "$w\frozen" --turns "$w\turns" --model-env $case.file 2>&1 | Out-String).Trim()
    $code = $LASTEXITCODE
    $j = Get-Content $receipt -Raw | ConvertFrom-Json
    $turn = ($j.notes | Where-Object { $_.key -eq "$label-turn" }).value
    $failed = ($j.notes | Where-Object { $_.key -eq "$label-failed" }).value
    $changed = (Get-FileHash "$cand\src\mod.py").Hash -ne (Get-FileHash "$w\frozen\src\mod.py").Hash
    "--- $($case.id) $($case.name): exit $code after $([int]((Get-Date) - $t0).TotalSeconds)s; candidate changed: $changed"
    if ($turn) { "    turn: stop=$($turn.stop_reason) usage=$($turn.usage | ConvertTo-Json -Compress -Depth 5)" }
    if ($failed) { "    FAILED: $($failed.reason)" }
    if ($said) { "    terminal: $($said.Substring(0, [Math]::Min(300, $said.Length)))" }
}
Remove-Item Env:LESSCODE_MODEL_NAME, Env:LESSCODE_MODEL_ENV, Env:LESSCODE_MODEL_BIN -ErrorAction SilentlyContinue
Get-Date -Format 'HH:mm'
