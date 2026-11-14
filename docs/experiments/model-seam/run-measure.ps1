# Task 5.3 of the model seam: `lesscode-model measure` on real models, the quick mode and then the
# full twenty trials, on the built-in task set. The cases named run one after the other; start two
# copies of this script only for cases with different upstreams (1, 2 and 5 share Mistral's service
# or the local endpoint in front of it; 3 is OpenAI's). The Anthropic case is left out.
# The report of each run is kept under measure\ as text and as JSON.
param([int[]]$Only = @(1, 2, 5, 3))
$repo = Split-Path (Split-Path (Split-Path $PSScriptRoot))
$exe = 'D:\BuildArtifacts\cargo-target\lesscode-model\debug\lesscode-model.exe'
$kept = Join-Path $PSScriptRoot 'measure'
New-Item -ItemType Directory -Force $kept | Out-Null
$home_env = Join-Path $env:USERPROFILE '.lesscode'
$cases = @{
    1 = @{ name = 'GLM 5.3, Mistral API'; file = "$home_env\mistral.env"; model = 'zai-glm-5-3' }
    2 = @{ name = 'Mistral Large, Mistral API'; file = "$home_env\mistral.env"; model = '' }
    3 = @{ name = 'gpt-6.1-sol, OpenAI API'; file = "$home_env\openai.env"; model = '' }
    5 = @{ name = 'GLM 5.3, local endpoint'; file = "$repo\.env"; model = '' }
}
foreach ($id in $Only) {
    $case = $cases[$id]
    if ($case.model) { $env:LESSCODE_MODEL_NAME = $case.model }
    else { Remove-Item Env:LESSCODE_MODEL_NAME -ErrorAction SilentlyContinue }
    foreach ($mode in 'quick', 'full') {
        $base = "$kept\case-$id-$mode"
        $arguments = @('measure', '--endpoint-env', $case.file, '--out', "$base.json")
        if ($mode -eq 'quick') { $arguments += '--quick' }
        $t0 = Get-Date
        & $exe @arguments > "$base.txt" 2> "$base.notices.txt"
        $code = $LASTEXITCODE
        "--- $id $($case.name), $mode`: started $($t0.ToString('HH:mm:ss')), $([int]((Get-Date) - $t0).TotalSeconds) s, exit $code"
        Get-Content "$base.txt" | ForEach-Object { "    $_" }
        if ((Get-Item "$base.notices.txt").Length -eq 0) { Remove-Item "$base.notices.txt" -Confirm:$false }
    }
}
Remove-Item Env:LESSCODE_MODEL_NAME -ErrorAction SilentlyContinue
"ended $(Get-Date -Format 'HH:mm:ss')"
