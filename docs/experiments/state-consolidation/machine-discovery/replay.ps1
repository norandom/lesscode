param(
    [Parameter(Mandatory)][string]$Binary,
    [Parameter(Mandatory)][string]$NewEvidence,
    [string]$Fixture = 'D:/BuildArtifacts/lesscode/fitness60-concept'
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$exe = (Resolve-Path $Binary).Path
$python = (Resolve-Path "$repo/.venv-task11/Scripts/python.exe").Path
$z3 = (Resolve-Path "$repo/.runs/e1c-facdrone/extras/bin/z3.exe").Path
$out = [IO.Path]::GetFullPath($NewEvidence)
if (!$out.StartsWith('D:\BuildArtifacts\', [StringComparison]::OrdinalIgnoreCase) -or (Test-Path $out)) {
    throw 'Use a new directory under D:\BuildArtifacts'
}
$null = New-Item -ItemType Directory -Path $out
function Run-Recorded($label, $program, [string[]]$arguments, [int]$expected) {
    $start = Get-Date
    & $program @arguments > "$out/$label.stdout" 2> "$out/$label.stderr"
    $code = $LASTEXITCODE
    @{cwd=(Get-Location).Path; program=$program; args=$arguments; started=$start.ToString('o')
      ended=(Get-Date).ToString('o'); exit=$code} | ConvertTo-Json | Set-Content "$out/$label.command.json"
    if ($code -ne $expected) { throw "$label exited $code, expected $expected. See $out" }
}
function Require($condition, $message) { if (!$condition) { throw $message } }
function Check-Sites($value, $package) {
    if ($value -is [Collections.IDictionary]) {
        if ($value.Contains('line') -and $value.Contains('file')) {
            $lines=[IO.File]::ReadAllLines((Join-Path $package $value.file))
            Require ($value.line -gt 0 -and $value.line -le $lines.Length -and $lines[$value.line-1].Contains($value.text)) "Bad source citation in $package at $($value.file) line $($value.line)"
        } else {
            foreach ($item in $value.Values) { Check-Sites $item $package }
        }
    } elseif ($value -is [array]) {
        foreach ($item in $value) { Check-Sites $item $package }
    }
}
function Edit-Once($file,$before,$after) {
    $s=[IO.File]::ReadAllText($file)
    Require (($s.Split([string[]]@($before),[StringSplitOptions]::None)).Count -eq 2) "Ambiguous edit in $file"
    [IO.File]::WriteAllText($file,$s.Replace($before,$after))
}

$index = "$Fixture/result.json"
Require ((Get-FileHash $index).Hash -eq '52C9FE089D40AE6A6B60F9F53E8A90541A2F4EA1B64BAF22D1FFFD1CDB05604F') 'R130 manifest changed'
$frozen = (Get-Content $index -Raw | ConvertFrom-Json -AsHashtable).sources
$clean = "$out/specimen"
foreach ($arm in 'implicit','explicit') {
    $null = New-Item -ItemType Directory -Path "$clean/$arm" -Force
}
foreach ($file in $frozen.Keys | Where-Object { $_.EndsWith('.py') }) {
    Copy-Item -LiteralPath "$Fixture/$file" -Destination "$clean/$file"
    Require ((Get-FileHash "$clean/$file").Hash -eq $frozen[$file]) "Frozen source changed at $file"
}
foreach ($arm in 'implicit','explicit') {
    Run-Recorded "extract-$arm" $exe @("$clean/$arm") 0
    Check-Sites (Get-Content "$out/extract-$arm.stdout" -Raw | ConvertFrom-Json -AsHashtable) "$clean/$arm"
}
$hidden = Get-Content "$out/extract-implicit.stdout" -Raw | ConvertFrom-Json
$table = Get-Content "$out/extract-explicit.stdout" -Raw | ConvertFrom-Json
Require ($hidden.entity -ceq 'job.Job' -and $hidden.field -ceq 'phase' -and $hidden.initial.value -ceq 'idle') 'Incorrect entity or initial value'
Require (($hidden.states -join ',') -ceq 'done,idle,running') 'Incorrect states'
Require ($hidden.rule_location -ceq 'distributed literal guards' -and $table.rule_location -ceq 'table rules.TRANSITIONS') 'Incorrect rule owners'
foreach ($m in $hidden,$table) {
    Require (($m.events.psobject.Properties.Name -join ',') -ceq 'begin,finish') 'Incorrect events'
    Require ($m.events.begin.from.value -ceq 'idle' -and $m.events.begin.to.value -ceq 'running') 'Incorrect begin rule'
    Require ($m.events.finish.from.value -ceq 'running' -and $m.events.finish.to.value -ceq 'done') 'Incorrect finish rule'
}
Require ($hidden.initial_write.line -eq 6 -and $hidden.events.begin.guard.file -ceq 'start.py' -and $hidden.events.finish.guard.file -ceq 'finish.py') 'Incorrect scattered sites'
Require ($hidden.events.finish.guard.line -eq 3 -and $hidden.events.finish.write.line -eq 5) 'Incorrect guard or write line'

$renamed = New-Item -ItemType Directory -Path "$out/renamed"
foreach ($p in Get-ChildItem "$clean/implicit/*.py") {
    $s = [IO.File]::ReadAllText($p.FullName)
    $s = $s.Replace('Job','Parcel').Replace('Begin','Starter').Replace('Finish','Closer').Replace('begin','admit').Replace('start','intake').Replace('finish','close')
    $s = $s.Replace('phase','stage').Replace('idle','waiting').Replace('running','moving').Replace('done','filed')
    $file = $p.Name.Replace('job','parcel').Replace('start','intake').Replace('finish','close')
    [IO.File]::WriteAllText((Join-Path $renamed.FullName $file),$s)
}
Run-Recorded 'renamed' $exe @($renamed.FullName) 0
$m = Get-Content "$out/renamed.stdout" -Raw | ConvertFrom-Json
Require ($m.entity -ceq 'parcel.Parcel' -and $m.field -ceq 'stage' -and $m.initial.value -ceq 'waiting') 'Renamed entity lost'
Require ($m.events.admit.from.value -ceq 'waiting' -and $m.events.admit.to.value -ceq 'moving' -and $m.events.close.from.value -ceq 'moving' -and $m.events.close.to.value -ceq 'filed') 'Renamed edges lost'
foreach ($control in 'extra-effect','missing-base') { Copy-Item "$clean/implicit" "$out/$control" -Recurse }
Edit-Once "$out/extra-effect/start.py" 'return True' "notify()`n        return True"
Edit-Once "$out/missing-base/job.py" 'from .start import Begin' 'from .absent import Begin'
foreach ($control in 'extra-effect','missing-base') {
    Run-Recorded $control $exe @("$out/$control") 1
}
Require ((Get-Content "$out/extra-effect.stderr" -Raw) -match 'unsupported method body') 'Unexpected effect diagnostic'
Require ((Get-Content "$out/missing-base.stderr" -Raw) -match 'missing module absent') 'Unexpected ownership diagnostic'

$adverse = "$out/changed"
Copy-Item $clean $adverse -Recurse
Edit-Once "$adverse/explicit/rules.py" '"finish": (RUNNING, "done")' '"finish": (INITIAL, "done")'
Run-Recorded 'compare-clean' $exe @("$clean/implicit","$clean/explicit",$z3,"$out/clean-query") 0
Run-Recorded 'compare-planted' $exe @("$clean/implicit","$adverse/explicit",$z3,"$out/planted-query") 2
$normal = Get-Content "$out/clean-query/result.json" -Raw | ConvertFrom-Json
$fault = Get-Content "$out/planted-query/result.json" -Raw | ConvertFrom-Json
Require ($normal.histories -eq 31 -and $normal.lengths.Count -eq 5 -and @($normal.lengths | Where-Object solver -ne 'unsat').Count -eq 0) 'Clean bounded comparison failed'
$history = @($fault.witness.events)
Require ($history.Count -eq 1 -and $history[0] -ceq 'finish') 'Wrong shortest solver witness'

# The frozen oracle numbers histories by length, then product(begin, finish).
$ordinal = 0
foreach ($event in $history) { $ordinal = 2 * $ordinal + 1 + [int]($event -ceq 'finish') }
$test = "test_histories.py::test_observed_transitions[Job1-history$ordinal]"
$previousPath = $env:PYTHONPATH
$previousBytecode = $env:PYTHONDONTWRITEBYTECODE
foreach ($run in 'suite','baseline-witness','clean-witness','planted-witness') {
    $root = if ($run -eq 'planted-witness') { $adverse } else { $clean }
    $selected = switch ($run) {
        'suite' { 'test_histories.py' }
        'baseline-witness' { $test.Replace('Job1','Job0') }
        default { $test }
    }
    Push-Location -LiteralPath $root
    try {
        $env:PYTHONPATH=$root
        $env:PYTHONDONTWRITEBYTECODE='1'
        $argsTest=@('-B','-m','pytest','-q','-n','2','--dist','loadfile','-p','no:cacheprovider','--rootdir',$root,'-c','NUL',
            '--basetemp',"$out/$run-tmp",'--junitxml',"$out/$run.xml",$selected)
        Run-Recorded $run $python $argsTest ([int]($run -eq 'planted-witness'))
    } finally {
        $env:PYTHONPATH=$previousPath
        $env:PYTHONDONTWRITEBYTECODE=$previousBytecode
        Pop-Location
    }
    $suite=([xml](Get-Content "$out/$run.xml" -Raw)).testsuites.testsuite
    $count=if ($run -eq 'suite') {63} else {1}
    Require ([int]$suite.tests -eq $count -and [int]$suite.errors -eq 0 -and [int]$suite.skipped -eq 0 -and [int]$suite.failures -eq [int]($run -eq 'planted-witness')) 'Invalid specimen test result'
}
$failure=Get-Content "$out/planted-witness.stdout" -Raw
Require ($failure -match 'assert True is False' -and $failure -match "finish") 'Expected concrete acceptance mismatch missing'
Write-Output "Verified discovery, both controls, 31 histories, the 63-test specimen suite and the planted witness in $out"
exit 0
