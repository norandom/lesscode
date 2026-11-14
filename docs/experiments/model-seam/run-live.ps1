# Task 5.1 of the model seam: the six first cases of requirement 2.7 through `lesscode-model call`
# (the turn of R102) and `lesscode-model check`, by model/tests/live.rs. One case after the other and
# one test thread: no two calls against one upstream side by side. Endpoint files live outside the
# repository, except the repository's own .env. What the binary said is kept under live\case-N.
#   -Only 4 -Part call    one case, one of the two commands
param([int[]]$Only = @(1, 2, 3, 4, 5, 6), [ValidateSet('both', 'call', 'check')][string]$Part = 'both')
$repo = Split-Path (Split-Path (Split-Path $PSScriptRoot))
$env:CARGO_TARGET_DIR = 'D:\BuildArtifacts\cargo-target\lesscode-model'
$env:LESSCODE_LIVE_TURN = 'D:\BuildArtifacts\lesscode\model-seam\turns\t2'
$home_env = Join-Path $env:USERPROFILE '.lesscode'
$cases = @(
    @{ id = 1; name = 'GLM 5.3, Mistral API'; file = "$home_env\mistral.env"; model = 'zai-glm-5-3' },
    @{ id = 2; name = 'Mistral Large, Mistral API'; file = "$home_env\mistral.env"; model = '' },
    @{ id = 3; name = 'gpt-6.1-sol, OpenAI API'; file = "$home_env\openai.env"; model = '' },
    @{ id = 4; name = 'Anthropic'; file = "$home_env\model.env"; model = '' },
    @{ id = 5; name = 'GLM 5.3, local endpoint'; file = "$repo\.env"; model = '' },
    @{ id = 6; name = 'Mistral Large, local endpoint'; file = "$repo\.env"; model = 'mistral-large-latest' }
)
$filter = @{ both = 'all()'; call = 'test(call_)'; check = 'test(check_)' }[$Part]
foreach ($case in $cases | Where-Object { $_.id -in $Only }) {
    $env:LESSCODE_LIVE_ENV = $case.file
    $env:LESSCODE_LIVE_OUT = Join-Path $PSScriptRoot "live\case-$($case.id)"
    if ($case.model) { $env:LESSCODE_MODEL_NAME = $case.model }
    else { Remove-Item Env:LESSCODE_MODEL_NAME -ErrorAction SilentlyContinue }
    "--- $($case.id) $($case.name) ($Part), started $(Get-Date -Format 'HH:mm:ss')"
    $said = (& cargo nextest run --locked --manifest-path "$repo\model\Cargo.toml" --test live `
            --test-threads 1 --no-fail-fast -E $filter 2>&1 | Out-String)
    "    tests: exit $LASTEXITCODE; " + (($said -split "`n" | Where-Object { $_ -match 'Summary|FAIL' }) -join ' | ').Trim()
    foreach ($kept in 'call.txt', 'check.txt') {
        $path = Join-Path $env:LESSCODE_LIVE_OUT $kept
        if (-not (Test-Path -LiteralPath $path) -or ($Part -ne 'both' -and $kept -ne "$Part.txt")) { continue }
        $text = Get-Content -LiteralPath $path -Raw
        if ($kept -eq 'call.txt' -and $text -match '(?s)--- standard output\r?\n(\{.*\})\s*--- standard error') {
            $answer = $Matches[1] | ConvertFrom-Json
            if ($answer.ok) {
                $r = $answer.reply
                "    call: $($r.sent.model) at $($r.sent.host), $([Math]::Round($r.seconds, 1)) s, $($r.input_tokens) in, $($r.output_tokens) out, stop $($r.stop), structured $($r.structured), waits $($r.waits.Count)"
            }
            else { "    call FAILED ($($answer.error.kind)): $($answer.error.reason)" }
        }
        elseif ($kept -eq 'check.txt') {
            ($text -split "`r?`n" | Where-Object { $_ -and $_ -notmatch '^--- ' }) | ForEach-Object { "    check: $_" }
        }
    }
}
Remove-Item Env:LESSCODE_MODEL_NAME, Env:LESSCODE_LIVE_ENV, Env:LESSCODE_LIVE_OUT, Env:LESSCODE_LIVE_TURN -ErrorAction SilentlyContinue
Get-Date -Format 'HH:mm'
