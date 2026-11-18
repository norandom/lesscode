param(
    [Parameter(Mandatory)][string]$Selector,
    [Parameter(Mandatory)][string]$NewEvidence,
    [string]$Checkout = 'D:/Source/commons-csv',
    [string]$Z3 = "$PSScriptRoot/../../../../.runs/e1c-facdrone/extras/bin/z3.exe"
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = [IO.Path]::GetFullPath($NewEvidence)
if ((Test-Path -LiteralPath $root) -or !$root.StartsWith('D:\BuildArtifacts\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Choose a new evidence directory under D:\BuildArtifacts'
}
$null = New-Item -ItemType Directory -Path $root
$exe = (Resolve-Path -LiteralPath $Selector).Path
$solver = (Resolve-Path -LiteralPath $Z3).Path
function Require($condition, $message) { if (!$condition) { throw $message } }
function Sha($path) { (Get-FileHash -LiteralPath $path).Hash }
function Save($path, $value) { ConvertTo-Json -Depth 12 -InputObject $value | Set-Content -LiteralPath $path -Encoding utf8NoBOM }
function Load($path) { Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -AsHashtable }
function ReplaceOne($text, $before, $after) {
    Require ($text.Split([string[]]@($before), [StringSplitOptions]::None).Count -eq 2) "Expected one occurrence of $before"
    $text.Replace($before, $after)
}

$rev = '2bd66bf181d44ffc92806f585d3955e915666f7f'
$archive = "$root/source.zip"
git -C $Checkout archive --format=zip --output $archive $rev -- src/main/java LICENSE.txt NOTICE.txt pom.xml
Require ($LASTEXITCODE -eq 0 -and (Sha $archive) -eq 'F0A1BDF1451BAF64981D412B92B9FBF2717094E7218EE0CE206F1B0D4BBD8B04') 'Unexpected source archive'
$jar = "$env:USERPROFILE/.m2/repository/commons-io/commons-io/2.22.0/commons-io-2.22.0.jar"
Require ((Sha $jar) -eq '2B9A7B1F726FB86216DBD2C8321EABE0221DBD5B1BE81C18E1CB53811B104758') 'Unexpected Commons IO jar'
$observer = "$PSScriptRoot/GuardReplay.java"
$src = 'src/main/java/org/apache/commons/csv'
$rel = "$src/CSVParser.java"
$fill = "            if (current == null) {`n                current = getNextRecord();`n            }`n"
$ret = "            return current != null;"
$edits = [ordered]@{
    clean = @("$fill$ret", "            if ((null == current)) {`n                current = getNextRecord();`n            }`n            return null != current;")
    unconditional = @($fill, "            current = getNextRecord();`n")
    inverted = @($fill, $fill.Replace('current == null', 'current != null'))
}
$changedArms = 'unconditional','inverted'
$arms = @('original') + @($edits.Keys)
$sourceIds = [ordered]@{}
foreach ($arm in $arms) {
    $dir = "$root/$arm"
    Expand-Archive -LiteralPath $archive -DestinationPath "$dir/source"
    $file = "$dir/source/$rel"
    if ($arm -ne 'original') {
        $body = [IO.File]::ReadAllText($file).Replace("`r`n", "`n")
        [IO.File]::WriteAllText($file, (ReplaceOne $body $edits[$arm][0] $edits[$arm][1]))
    }
    $sourceIds[$arm] = Sha $file
}
$original = "$root/original/source/$rel"
foreach ($arm in $edits.Keys) {
    & $exe lookahead $original "$root/$arm/source/$rel" $solver "$root/$arm-query" *> "$root/$arm-solver.log"
    Require ($LASTEXITCODE -eq 0) "Solver failed on $arm (see $root/$arm-solver.log)"
}
$clean = Load "$root/clean-query/result.json"
Require ($null -eq $clean.witness -and ($clean.lengths.status -join ',') -ceq 'unsat,unsat,unsat,unsat,unsat') 'Clean control has a witness or missing verdict'
$queries = [ordered]@{}
foreach ($arm in $changedArms) {
    $q = Load "$root/$arm-query/result.json"
    $trace = $q.witness
    Require ($trace.Count -gt 0 -and $trace.Count -le 4 -and $q.lengths.Count -eq 5) "Missing bounded witness on $arm"
    for ($i = 0; $i -le $trace.Count; $i++) {
        $expected = if ($i -lt $trace.Count) { 'unsat' } else { 'sat' }
        Require ($q.lengths[$i].length -eq $i -and $q.lengths[$i].status -ceq $expected) "Shortest witness disagrees with verdicts on $arm"
    }
    $queries[$arm] = $q
}

# Each control edits the original and must be refused before a query exists.
$controls = [ordered]@{
    call = @("                current = getNextRecord();`n            }`n$ret", "                current = CSVParser.this.nextRecord();`n            }`n$ret", 'unsupported fill')
    statement = @($ret, "            recordCount++;`n$ret", 'unsupported hasNext body')
    return = @($ret, '            return current != null || isClosed();', 'unsupported return')
    outside = @('"No more CSV records available"', '"No more records"', 'source outside the modeled regions changed')
}
$body = [IO.File]::ReadAllText($original).Replace("`r`n", "`n")
foreach ($name in $controls.Keys) {
    $before, $after, $reason = $controls[$name]
    $file = "$root/control-$name.java"
    [IO.File]::WriteAllText($file, (ReplaceOne $body $before $after))
    & $exe lookahead $original $file $solver "$root/control-$name-query" *> "$root/control-$name.log"
    Require ($LASTEXITCODE -ne 0 -and !(Test-Path "$root/control-$name-query") -and
        ((Get-Content -Raw "$root/control-$name.log").Contains($reason))) "Unexpected control outcome on $name"
}
$seal = [ordered]@{ at=(Get-Date).ToString('o'); revision=$rev; sources=$sourceIds; selector=(Sha $exe); z3=(Sha $solver);
    observer=(Sha $observer); script=(Sha $PSCommandPath); jar=(Sha $jar); clean=(Sha "$root/clean-query/result.json");
    results=[ordered]@{}; witnesses=[ordered]@{} }
foreach ($arm in $changedArms) {
    $seal.results[$arm] = Sha "$root/$arm-query/result.json"
    $seal.witnesses[$arm] = $queries[$arm].witness
}
Save "$root/seal.json" $seal

& java.exe -version *> "$root/java-version.log"
Require ($LASTEXITCODE -eq 0 -and (Get-Content -Raw "$root/java-version.log") -match 'version "21[.]') 'Expected JDK 21'
& javac.exe -version *> "$root/javac-version.log"
Require ($LASTEXITCODE -eq 0 -and (Get-Content -Raw "$root/javac-version.log") -match 'javac 21[.]') 'Expected javac 21'
foreach ($arm in $arms) {
    $dir = "$root/$arm"
    Require ((Sha "$dir/source/$rel") -ceq $sourceIds[$arm]) 'Parser changed after sealing'
    $javaFiles = @(Get-ChildItem "$dir/source/$src/*.java" | Sort-Object Name)
    $null = New-Item -ItemType Directory -Path "$dir/classes"
    & javac.exe -proc:none -encoding UTF-8 -classpath $jar -d "$dir/classes" @($javaFiles.FullName) $observer *> "$dir/compile.log"
    Require ($LASTEXITCODE -eq 0) "Compilation failed on $arm"
    & java.exe -cp "$dir/classes;$jar" GuardReplay all > "$dir/census.tsv" 2> "$dir/census.stderr"
    Require ($LASTEXITCODE -eq 0) "JVM census failed on $arm"
}
# Replay each sealed witness in fresh processes on the original and its arm.
foreach ($arm in $changedArms) {
    $trace = $queries[$arm].witness
    $events = @($trace | ForEach-Object { $_.event })
    $witnessId = $events -join ','
    foreach ($side in 'left','right') {
        $build = if ($side -eq 'left') { 'original' } else { $arm }
        $out = "$root/$build/witness-$arm.tsv"
        & java.exe -cp "$root/$build/classes;$jar" GuardReplay witness @events > $out 2> "$out.stderr"
        Require ($LASTEXITCODE -eq 0) "Fresh witness replay failed on $build"
        $expectedRows = @("$witnessId`tstart`tstart`t0")
        foreach ($step in $trace) { $expectedRows += "$witnessId`t$($step.event)`t$($step[$side].value)`t$($step[$side].count)" }
        Require ((@(Get-Content $out) -join "`n") -ceq ($expectedRows -join "`n")) "Predicted $arm trace differed on $build"
    }
}
Require ((Sha "$root/original/census.tsv") -ceq (Sha "$root/clean/census.tsv")) 'Clean JVM histories differed'

function Census($leftPath, $rightPath) {
    $left = @(Get-Content $leftPath)
    $right = @(Get-Content $rightPath)
    Require ($left.Count -eq $right.Count) 'Incomplete census'
    $histories = @{}
    $different = @{}
    for ($i = 0; $i -lt $left.Count; $i++) {
        $a = $left[$i].Split("`t")
        $b = $right[$i].Split("`t")
        Require ($a.Count -eq 4 -and $b.Count -eq 4 -and $a[0] -ceq $b[0] -and $a[1] -ceq $b[1]) 'Misaligned census rows'
        $id = $a[0]
        $operations = if ($id -ne 'root') { $id.Split(',') }
        if ($a[1] -ceq 'start') {
            Require (!$histories.ContainsKey($id) -and $a[2] -ceq 'start' -and $a[3] -ceq '0') 'Duplicate or invalid start row'
            $histories[$id] = 0
        } else {
            Require ($histories.ContainsKey($id) -and $a[1] -cin @('hasNext','close','next') -and $a[1] -ceq @($operations)[$histories[$id]]) 'Unexpected census operation'
            $histories[$id]++
        }
        if ($left[$i] -cne $right[$i]) { $different[$id] = @($operations).Count }
    }
    [pscustomobject]@{ histories=$histories; different=$different; rows=$left.Count }
}
$summary = [ordered]@{}
foreach ($arm in $changedArms) {
    $c = Census "$root/original/census.tsv" "$root/$arm/census.tsv"
    $byLength = @()
    for ($n = 0; $n -le 4; $n++) {
        $ids = @($c.histories.Keys | Where-Object { $(if ($_ -eq 'root') {0} else {$_.Split(',').Count}) -eq $n })
        Require ($ids.Count -eq [math]::Pow(3,$n) -and @($ids | Where-Object { $c.histories[$_] -ne $n }).Count -eq 0) 'Missing JVM histories'
        $diff = @($ids | Where-Object { $c.different.ContainsKey($_) }).Count
        $verdict = if ($diff -gt 0) { 'sat' } else { 'unsat' }
        Require ($queries[$arm].lengths[$n].status -ceq $verdict) "Solver disagreed with JVM at length $n on $arm"
        $byLength += [ordered]@{length=$n;histories=$ids.Count;different=$diff;solver=$verdict}
    }
    $witnessId = @($queries[$arm].witness | ForEach-Object { $_.event }) -join ','
    Require ($c.different.ContainsKey($witnessId)) "Fresh witness absent from JVM census on $arm"
    $summary[$arm] = [ordered]@{ lengths=$byLength; histories=$c.histories.Count; rows=$c.rows; witness=$witnessId;
        different_histories=@($c.different.Keys | Sort-Object) }
}
Save "$root/result.json" ([ordered]@{ at=(Get-Date).ToString('o'); clean_identical=$true; fresh_witnesses_match=$true;
    controls_refused=@($controls.Keys); arms=$summary })
Get-Content -Raw "$root/result.json"
