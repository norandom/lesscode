param(
    [Parameter(Mandatory)][ValidateSet('prepare','call','observe')][string]$Stage,
    [Parameter(Mandatory)][string]$Run,
    [string]$Selector,
    [string]$Checkout='D:/Source/commons-csv',
    [ValidateSet(30,180)][int]$MaxWait=30
)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=[IO.Path]::GetFullPath($Run)
if (!$root.StartsWith('D:\BuildArtifacts\',[StringComparison]::OrdinalIgnoreCase)) {throw 'Choose an evidence directory under D:\BuildArtifacts'}
$revision='2bd66bf181d44ffc92806f585d3955e915666f7f'
$package="$root/source/src/main/java/org/apache/commons/csv"
function Sha($f) {(Get-FileHash -LiteralPath $f).Hash}
function Save($f,$data) {$data|ConvertTo-Json -Depth 6|Set-Content -LiteralPath $f -Encoding utf8NoBOM}
if ($Stage -eq 'prepare') {
    if ((Test-Path $root) -or !$Selector) {throw 'Preparation requires a fresh directory and the compiled selector'}
    $null=New-Item -ItemType Directory -Path $root
    git -C $Checkout archive --format=zip --output "$root/source.zip" $revision -- src/main/java LICENSE.txt NOTICE.txt pom.xml
    if ($LASTEXITCODE) {throw 'Git source export failed'}
    Expand-Archive -LiteralPath "$root/source.zip" -DestinationPath "$root/source"
    & $Selector $package "$root/packet"
    if ($LASTEXITCODE) {throw 'Could not build source packet'}
    Save "$root/inputs.json" ([ordered]@{time=(Get-Date).ToString('o');revision=$revision;archive=(Sha "$root/source.zip");selector=(Sha $Selector);
        request=(Sha "$root/packet/request.json");selection=(Sha "$root/packet/selection.json");oracle=(Sha "$PSScriptRoot/Observation.java");scorer=(Sha "$PSScriptRoot/score.ps1")})
    return
}
$inputPins=Get-Content "$root/inputs.json" -Raw|ConvertFrom-Json -AsHashtable
if ($inputPins.revision -ne $revision -or (Sha "$root/packet/request.json") -ne $inputPins.request -or (Sha "$root/packet/selection.json") -ne $inputPins.selection) {throw 'Frozen packet changed'}
if ($Stage -eq 'call') {
    if (Test-Path "$root/call-start.json") {throw 'Forecast was already attempted'}
    $model='D:/BuildArtifacts/lesscode/task-8.4/released/bin/lesscode-model.exe'
    if ((Sha $model) -ne 'BC08C4AD24992DBD602DA5473604A85643FB51CC7622B55881FF363E003711F5') {throw 'Model CLI differs from R143'}
    Save "$root/call-start.json" @{time=(Get-Date).ToString('o');request=$inputPins.request;max_wait=$MaxWait}
    Get-Content "$root/packet/request.json" -Raw | & $model call --endpoint-env .env --answer-limit 300 --max-wait $MaxWait --record "$root/model.record" > "$root/answer.json" 2> "$root/model.log"
    $rc=$LASTEXITCODE
    Save "$root/seal.json" ([ordered]@{time=(Get-Date).ToString('o');exit=$rc;answer=(Sha "$root/answer.json");request=$inputPins.request})
    return
}
$seal=Get-Content "$root/seal.json" -Raw|ConvertFrom-Json -AsHashtable
if ((Sha "$root/answer.json") -ne $seal.answer -or $seal.request -ne $inputPins.request -or (Sha "$PSScriptRoot/Observation.java") -ne $inputPins.oracle) {throw 'Answer or oracle changed'}
$out="$root/jvm"
if (Test-Path $out) {throw 'Choose a fresh trial for another observation'}
$null=New-Item -ItemType Directory -Path "$out/classes"
$jar="$env:USERPROFILE/.m2/repository/commons-io/commons-io/2.22.0/commons-io-2.22.0.jar"
if ((Sha $jar) -ne '2B9A7B1F726FB86216DBD2C8321EABE0221DBD5B1BE81C18E1CB53811B104758') {throw 'Commons IO dependency differs'}
$sources=@(Get-ChildItem -LiteralPath $package -File -Filter '*.java'|Sort-Object Name)
$hashes=(Get-Content "$root/packet/selection.json" -Raw|ConvertFrom-Json -AsHashtable).hashes
if ($sources.Count -ne $hashes.Count) {throw 'Source population changed'}
foreach ($f in $sources) {if ((Sha $f.FullName) -ne $hashes[$f.Name]) {throw "Changed Java source $($f.Name)"}}
$start=(Get-Date).ToString('o')
$javac=(Get-Command javac.exe).Source
$java=(Get-Command java.exe).Source
& $javac -proc:none -encoding UTF-8 -classpath $jar -d "$out/classes" @($sources.FullName) "$PSScriptRoot/Observation.java" *> "$out/compile.log"
if ($LASTEXITCODE) {throw "Compilation failed. See $out/compile.log"}
& $java -cp "$out/classes;$jar" Observation > "$out/observed.tsv" 2> "$out/stderr.txt"
if ($LASTEXITCODE) {throw 'Oracle process failed'}
Save "$out/run.json" ([ordered]@{start=$start;end=(Get-Date).ToString('o');exit=$LASTEXITCODE;observations=(Sha "$out/observed.tsv");answer=$seal.answer;
    jar=(Sha $jar);javac=$javac;compiler_version=(& $javac -version);java=$java;java_version=(& $java --version|Select-Object -First 1)})
