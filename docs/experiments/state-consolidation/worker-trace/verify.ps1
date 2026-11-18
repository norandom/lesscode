param(
    [Parameter(Mandatory)][string]$Tracer,
    [Parameter(Mandatory)][string]$Evidence,
    [string]$NewResults
)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
function Require($ok,$reason) {if(!$ok){throw $reason}}
function Json($file) {Get-Content -LiteralPath $file -Raw | ConvertFrom-Json -AsHashtable}
function Mutants($receipt) {
    @($receipt.mutants | Sort-Object number | ForEach-Object {
        (@($_.number,$_.sha256,$_.class,[bool]$_.reused_from) -join '|')
    }) -join ','
}
$results=$Evidence
if ($NewResults) {
    $results=[IO.Path]::GetFullPath($NewResults)
    Require ($results.StartsWith('D:\BuildArtifacts\',[StringComparison]::OrdinalIgnoreCase) -and !(Test-Path $results)) 'Choose a new verification directory under D:\BuildArtifacts'
    $null=New-Item -ItemType Directory -Path $results
}
foreach ($name in 'missing-event','missing-origin','verified.json') {
    Require (!(Test-Path (Join-Path $results $name))) 'Verification results already exist. Use -NewResults to keep the previous run.'
}
$summary=@()
foreach($arm in 'before','fixed','disabled'){
    $base="$Evidence/$arm"
    $prediction=Json "$base/prediction.json"
    $reference=if($arm -eq 'fixed'){'fixed'}else{'before'}
    foreach($phase in 'warm','unchanged','changed','fresh'){
        $p=$prediction[$phase]
        $children=@(Get-Content -LiteralPath "$base/children/$phase.jsonl" | ForEach-Object {ConvertFrom-Json $_ -AsHashtable})
        $receipt=Json "$base/history/$phase.json"
        $plain=Json "$Evidence/plain-$reference/$phase.json"
        Require ((Mutants $receipt) -ceq (Mutants $plain)) "Outcome drift on $arm/$phase"
        Require ($p.test_identity -ceq $receipt.test_files_sha256 -and $p.test_identity -ceq $plain.test_files_sha256) "Unmatched test identity on $arm/$phase"
        $expectedWorkspace=if($phase -in 'warm','fresh'){'create'}else{'open'}
        Require ($p.worker -ceq $expectedWorkspace) "Unexpected worker path on $arm/$phase"
        $hits=@($receipt.mutants | Where-Object reused_from).Count
        Require ($hits -eq $p.cache_hits -and ($hits+$p.mutant_executions) -eq $receipt.mutants.Count) "Lookup or execution mismatch on $arm/$phase"
        Require ($p.commands.Count -eq $children.Count -and $children.Count -gt 0) "Missing child observation on $arm/$phase"
        $stale=0
        for($i=0;$i -lt $children.Count;$i++){
            $cmd=$p.commands[$i];$child=$children[$i]
            Require ($cmd.cwd -ceq $child.cwd -and $cmd.file.Replace('/','\') -ceq $child.file.Replace('/','\') -and $cmd.predicted_hash -ceq $child.hash) "Child bytes or path disagree on $arm/$phase/$i"
            if($child.hash -cne $p.project_hash){$stale++}
        }
        $expectedStale=if($phase -eq 'changed' -and $arm -ne 'fixed'){$children.Count}else{0}
        Require ($stale -eq $expectedStale) "Unexpected count of stale executions on $arm/$phase"
        $selected=@($receipt.mutants | Where-Object { $_.operator -ceq 'negate' -and $_.site.start_line -eq 2 })
        Require ($selected.Count -eq 1) "Ambiguous selected fault on $arm/$phase"
        $summary += [ordered]@{arm=$arm;phase=$phase;worker=$p.worker;cache_hits=$hits;mutants_run=$p.mutant_executions;
            child_hashes_matched=$children.Count;stale=$stale;selected_fault=$selected[0].class;last_copy=$p.commands[0].last_copy}
    }
    Require ($prediction.warm.configuration_with_tests -ceq $prediction.unchanged.configuration_with_tests -and
        $prediction.changed.configuration_with_tests -cne $prediction.warm.configuration_with_tests -and
        $prediction.changed.configuration_with_tests -ceq $prediction.fresh.configuration_with_tests) "Unexpected identity transitions on $arm"
}
# In the disabled copy, the refresh implementation and its copy site survive.
$fixedSite=@((Json "$Evidence/fixed/overlay/sites.json").sites | Where-Object { $_.operation -ceq 'copy' -and $_.function -ceq 'refresh' })
$disabledSite=@((Json "$Evidence/disabled/overlay/sites.json").sites | Where-Object { $_.operation -ceq 'copy' -and $_.function -ceq 'refresh' })
Require ($fixedSite.Count -eq 1 -and $disabledSite.Count -eq 1 -and $fixedSite[0].quote -ceq $disabledSite[0].quote) 'Missing intact refresh control'
Require ((Json "$Evidence/fixed/prediction.json").changed.executed_sites -ccontains $fixedSite[0].id) 'Fixed refresh did not copy a file'
Require ((Json "$Evidence/disabled/prediction.json").changed.executed_sites -cnotcontains $disabledSite[0].id) 'Disabled refresh unexpectedly executed'

# Drop a successful watched copy. First test event continuity, then remove
# the gap as well to check that the predictor needs a real byte origin.
foreach($control in 'missing-event','missing-origin'){
    $dst=Join-Path $results $control
    $null=New-Item -ItemType Directory -Path $dst
    Copy-Item "$Evidence/before/events/*.jsonl" $dst
    $events=@(Get-Content "$dst/warm.jsonl" | ForEach-Object {ConvertFrom-Json $_ -AsHashtable})
    $kept=@($events |
        Where-Object {!($_.op -ceq 'copy' -and $_.data.to.Replace('/','\').EndsWith('\tests\check.txt'))})
    Require ($events.Count -eq $kept.Count+1) 'Expected one successful initial copy of the watched file'
    $lines=for($i=0;$i -lt $kept.Count;$i++){
        if($control -eq 'missing-origin'){$kept[$i].sequence=$i+1}
        $kept[$i]|ConvertTo-Json -Compress -Depth 8
    }
    $lines|Set-Content "$dst/warm.jsonl" -Encoding utf8NoBOM
    & $Tracer predict "$Evidence/before/overlay/sites.json" $dst tests/check.txt "$dst/prediction.json" 2> "$dst/diagnostic.txt"
    Require ($LASTEXITCODE -eq 1 -and !(Test-Path "$dst/prediction.json")) "Incomplete trace accepted in $control"
    $expected=if($control -eq 'missing-event'){'missing or reordered driver event'}else{'no observed copy'}
    Require ((Get-Content "$dst/diagnostic.txt" -Raw) -match $expected) "Wrong rejection for $control"
}
$summary | ConvertTo-Json -Depth 6 | Set-Content "$results/verified.json" -Encoding utf8NoBOM
$summary | ForEach-Object {[pscustomobject]$_} | Format-Table arm,phase,worker,cache_hits,mutants_run,child_hashes_matched,stale,selected_fault
exit 0
