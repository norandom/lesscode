# Task 8.4: `lesscode-core regenerate run` on the six first cases through the released form of both
# binaries: the two executables of one release archive pair, side by side in -Bin. The core finds
# lesscode-model beside itself; LESSCODE_MODEL_BIN is not set. Endpoint files live outside the
# repository, except the repository's .env. One case after the other, never two against one endpoint.
param(
    [Parameter(Mandatory = $true)][string]$Bin,
    [int[]]$Only = @(1, 2, 3, 4, 5, 6),
    [string]$Work = 'D:\BuildArtifacts\lesscode\model-seam',
    [string]$Out = (Join-Path $PSScriptRoot 'released')
)
$core = Join-Path $Bin 'lesscode-core.exe'
$home_env = Join-Path $env:USERPROFILE '.lesscode'
$cases = @(
    @{ id = 1; name = 'GLM 5.3, Mistral API'; file = "$home_env\mistral.env"; model = 'zai-glm-5-3' },
    @{ id = 2; name = 'Mistral Large, Mistral API'; file = "$home_env\mistral.env"; model = '' },
    @{ id = 3; name = 'gpt-6.1-sol, OpenAI API'; file = "$home_env\openai.env"; model = '' },
    @{ id = 4; name = 'Anthropic'; file = "$home_env\model.env"; model = '' },
    @{ id = 5; name = 'GLM 5.3, local endpoint'; file = (Join-Path $PSScriptRoot '..\..\..\.env'); model = '' },
    @{ id = 6; name = 'Mistral Large, local endpoint'; file = (Join-Path $PSScriptRoot '..\..\..\.env'); model = 'mistral-large-latest' }
)
$env:LESSCODE_MODEL_BIN = $null
$env:LESSCODE_MODEL_ENV = $null
New-Item -ItemType Directory -Force $Out | Out-Null
"core:  $(& $core --version)   model: $(& (Join-Path $Bin 'lesscode-model.exe') --version)"
foreach ($case in $cases | Where-Object { $_.id -in $Only }) {
    $label = "released-$($case.id)"
    $stamp = Get-Date -Format 'HHmmss'
    $cand = Join-Path $Work "cand-$label-$stamp"
    New-Item -ItemType Directory -Force $cand | Out-Null
    Copy-Item -Path (Join-Path $Work 'frozen\*') -Destination $cand -Recurse -Force
    $env:LESSCODE_MODEL_NAME = if ($case.model) { $case.model } else { $null }
    $receipt = Join-Path $Work "receipt-$label-$stamp.json"
    $t0 = Get-Date
    $said = (& $core regenerate run --request (Join-Path $Work 'request.json') --cand $cand --receipt $receipt `
            --label $label --frozen (Join-Path $Work 'frozen') --turns (Join-Path $Work "turns-$label-$stamp") `
            --model-env $case.file 2>&1 | Out-String).Trim()
    $code = $LASTEXITCODE
    $j = Get-Content $receipt -Raw | ConvertFrom-Json
    $turn = ($j.notes | Where-Object { $_.key -eq "$label-turn" }).value
    $failed = ($j.notes | Where-Object { $_.key -eq "$label-failed" }).value
    $changed = (Get-FileHash (Join-Path $cand 'src\mod.py')).Hash -ne (Get-FileHash (Join-Path $Work 'frozen\src\mod.py')).Hash
    "--- $($case.id) $($case.name), $(Get-Date -Format 'HH:mm:ss'): exit $code after $([int]((Get-Date) - $t0).TotalSeconds) s; candidate changed: $changed"
    if ($turn) {
        "    turn: mode=$($turn.mode) stop=$($turn.stop_reason) usage=$($turn.usage | ConvertTo-Json -Compress) waits=$(@($turn.waits).Count)"
        "    sent: $($turn.sent | ConvertTo-Json -Compress)"
    }
    if ($failed) { "    FAILED: $($failed.reason)" }
    if ($said) { "    terminal: $($said.Substring(0, [Math]::Min(300, $said.Length)))" }
    Copy-Item $receipt (Join-Path $Out "receipt-$label.json") -Force
    Copy-Item (Join-Path $cand 'src\mod.py') (Join-Path $Out "candidate-$($case.id).py") -Force
}
$env:LESSCODE_MODEL_NAME = $null
