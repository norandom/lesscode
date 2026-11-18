param(
    [Parameter(Mandatory)][string]$Selector,
    [Parameter(Mandatory)][string]$NewEvidence,
    [string]$Checkout='D:/Source/commons-csv',
    [string]$Previous='D:/BuildArtifacts/lesscode/corrective-audit/task70'
)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=[IO.Path]::GetFullPath($NewEvidence)
if ((Test-Path $root) -or !$root.StartsWith('D:\BuildArtifacts\',[StringComparison]::OrdinalIgnoreCase)) {throw 'Choose a new evidence directory under D:\BuildArtifacts'}
$null=New-Item -ItemType Directory -Path $root
$exe=(Resolve-Path -LiteralPath $Selector).Path
function Sha($file) {(Get-FileHash -LiteralPath $file).Hash}
function Json($file) {Get-Content -LiteralPath $file -Raw|ConvertFrom-Json -AsHashtable}
function Save($file,$value) {ConvertTo-Json -InputObject $value -Depth 10|Set-Content -LiteralPath $file -Encoding utf8NoBOM}
function Require($condition,$message) {if (!$condition) {throw $message}}
function ReplaceOnce($file,$prior,$new) {
    $body=[IO.File]::ReadAllText($file)
    Require ($body.Split([string[]]@($prior),[StringSplitOptions]::None).Count -eq 2) "Expected one occurrence of $prior"
    [IO.File]::WriteAllText($file,$body.Replace($prior,$new))
}
function NumericFact($inventory,$owner,$member) {
    $facts=@($inventory.initializers|Where-Object {$_.target -ceq "$owner.$member" -and $_.reference -ceq "builder.$member"})
    Require ($facts.Count -eq 1 -and $facts[0].declaration.owner -ceq "$owner.Builder") "Missing typed field $owner.$member"
    $fact=$facts[0]
    foreach ($line in $fact.assignment_line,$fact.parameter_line) {
        Require ($inventory.lines[$fact.assignment_file].Contains([string]$line)) 'Missing assignment or parameter in packet'
    }
    Require ($inventory.lines[$fact.declaration.file].Contains([string]$fact.declaration.line)) 'Missing field declaration in packet'
    return $fact
}

# This known iterator model is the R144 reference for testing source-derived
# initialization. It does not supply the default values or discover transitions.
function Forecast([long]$initial,[long]$limit) {
    $records=@('oak','elm')
    $histories=[ordered]@{H1='hasNextA,hasNextA,nextB,nextA,hasNextB,nextB';H2='nextB,hasNextA,hasNextB,nextB,nextA';H3='hasNextA,close,hasNextB,nextA'}
    foreach ($id in $histories.Keys) {
        $read=0
        $pending=$null
        $closed=$false
        "$id`tstart`ttrue`t$initial"
        foreach ($op in $histories[$id].Split(',')) {
            if ($op -ceq 'close') {$closed=$true; $value='closed'}
            else {
                if (!$closed -and $null -eq $pending -and $read -lt $records.Count -and ($limit -le 0 -or $read -lt $limit)) {
                    $pending=$records[$read++]
                }
                $available=!$closed -and $null -ne $pending
                if ($op.StartsWith('hasNext')) {$value=$available.ToString().ToLowerInvariant()}
                elseif ($available) {$value=$pending; $pending=$null}
                else {$value='throws:NoSuchElementException'}
            }
            "$id`t$op`t$value`t$($initial+$read)"
        }
    }
}
$revision='2bd66bf181d44ffc92806f585d3955e915666f7f'
$archive="$root/source.zip"
git -C $Checkout archive --format=zip --output $archive $revision -- src/main/java LICENSE.txt NOTICE.txt pom.xml
Require ($LASTEXITCODE -eq 0 -and (Sha $archive) -eq 'F0A1BDF1451BAF64981D412B92B9FBF2717094E7218EE0CE206F1B0D4BBD8B04') 'Historical archive changed'
$relative='src/main/java/org/apache/commons/csv'
$oracle="$PSScriptRoot/Observation.java"
$jar="$env:USERPROFILE/.m2/repository/commons-io/commons-io/2.22.0/commons-io-2.22.0.jar"
Require ((Sha $oracle) -eq 'BA37815A631957C21B8AC0D00830F70A0785A48E03021E349E27D8E9C8AD419A' -and (Sha $jar) -eq '2B9A7B1F726FB86216DBD2C8321EABE0221DBD5B1BE81C18E1CB53811B104758') 'Observer or dependency changed'
$arms=[ordered]@{baseline=@(0,0);counter=@(6,0);limit=@(0,1)}
$predictions=[ordered]@{}
foreach ($arm in $arms.Keys) {
    $dir="$root/$arm"
    Expand-Archive -LiteralPath $archive -DestinationPath "$dir/source"
    $src="$dir/source/$relative"
    if ($arm -eq 'counter') {ReplaceOnce "$src/CSVParser.java" 'private long recordNumber = 1;' 'private long recordNumber = 7;'}
    if ($arm -eq 'limit') {ReplaceOnce "$src/CSVFormat.java" 'private long maxRows;' 'private long maxRows = 1;'}
    & $exe $src "$dir/packet"
    Require ($LASTEXITCODE -eq 0) 'Selector failed'
    $index=Json "$dir/packet/selection.json"
    $counter=NumericFact $index CSVParser recordNumber
    $cap=NumericFact $index CSVFormat maxRows
    $initial=$counter.value_at_declaration_default
    $limit=$cap.value_at_declaration_default
    Require ($null -ne $initial -and $null -ne $limit -and $initial -eq $arms[$arm][0] -and $limit -eq $arms[$arm][1]) "Wrong numeric facts for $arm"
    $default=@($index.initializers|Where-Object {$_.target -ceq 'CSVParser.format' -and $_.reference -ceq 'CSVFormat.DEFAULT'})
    Require ($default.Count -eq 1 -and $default[0].declaration.factories.Count -eq 1 -and $default[0].declaration.factories[0].method -ceq 'CSVFormat.Builder.create') 'Default factory unresolved'
    @(Forecast $initial $limit)|Set-Content "$dir/forecast.tsv" -Encoding utf8NoBOM
    $predictions[$arm]=[ordered]@{counter=$counter;row_limit=$cap;default_format=$default[0];selection=(Sha "$dir/packet/selection.json");forecast=(Sha "$dir/forecast.tsv")}
}
Save "$root/seal.json" ([ordered]@{at=(Get-Date).ToString('o');revision=$revision;selector=(Sha $exe);observer=(Sha $oracle);jar=(Sha $jar);script=(Sha $PSCommandPath);arms=$predictions})
$summary=[ordered]@{}
foreach ($arm in $arms.Keys) {
    $dir="$root/$arm"
    $src="$dir/source/$relative"
    $seal=(Json "$root/seal.json").arms[$arm]
    Require ((Sha "$dir/packet/selection.json") -eq $seal.selection -and (Sha "$dir/forecast.tsv") -eq $seal.forecast) 'Sealed prediction changed'
    $index=Json "$dir/packet/selection.json"
    $sources=@(Get-ChildItem "$src/*.java"|Sort-Object Name)
    Require ($sources.Count -eq $index.hashes.Count) 'Java source population changed'
    foreach ($f in $sources) {Require ((Sha $f.FullName) -eq $index.hashes[$f.Name]) 'Java source differs from extraction'}
    $null=New-Item -ItemType Directory -Path "$dir/classes"
    & javac.exe -proc:none -encoding UTF-8 -classpath $jar -d "$dir/classes" @($sources.FullName) $oracle *> "$dir/compile.log"
    Require ($LASTEXITCODE -eq 0) "Compilation failed on $arm"
    & java.exe -cp "$dir/classes;$jar" Observation > "$dir/observed.tsv" 2> "$dir/stderr.txt"
    Require ($LASTEXITCODE -eq 0) "Observation failed on $arm"
    $observed=@(Get-Content "$dir/observed.tsv")
    $expected=@(Get-Content "$dir/forecast.tsv")
    Require ($expected.Count -eq 18 -and $observed.Count -eq $expected.Count) 'Missing history or operation'
    $fields=0
    for ($i=0;$i -lt $expected.Count;$i++) {
        $a=$expected[$i].Split("`t"); $b=$observed[$i].Split("`t")
        Require ($a.Count -eq 4 -and $b.Count -eq 4 -and $a[0] -ceq $b[0] -and $a[1] -ceq $b[1]) 'Mismatched history or operation'
        foreach ($column in 2,3) {if ($a[$column] -ceq $b[$column]) {$fields++}}
    }
    $summary[$arm]=[ordered]@{observed_at=(Get-Date).ToString('o');observations=(Sha "$dir/observed.tsv");correct=$fields;total=$expected.Count*2}
}
Save "$root/results.json" $summary
foreach ($arm in $arms.Keys) {Require ($summary[$arm].correct -eq $summary[$arm].total) "Behavioral mismatch on $arm"}

$selection="$Previous/delivery/packet/selection.json"
$answer="$Previous/delivery/answer.json"
Require ((Sha $selection) -eq 'CCB0CE46F2F855130708E10391A920AEAD83B82E274CF15ECCFFBF952AF28575' -and (Sha $answer) -eq 'F1A014EC856B76ABBB2C9D5A6F606D3C3949F598A9DB4DD5795799D7D9930159') 'Historical citation inputs changed'
& $exe citations $selection $answer "$root/citations.json"
Require ($LASTEXITCODE -eq 0) 'Historical quotation unresolved'
$rows=(Json "$root/citations.json").citations
Require (@($rows|Where-Object status -ceq exact).Count -eq 20 -and @($rows|Where-Object status -ceq relocated).Count -eq 2) 'Historical resolution mismatch'

# Make each nested Builder a plausible but incorrect source for the other
# field, and test unsupported Java expressions on real copies of the package.
$controls=[ordered]@{}
foreach ($check in 'other-owner','expression') {
    $dir="$root/$check"
    $null=New-Item -ItemType Directory -Path "$dir/src"
    Copy-Item "$root/baseline/source/$relative/*.java" "$dir/src"
    if ($check -eq 'other-owner') {
        ReplaceOnce "$dir/src/CSVParser.java" 'private long recordNumber = 1;' "private long recordNumber = 1;`n        private long maxRows = 42;"
        ReplaceOnce "$dir/src/CSVFormat.java" 'private long maxRows;' "private long maxRows;`n        private long recordNumber = 99;"
    } else {
        ReplaceOnce "$dir/src/CSVParser.java" 'private long recordNumber = 1;' 'private long recordNumber = 1 + 6;'
        ReplaceOnce "$dir/src/CSVFormat.java" 'this.maxRows = builder.maxRows;' 'this.maxRows = builder.maxRows + 1;'
    }
    & $exe "$dir/src" "$dir/packet"
    Require ($LASTEXITCODE -eq 0) 'Source control did not parse'
    $inventory=Json "$dir/packet/selection.json"
    $a=NumericFact $inventory CSVParser recordNumber
    $b=NumericFact $inventory CSVFormat maxRows
    if ($check -eq 'other-owner') {
        Require ($a.declaration.long_default -eq 1 -and $b.declaration.long_default -eq 0) 'Selector used the wrong nested owner'
    } else {
        Require ($null -eq $a.declaration.long_default -and $null -eq $a.value_at_declaration_default -and $null -eq $b.value_at_declaration_default) 'Unsupported expression got a numeric value'
    }
    $controls[$check]=@($a,$b)
}
$quotes=@(
    @{file='CSVParser.java';line=1;quote='if (isClosed()) {'},
    @{file='CSVParser.java';line=1;quote='no_such_source_expression();'},
    @{file='CSVFormat.java';line=905;quote='return csvRecordIterator;'},
    @{file='unknown.java';line=905;quote='return csvRecordIterator;'}
)
Save "$root/invalid-citations.json" @{ok=$true;reply=@{stop='complete';text=(ConvertTo-Json -Depth 4 -InputObject @{evidence=$quotes})}}
& $exe citations $selection "$root/invalid-citations.json" "$root/unresolved.json" 2> "$root/unresolved.log"
$rc=$LASTEXITCODE
$refused=Json "$root/unresolved.json"
Require ($rc -ne 0 -and !$refused.all_resolved -and ($refused.citations.status -join ',') -ceq 'ambiguous,missing,missing,unknown_file') 'Wrong citation control result'
Require ($refused.citations[0].matching_lines.Count -gt 1 -and @($refused.citations|Where-Object {$null -ne $_.resolved_line}).Count -eq 0) 'Guessed an unresolved location'
$controls['citation_status']=$refused.citations.status
Save "$root/controls.json" $controls
$summary | ConvertTo-Json -Depth 4
exit 0
