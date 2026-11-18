param(
    [Parameter(Mandatory)][ValidateSet('prepare','call')][string]$Stage,
    [Parameter(Mandatory)][string]$Run,
    [string]$Tracer
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = [IO.Path]::GetFullPath($Run)
if (!$root.StartsWith('D:\BuildArtifacts\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Expected an evidence directory under D:\BuildArtifacts' }
$frozen = 'D:/BuildArtifacts/lesscode/corrective-audit'
$model = 'D:/BuildArtifacts/lesscode/task-8.4/released/bin/lesscode-model.exe'
function Sha($file) { (Get-FileHash -LiteralPath $file).Hash }
function Save($path,$data) { $data | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $path -Encoding utf8NoBOM }
$arms = [ordered]@{X=@('fix','fixed'); Y=@('pre-fix','before')}
if ($Stage -eq 'prepare') {
    if (!$Tracer -or (Test-Path $root)) { throw 'Preparation requires a tracer and a new evidence directory' }
    $null = New-Item -ItemType Directory -Path $root
    $index = "$frozen/task68/evidence-sha256.json"
    if ((Sha $index) -ne 'F7C983F029B61DE02867DD81D9B0AC2C24D0C6DB54D9FB360AB91EDEDEC41C17') { throw 'Changed R142 evidence index' }
    $pins = @{}
    Get-Content $index -Raw | ConvertFrom-Json | ForEach-Object { $pins[$_.path]=$_.sha256 }
    $requests = [ordered]@{}
    foreach ($id in $arms.Keys) {
        $snapshot,$arm = $arms[$id]
        $dir = "$root/$id/input"
        $null = New-Item -ItemType Directory -Path $dir
        foreach ($item in 'overlay/sites.json','events/warm.jsonl','events/unchanged.jsonl') {
            $key = "trial/$arm/$item"
            $path = "$frozen/task68/$key"
            if ((Sha $path) -ne $pins[$key]) { throw "Early evidence differs at $key" }
            Copy-Item -LiteralPath $path -Destination $dir
        }
        $request = "$root/$id/request.json"
        & $Tracer forecast "$frozen/task67/$snapshot/native" $dir $request
        if ($LASTEXITCODE) { throw "Could not prepare $id" }
        if ((Get-Content $request -Raw) -match 'BuildArtifacts|corrective-audit|task6[789]|60f79f3|aa9d7d7|pre-fix|plain-fixed|verified\.json|[A-Z]:\\\\') { throw 'Identifying information in packet' }
        $requests[$id] = Sha $request
    }
    Save "$root/packets.json" ([ordered]@{time=(Get-Date).ToString('o'); tracer=(Sha $Tracer); requests=$requests})
    return
}
if ((Sha $model) -ne 'BC08C4AD24992DBD602DA5473604A85643FB51CC7622B55881FF363E003711F5') { throw 'Changed model CLI' }
$packets = Get-Content "$root/packets.json" -Raw | ConvertFrom-Json -AsHashtable
foreach ($id in $arms.Keys) {
    if ((Sha "$root/$id/request.json") -ne $packets.requests[$id] -or (Test-Path "$root/$id/started.json")) { throw 'Requests changed or already attempted' }
}
foreach ($id in $arms.Keys) {
    $dir = "$root/$id"
    $start = (Get-Date).ToString('o')
    Save "$dir/started.json" @{time=$start;request=$packets.requests[$id]}
    Get-Content "$dir/request.json" -Raw | & $model call --endpoint-env .env --answer-limit 300 --max-wait 30 --record "$dir/model.record" > "$dir/answer.json" 2> "$dir/model.log"
    $status = $LASTEXITCODE
    Save "$dir/seal.json" ([ordered]@{start=$start;end=(Get-Date).ToString('o');exit=$status;request=(Sha "$dir/request.json");answer=(Sha "$dir/answer.json")})
}
