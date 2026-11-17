param([string]$Plan, [string]$Output)
$ErrorActionPreference = 'Stop'
$planData = Get-Content -LiteralPath $Plan -Raw | ConvertFrom-Json
$outRoot = [IO.Path]::GetFullPath($Output)
if (-not $outRoot.StartsWith('D:\BuildArtifacts\lesscode\model-simplification\ms1\') -or (Test-Path -LiteralPath $outRoot)) {
    throw 'Freeze requires a new directory in the experiment root'
}
New-Item -ItemType Directory -Path $outRoot | Out-Null
foreach ($group in $planData.input_groups) {
    $sourceRoot = (Resolve-Path -LiteralPath $group.root).Path
    foreach ($relative in $group.files) {
        $sourcePath = [IO.Path]::GetFullPath((Join-Path $sourceRoot $relative))
        if (-not $sourcePath.StartsWith($sourceRoot + '\')) { throw 'Input escapes its source root' }
        if ((Get-Item -LiteralPath $sourcePath).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Reparse input' }
        $destination = Join-Path $outRoot "$($group.name)\$relative"
        New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force | Out-Null
        Copy-Item -LiteralPath $sourcePath -Destination $destination
    }
}
foreach ($case in $planData.cases) {
    $context = foreach ($file in $case.context) {
        "`n--- $file ---`n" + [IO.File]::ReadAllText((Join-Path $outRoot "source\$file"))
    }
    $request = [ordered]@{protocol=1;purpose="H-MS1 $($case.id)";system=$planData.system;
        user=$case.instruction + ($context -join "`n");max_output_tokens=8192;temperature=$null;structure=$null}
    $request | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath "$outRoot\request-$($case.id).json" -Encoding utf8NoBOM
}
Copy-Item -LiteralPath $Plan -Destination "$outRoot\execution-plan.json"
$inventory = Get-ChildItem -LiteralPath $outRoot -Recurse -File | Sort-Object FullName | ForEach-Object {
    [ordered]@{path=[IO.Path]::GetRelativePath($outRoot,$_.FullName).Replace('\','/');
        size=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}
}
$manifest = [ordered]@{schema='h-ms1-freeze-v1';created_at=(Get-Date).ToUniversalTime().ToString('o');
    status='prepared-for-P1-not-qualified';plan='execution-plan.json';inputs=@($inventory);
    budgets=$planData.budgets;remaining_gate='P1 qualification and baseline ceilings before any live call'}
$manifest | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath "$outRoot\freeze.json" -Encoding utf8NoBOM
(Get-FileHash -LiteralPath "$outRoot\freeze.json" -Algorithm SHA256).Hash |
    Set-Content -LiteralPath "$outRoot\freeze.json.sha256" -Encoding ascii
