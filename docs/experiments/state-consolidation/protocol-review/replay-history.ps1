param(
    [Parameter(Mandatory)][string]$Driver,
    [Parameter(Mandatory)][string]$ToyCheck,
    [Parameter(Mandatory)][string]$NewEvidence,
    [Parameter(Mandatory)][ValidateSet('before','after')][string]$Revision
)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$root=[IO.Path]::GetFullPath($NewEvidence)
if (!$root.StartsWith('D:\BuildArtifacts\',[StringComparison]::OrdinalIgnoreCase) -or (Test-Path $root)) {
    throw 'Use a fresh evidence directory under D:\BuildArtifacts'
}
$exe=(Resolve-Path $Driver).Path
$test=(Resolve-Path $ToyCheck).Path
$null=New-Item -ItemType Directory -Path "$root/project/src","$root/project/tests"
$app="def step(state, a):`n    if a > 3:`n        raise ValueError('big')`n    state.total = a`n    return a`n"
$subject="$root/project/src/app.py"
$expect="$root/project/tests/check.txt"
[IO.File]::WriteAllText($subject,$app)
[IO.File]::WriteAllText($expect,"def step(`n")
$config="$root/config.json"
@{language='python';source_root='project';files=@('src/app.py');test_paths=@('tests/');workers=1;
  test=@($test,'--file','src/app.py','--expect-from','tests/check.txt','--print','TOY RAN');
  timeout_floor_s=10;timeout_ceiling_s=60} | ConvertTo-Json -Depth 4 | Set-Content $config -Encoding utf8NoBOM
function Require($condition,$message){if(!$condition){throw $message}}
$rows=@()
foreach($phase in 'warm','unchanged','changed','fresh'){
    if($phase -eq 'changed'){[IO.File]::WriteAllText($expect,"if a > 3:`n")}
    $work=if($phase -eq 'fresh'){"$root/new-work"}else{"$root/same-work"}
    $receipt="$root/$phase.json"
    $argv=@('deconvolute','run','--config',$config,'--work',$work,'--out',$receipt)
    $start=Get-Date
    & $exe @argv > "$root/$phase.stdout" 2> "$root/$phase.stderr"
    $rc=$LASTEXITCODE
    @{exe=$exe;argv=$argv;cwd=(Get-Location).Path;started=$start.ToString('o');ended=(Get-Date).ToString('o');exit=$rc} |
        ConvertTo-Json | Set-Content "$root/$phase.command.json"
    Require ($rc -eq 0) "Invalid $phase driver run, exit $rc"
    $r=Get-Content $receipt -Raw | ConvertFrom-Json
    Require (!$r.incomplete -and $null -eq $r.void) "Incomplete $phase receipt"
    $selected=@($r.mutants | Where-Object { $_.operator -ceq 'negate' -and $_.site.start_line -eq 2 })
    Require ($selected.Count -eq 1) 'Expected a unique line-2 condition inversion'
    $worker=@(Get-ChildItem -Directory $work | Where-Object Name -match '^w\d+$' | Get-ChildItem -Recurse -File -Filter check.txt)
    Require ($worker.Count -eq 1) 'Expected one worker check file'
    $rows += [pscustomobject]@{phase=$phase;class=$selected[0].class;reused=[bool]$selected[0].reused_from;
        test_identity=$r.test_files_sha256;config_sha=$r.config_sha256;source_sha=(Get-FileHash $subject).Hash;
        project_test=(Get-FileHash $expect).Hash;worker_test=(Get-FileHash $worker[0].FullName).Hash;
        worker_file=$worker[0].FullName;mutants=@($r.mutants).Count}
}
$rows | ConvertTo-Json | Set-Content "$root/history.json"
$warm,$same,$changed,$fresh=$rows
Require ($warm.class -ceq 'survived' -and $same.class -ceq 'survived' -and $fresh.class -ceq 'killed') 'Unexpected warm, unchanged or fresh result'
Require (!$warm.reused -and $same.reused -and !$changed.reused -and !$fresh.reused) 'Unexpected outcome reuse'
Require ($warm.test_identity -and $warm.test_identity -eq $same.test_identity -and $changed.test_identity -ne $same.test_identity -and $changed.test_identity -eq $fresh.test_identity) 'Invalid test identity control'
Require (@($rows.config_sha | Sort-Object -Unique).Count -eq 1 -and @($rows.source_sha | Sort-Object -Unique).Count -eq 1) 'Config or source drifted'
Require ($fresh.worker_test -eq $fresh.project_test -and $warm.worker_test -eq $warm.project_test -and $changed.project_test -ne $warm.project_test) 'Invalid byte control'
if ($Revision -eq 'before') {
    Require ($changed.class -ceq 'survived' -and $changed.worker_test -eq $warm.project_test) 'Pre-fix stale-worker prediction failed'
} else {
    Require ($changed.class -ceq 'killed' -and $changed.worker_test -eq $changed.project_test) 'Fixed worker did not run the changed tests'
}
$rows | Format-Table phase,class,reused
exit 0
