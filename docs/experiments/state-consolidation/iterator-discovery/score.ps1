param([Parameter(Mandatory)][string]$Run,[Parameter(Mandatory)][string]$NewScore)
$ErrorActionPreference='Stop'
function Json($f) {Get-Content -LiteralPath $f -Raw|ConvertFrom-Json -AsHashtable}
function Sha($f) {(Get-FileHash -LiteralPath $f).Hash}
function Integer($v) {($v -is [int] -or $v -is [long])}
function EqualScalar($v,$expected) {
    if ($expected -is [bool]) {return ($v -is [bool] -and $v -eq $expected)}
    if (Integer $expected) {return ((Integer $v) -and $v -eq $expected)}
    return ($v -is [string] -and $v -ceq $expected)
}
$dest=[IO.Path]::GetFullPath($NewScore)
if ((Test-Path $dest) -or !$dest.StartsWith('D:\BuildArtifacts\',[StringComparison]::OrdinalIgnoreCase)) {throw 'Choose a new D:\BuildArtifacts score file'}
$pins=Json "$Run/inputs.json"
$seal=Json "$Run/seal.json"
$jvm=Json "$Run/jvm/run.json"
if ((Sha $PSCommandPath) -ne $pins.scorer -or (Sha "$Run/packet/request.json") -ne $pins.request -or $seal.request -ne $pins.request -or
    (Sha "$Run/packet/selection.json") -ne $pins.selection -or (Sha "$Run/answer.json") -ne $seal.answer -or $jvm.answer -ne $seal.answer -or
    $jvm.exit -ne 0 -or (Sha "$Run/jvm/observed.tsv") -ne $jvm.observations) {throw 'Frozen inputs or observations changed'}
$lines=(Json "$Run/packet/selection.json").lines
$report=$null
$diagnostic=$null
try {
    $reply=Json "$Run/answer.json"
    if ($seal.exit -ne 0 -or $reply.ok -ne $true -or $reply.reply.stop -cne 'complete' -or $reply.reply.sent.model -cne 'zai-glm-5-3' -or $reply.reply.sent.reasoning_level -cne 'high') {throw 'Incomplete or mismatched model call'}
    $body=$reply.reply.text.Trim()
    if ($body.StartsWith('```json') -and $body.EndsWith('```')) {$body=$body.Substring(7,$body.Length-10)}
    $report=ConvertFrom-Json -InputObject $body -AsHashtable
    if ($report -isnot [System.Collections.IDictionary]) {throw 'Expected a report object'}
} catch {$diagnostic=$_.Exception.Message}
$schemaOK=$report -is [System.Collections.IDictionary] -and $report.state -is [string] -and ![string]::IsNullOrWhiteSpace($report.state)
$citations=@()
foreach ($c in @($report.evidence)) {
    $valid=$false
    if ($c.file -is [string] -and (Integer $c.line) -and $c.quote -is [string] -and $lines.Contains($c.file)) {
        $source=$lines[$c.file][[string]$c.line]
        $valid=$null -ne $source -and $source -ceq $c.quote.Trim()
    }
    $citations+=[ordered]@{file=$c.file;line=$c.line;valid=$valid;quote=$c.quote}
}
$schedules=[ordered]@{H1='hasNextA,hasNextA,nextB,nextA,hasNextB,nextB';H2='nextB,hasNextA,hasNextB,nextB,nextA';H3='hasNextA,close,hasNextB,nextA'}
$raw=@(Get-Content "$Run/jvm/observed.tsv")
$rows=@()
$controls=@()
$consumed=0
foreach ($id in $schedules.Keys) {
    $ops=@('start')+@($schedules[$id].Split(','))
    $firstRow=$rows.Count
    $guess=if ($report -is [System.Collections.IDictionary]) {$report[$id]}
    if ($guess.steps -isnot [array] -or $guess.steps.Count -ne $ops.Count-1) {$schemaOK=$false}
    for ($i=0;$i -lt $ops.Count;$i++) {
        if ($consumed -ge $raw.Count) {throw 'Incomplete reference'}
        $cells=$raw[$consumed++].Split("`t")
        if ($cells.Count -ne 4 -or $cells[0] -cne $id -or $cells[1] -cne $ops[$i]) {throw 'Wrong reference operation'}
        $expectedValue=if ($i -eq 0) {[bool]::Parse($cells[2])} else {$cells[2]}
        $expectedCount=[long]$cells[3]
        $v=$null
        $n=$null
        if ($i -eq 0) {$v=$guess.same;$n=$guess.initial}
        elseif ($guess.steps -is [array] -and $guess.steps.Count -ge $i) {
            $pair=$guess.steps[$i-1]
            if ($pair -is [array] -and $pair.Count -eq 2) {$v,$n=$pair} else {$schemaOK=$false}
        }
        $rows+=[ordered]@{history=$id;op=$ops[$i];value=$expectedValue;forecast=$v;value_match=(EqualScalar $v $expectedValue);
            count=$expectedCount;forecast_count=$n;count_match=(EqualScalar $n $expectedCount)}
    }
    $last=$rows[-1]
    $rejected=switch ($id) {
        'H1' {!(EqualScalar (!$rows[$firstRow].value) $rows[$firstRow].value)}
        'H2' {!(EqualScalar ($last.count+1) $last.count)}
        'H3' {!(EqualScalar ($last.value+'-altered') $last.value)}
    }
    if (!$rejected) {throw "Reversed observation accepted on $id"}
    $controls+=$id
}
if ($consumed -ne $raw.Count) {throw 'Surplus reference rows'}
$hit=@($rows|Where-Object value_match).Count+@($rows|Where-Object count_match).Count
$total=2*$rows.Count
$files=@($citations|Where-Object valid|ForEach-Object {$_.file}|Sort-Object -Unique)
$pass=!$diagnostic -and $schemaOK -and $hit -eq $total -and $files.Count -ge 2 -and @($citations|Where-Object {!$_.valid}).Count -eq 0
[ordered]@{scored=(Get-Date).ToString('o');machine_pass=$pass;correct=$hit;total=$total;controls=$controls;diagnostic=$diagnostic;
    state=$report.state;citations=$citations;rows=$rows}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath $dest -Encoding utf8NoBOM
if (!$pass) {exit 2}
exit 0
