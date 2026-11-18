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
$arms = 'original','parentheses','changed'
$head = "public CSVRecord next() {`n            if (isClosed())"
$sourceIds = [ordered]@{}
foreach ($arm in $arms) {
    $dir = "$root/$arm"
    Expand-Archive -LiteralPath $archive -DestinationPath "$dir/source"
    $file = "$dir/source/$rel"
    $body = [IO.File]::ReadAllText($file).Replace("`r`n", "`n")
    if ($arm -ne 'original') {
        $test = if ($arm -eq 'changed') { 'isClosed() && current == null' } else { '(isClosed())' }
        $body = ReplaceOne $body $head ($head.Replace('isClosed()', $test))
        [IO.File]::WriteAllText($file, $body)
    }
    $sourceIds[$arm] = Sha $file
}
$original = "$root/original/source/$rel"
foreach ($arm in 'parentheses','changed') {
    & $exe guard $original "$root/$arm/source/$rel" $solver "$root/$arm-query" *> "$root/$arm-solver.log"
    Require ($LASTEXITCODE -eq 0) "Solver failed on $arm (see $root/$arm-solver.log)"
}
$clean = Load "$root/parentheses-query/result.json"
$changed = Load "$root/changed-query/result.json"
Require ($null -eq $clean.witness -and ($clean.lengths.status -join ',') -ceq 'unsat,unsat,unsat,unsat,unsat') 'Clean control has a witness or missing verdict'
$trace = $changed.witness
Require ($trace.Count -gt 0 -and $trace.Count -le 4 -and $changed.lengths.Count -eq 5) 'Missing bounded witness'
$events = @($trace | ForEach-Object { $_.event })
$witnessId = $events -join ','
for ($i = 0; $i -le $trace.Count; $i++) {
    $expected = if ($i -lt $trace.Count) { 'unsat' } else { 'sat' }
    Require ($changed.lengths[$i].length -eq $i -and $changed.lengths[$i].status -ceq $expected) 'Shortest witness disagrees with verdicts'
}

$controls = [ordered]@{
    receiver = @($head, $head.Replace('isClosed()', 'CSVParser.this.isClosed()'), 'unsupported guard expression')
    disjunction = @($head, $head.Replace('isClosed()', 'isClosed() || current == null'), 'unsupported guard operator')
    effect = @('throw new NoSuchElementException("CSVParser has been closed");', 'current = null; throw new NoSuchElementException("CSVParser has been closed");', 'guard must only throw')
}
$body = [IO.File]::ReadAllText($original).Replace("`r`n", "`n")
foreach ($name in $controls.Keys) {
    $before, $after, $reason = $controls[$name]
    $file = "$root/$name.java"
    [IO.File]::WriteAllText($file, (ReplaceOne $body $before $after))
    & $exe guard $original $file $solver "$root/$name-query" *> "$root/$name.log"
    Require ($LASTEXITCODE -ne 0 -and !(Test-Path "$root/$name-query") -and
        ((Get-Content -Raw "$root/$name.log").Contains($reason))) "Unexpected guard control outcome on $name"
}
$seal = [ordered]@{ at=(Get-Date).ToString('o'); revision=$rev; sources=$sourceIds; selector=(Sha $exe); z3=(Sha $solver);
    observer=(Sha $observer); script=(Sha $PSCommandPath); jar=(Sha $jar); clean=(Sha "$root/parentheses-query/result.json");
    changed=(Sha "$root/changed-query/result.json"); witness=$trace }
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
    if ($arm -eq 'parentheses') { continue }
    & java.exe -cp "$dir/classes;$jar" GuardReplay witness @events > "$dir/witness.tsv" 2> "$dir/witness.stderr"
    Require ($LASTEXITCODE -eq 0) "Fresh witness replay failed on $arm"
    $side = if ($arm -eq 'original') { 'left' } else { 'right' }
    $expectedRows = @("$witnessId`tstart`tstart`t0")
    foreach ($step in $trace) { $expectedRows += "$witnessId`t$($step.event)`t$($step[$side].value)`t$($step[$side].count)" }
    $actualRows = @(Get-Content "$dir/witness.tsv")
    Require (($actualRows -join "`n") -ceq ($expectedRows -join "`n")) "Predicted trace differed on $arm"
}
Require ((Sha "$root/original/census.tsv") -ceq (Sha "$root/parentheses/census.tsv")) 'Clean JVM histories differed'
$left = @(Get-Content "$root/original/census.tsv")
$right = @(Get-Content "$root/changed/census.tsv")
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
$byLength = @()
for ($n=0; $n -le 4; $n++) {
    $ids = @($histories.Keys | Where-Object { $(if ($_ -eq 'root') {0} else {$_.Split(',').Count}) -eq $n })
    Require ($ids.Count -eq [math]::Pow(3,$n) -and @($ids | Where-Object { $histories[$_] -ne $n }).Count -eq 0) 'Missing JVM histories'
    $diff = @($ids | Where-Object { $different.ContainsKey($_) }).Count
    $verdict = if ($diff -gt 0) { 'sat' } else { 'unsat' }
    Require ($changed.lengths[$n].status -ceq $verdict) "Solver disagreed with JVM at length $n"
    $byLength += [ordered]@{length=$n;histories=$ids.Count;different=$diff;solver=$verdict}
}
Require ($different.ContainsKey($witnessId)) 'Fresh witness absent from JVM census'
Save "$root/result.json" ([ordered]@{ at=(Get-Date).ToString('o');lengths=$byLength;histories_per_arm=$histories.Count;
    rows_per_arm=$left.Count; clean_identical=$true; witness=$events; fresh_witness_matches=$true; different_histories=@($different.Keys | Sort-Object) })
Get-Content -Raw "$root/result.json"
