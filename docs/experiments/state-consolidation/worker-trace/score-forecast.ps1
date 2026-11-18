param([Parameter(Mandatory)][string]$Run, [Parameter(Mandatory)][string]$NewScore)
$ErrorActionPreference = 'Stop'
function ReadJson($path) { Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -AsHashtable }
function Sha($path) { (Get-FileHash -LiteralPath $path).Hash }
function CompareRow($guess,$expected) {
    [ordered]@{
        test_version=($guess.test_version -is [string] -and $guess.test_version -ceq $expected.test_version)
        reused=($guess.reused -is [bool] -and $guess.reused -eq $expected.reused)
        classification=($guess.classification -is [string] -and $guess.classification -ceq $expected.classification)
    }
}
$destination = [IO.Path]::GetFullPath($NewScore)
if ((Test-Path $destination) -or !$destination.StartsWith('D:\BuildArtifacts\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Choose a new score file under D:\BuildArtifacts' }
$packets = ReadJson "$Run/packets.json"
$arms = [ordered]@{X='fixed';Y='before'}
foreach ($id in $arms.Keys) {
    $seal = ReadJson "$Run/$id/seal.json"
    if ((Sha "$Run/$id/answer.json") -ne $seal.answer -or (Sha "$Run/$id/request.json") -ne $seal.request -or $packets.requests[$id] -ne $seal.request) { throw 'Incomplete or changed sealed pair' }
}
$frozen = 'D:/BuildArtifacts/lesscode/corrective-audit/task68'
if ((Sha "$frozen/evidence-sha256.json") -ne 'F7C983F029B61DE02867DD81D9B0AC2C24D0C6DB54D9FB360AB91EDEDEC41C17') { throw 'Changed reference index' }
$pins = @{}
ReadJson "$frozen/evidence-sha256.json" | ForEach-Object { $pins[$_.path]=$_.sha256 }
function Reference($path) {
    $file = Join-Path $frozen $path
    if ((Sha $file) -ne $pins[$path]) { throw "Unverified reference $path" }
    $file
}
$basis = @{
    identity=@('run_command::identity_with_tests','store::key','run_command::run_run','run_command::planned')
    workspace=@('runner::open_copy','workspace::open','workspace::create','sync::refresh','workspace::copy_tree')
    execution=@('runner::one','process::run')
}
$hashes = @{}
foreach ($line in 'def step(','if a > 3:') {
    $hash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($line+"`n")))
    $hashes[$hash] = if ($line -eq 'def step(') { 'original' } else { 'edited' }
}
$scores = [ordered]@{}
$rejected = 0
$machinePass = $true
foreach ($id in $arms.Keys) {
    $dir = "$Run/$id"
    $envelope = $null
    $answer = $null
    $diagnostic = $null
    try {
        $envelope = ReadJson "$dir/answer.json"
        if ((ReadJson "$dir/seal.json").exit -ne 0 -or $envelope.ok -ne $true -or $envelope.reply.stop -cne 'complete') { throw 'Model call did not complete normally' }
        if ($envelope.reply.sent.model -cne 'zai-glm-5-3' -or $envelope.reply.sent.reasoning_level -cne 'high') {throw 'Model configuration differs from R141'}
        $body = ([string]$envelope.reply.text).Trim()
        if ($body.StartsWith('```json') -and $body.EndsWith('```')) { $body=$body.Substring(7,$body.Length-10) }
        $answer = ConvertFrom-Json -InputObject $body -AsHashtable
        if ($answer -isnot [System.Collections.IDictionary]) { throw 'Missing answer object' }
    } catch { $diagnostic=$_.Exception.Message }
    $rows = [ordered]@{}
    foreach ($scenario in 'reopened','fresh') {
        $phase = if ($scenario -eq 'reopened') { 'changed' } else { 'fresh' }
        $prefix = "trial/$($arms[$id])"
        $receipt = ReadJson (Reference "$prefix/history/$phase.json")
        $children = @(Get-Content (Reference "$prefix/children/$phase.jsonl") | ForEach-Object { ConvertFrom-Json $_ -AsHashtable })
        $hash = @($children.hash | Sort-Object -Unique)
        $mutation = @($receipt.mutants | Where-Object { $_.operator -ceq 'negate' -and $_.site.start_line -eq 2 })
        if ($receipt.incomplete -or $null -ne $receipt.void -or $mutation.Count -ne 1 -or $hash.Count -ne 1 -or !$hashes.ContainsKey($hash[0]) -or $children.Count -ne 7) { throw 'Inconclusive reference' }
        $expected = @{test_version=$hashes[$hash[0]];reused=[bool]$mutation[0].reused_from;classification=$mutation[0].class}
        if ($expected.classification -cnotin 'survived','killed') { throw 'Unexpected mutation classification' }
        foreach ($field in $expected.Keys) {
            $altered = $expected.Clone()
            $altered[$field] = switch ($field) {
                'test_version' { if ($expected[$field] -eq 'original') {'edited'} else {'original'} }
                'reused' { !$expected[$field] }
                'classification' { if ($expected[$field] -eq 'killed') {'survived'} else {'killed'} }
            }
            $control = CompareRow $altered $expected
            if ($control[$field] -or @($control.Values -eq $false).Count -ne 1) { throw "Failed inverted-$field control" }
            $rejected++
        }
        $guess = if ($answer -is [System.Collections.IDictionary]) { $answer[$scenario] }
        $rows[$scenario] = @{expected=$expected;predicted=$guess;matches=(CompareRow $guess $expected);child_observations=$children.Count}
    }
    $evidence = [ordered]@{}
    $request = ReadJson "$dir/request.json"
    foreach ($field in $basis.Keys) {
        $cited = if ($answer.basis -is [System.Collections.IDictionary]) { $answer.basis[$field] }
        $evidence[$field] = $cited -is [string] -and $cited -cin $basis[$field] -and $request.user.Contains('"id": "'+$cited+'"')
    }
    $scores[$id] = [ordered]@{rows=$rows;basis=$evidence;diagnostic=$diagnostic;explanation=$answer.explanation;tokens=$envelope.reply.output_tokens;input_tokens=$envelope.reply.input_tokens;stop=$envelope.reply.stop}
    foreach ($row in $rows.Values) {
        if ($row.matches.Values -contains $false) { $machinePass=$false }
    }
    if ($diagnostic -or $evidence.Values -contains $false -or $answer.explanation -isnot [string] -or !$answer.explanation.Trim()) { $machinePass=$false }
}
[ordered]@{scored=(Get-Date).ToString('o');arms=$scores;inverted_fields_rejected=$rejected;machine_pass=$machinePass} | ConvertTo-Json -Depth 9 | Set-Content -LiteralPath $destination -Encoding utf8NoBOM
if (!$machinePass) { exit 2 }
exit 0
