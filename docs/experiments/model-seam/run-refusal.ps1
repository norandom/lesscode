# Task 5.2 of the model seam: provoke a real rate limit on the local endpoint and see one call
# through `lesscode-model call` announce its estimate, wait and be answered.
# Rounds of large calls side by side, all through `call`, growing by the steps below; in every round
# one further call is the small turn of R102 (the probe). This script is the only run against the
# upstream while it works. It stops when a call was answered after a wait, when the time is up, or when
# the input-token budget is spent. Records and notices are kept under refusal\; the large request
# bodies and the answers are not kept in the repository.
param([int]$Minutes = 60, [long]$InputTokenBudget = 15000000)
$repo = Split-Path (Split-Path (Split-Path $PSScriptRoot))
$exe = 'D:\BuildArtifacts\cargo-target\lesscode-model\debug\lesscode-model.exe'
$endpoint = "$repo\.env"
$turn = 'D:\BuildArtifacts\lesscode\model-seam\turns\t2'
$work = 'D:\BuildArtifacts\lesscode\model-seam\refusal'
$kept = Join-Path $PSScriptRoot 'refusal'
New-Item -ItemType Directory -Force $work, $kept | Out-Null
Remove-Item "$kept\*" -Confirm:$false -ErrorAction SilentlyContinue
Remove-Item Env:LESSCODE_MODEL_NAME -ErrorAction SilentlyContinue

# calls side by side, and the size of each large request in hundreds of entries (about 16 tokens each)
$steps = @(@(8, 15), @(16, 15), @(16, 45), @(24, 45), @(24, 75), @(32, 75))
$words = 'ledger', 'harbour', 'window', 'copper', 'meadow', 'signal', 'basket', 'river', 'anchor', 'timber'
$probe = @{ protocol = 1; purpose = 'refusal-probe'; max_output_tokens = 64000
    system = (Get-Content "$turn\system.txt" -Raw); user = (Get-Content "$turn\user.txt" -Raw) } | ConvertTo-Json

$deadline = (Get-Date).AddMinutes($Minutes)
$spent = 0L; $round = 0; $done = $false
"started $(Get-Date -Format 'HH:mm:ss'), until $($deadline.ToString('HH:mm:ss')), budget $InputTokenBudget input tokens"
while (-not $done -and (Get-Date) -lt $deadline -and $spent -lt $InputTokenBudget) {
    $step = $steps[[Math]::Min($round, $steps.Count - 1)]
    $round++
    $side, $hundreds = $step
    $requests = 0..$side | ForEach-Object {
        $file = "$work\request-$round-$_.json"
        if ($_ -eq 0) { $body = $probe }
        else {
            # Each body starts differently, so that no stored prefix shortens it.
            $text = [System.Text.StringBuilder]::new("Document $round-$_ of $(Get-Random).`n")
            foreach ($n in 1..($hundreds * 100)) {
                [void]$text.Append("Entry $($n + $_ * 7 + $round * 13): the $($words[($n + $_) % 10]) by the $($words[($n * 3 + $round) % 10]) was counted $($n % 97) times.`n")
            }
            $body = @{ protocol = 1; purpose = "refusal-load-$round"; max_output_tokens = 1500
                system = 'You summarise documents.'
                user = "$text`nWrite about 800 words on the regularities in the entries above." } | ConvertTo-Json
        }
        Set-Content -LiteralPath $file -Value $body -NoNewline
        $file
    }
    $t0 = Get-Date
    $requests | ForEach-Object -ThrottleLimit ($side + 1) -Parallel {
        $name = [IO.Path]::GetFileNameWithoutExtension($_).Replace('request', 'call')
        Get-Content -LiteralPath $_ -Raw | & $using:exe call --endpoint-env $using:endpoint `
            --record "$using:kept\$name.jsonl" > "$using:work\$name.out" 2> "$using:kept\$name.notices.txt"
        Set-Content "$using:work\$name.exit" $LASTEXITCODE
    }
    $lines = Get-ChildItem "$kept\call-$round-*.jsonl" | ForEach-Object { Get-Content $_ | ConvertFrom-Json }
    $answered = @($lines | Where-Object ok)
    $waited = @($answered | Where-Object { $_.waits.Count -gt 0 })
    $tokens = ($answered | Measure-Object input_tokens -Sum).Sum
    $spent += [long]$tokens
    $notices = @(Get-ChildItem "$kept\call-$round-*.notices.txt" | Where-Object Length -gt 0)
    "round $round at $($t0.ToString('HH:mm:ss')): $side large calls of $hundreds hundred entries and the probe; " +
    "$($answered.Count) answered, $($lines.Count - $answered.Count) failed, $($waited.Count) answered after a wait, " +
    "$($notices.Count) with notices, $tokens input tokens, $([int]((Get-Date) - $t0).TotalSeconds) s"
    $lines | Where-Object { -not $_.ok } | Group-Object failure | ForEach-Object { "    failed x$($_.Count): $($_.Name)" }
    Get-ChildItem "$kept\call-$round-*.notices.txt" | Where-Object Length -eq 0 | Remove-Item -Confirm:$false
    Remove-Item "$work\request-$round-*.json" -Confirm:$false
    if ($waited.Count -gt 0) { $done = $true }
}
"ended $(Get-Date -Format 'HH:mm:ss'): $round rounds, $spent input tokens answered, " + $(if ($done) { 'a call was answered after a wait' } else { 'no call was answered after a wait' })
