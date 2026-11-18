# Write the bound-check target for the reply-checking machine (R154).
# Usage: make-target.ps1 RUN_DIR Z3_EXE
# RUN_DIR holds scattered.exe, one-place.exe and planted.exe: the test binary
# of tools/guide-optimizer with harness.rs inserted, built from the tree as
# it is, with the one-place change, and with that change minus its refusal arm.
#
# A history is three reply codes: 0 complete and admissible, 1 complete and
# rejected, 2 cut off, 3 refused, 4 failed call. Two repairs are allowed. The
# modelled result is ten times the number of the last call, plus 0 accepted,
# 1 rejected, 2 failed.
param([string]$Run, [string]$Z3)
$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot
$sha = { param($p) (Get-FileHash $p -Algorithm SHA256).Hash.ToLower() }

# The loop of model/src/repair.rs lines 113 to 136, with the check's verdict
# given by $Pass: a failed call ends it, a pass is accepted, a refusal or the
# last call is rejected, anything else is asked again.
function Loop([scriptblock]$Pass) {
    $rest = '31'
    foreach ($i in 3, 2, 1) {
        $r = "r$i"
        $ends = if ($i -eq 3) { 'true' } else { "(= $r 3)" }
        $next = if ($i -eq 3) { '31' } else { $rest }
        $rest = "(ite (= $r 4) $($i * 10 + 2) (ite $(& $Pass $r) $($i * 10) (ite $ends $($i * 10 + 1) $next)))"
    }
    $rest
}
# adapter.rs lines 198 to 205: only a complete reply is judged.
$adapter = Loop { param($r) "(= $r 0)" }
# The planted copy judges a refused reply.
$planted = Loop { param($r) "(or (= $r 0) (= $r 3))" }
# check.rs line 71 and measure.rs line 165: the text is judged, however the reply ended.
$plain = Loop { param($r) "(distinct $r 1)" }

$arm = { param($label, $exe, $test, $expression)
    [ordered]@{ label = $label; expression = $expression
        command = @('pwsh', '-NoProfile', '-File', "$here\run.ps1", "$Run\$exe", $test, '{inputs}', '{output}') } }
$target = [ordered]@{
    solver  = [ordered]@{ path = $Z3; sha256 = (& $sha $Z3) }
    pins    = @('scattered.exe', 'one-place.exe', 'planted.exe' | ForEach-Object { [ordered]@{ path = "$Run\$_"; sha256 = (& $sha "$Run\$_") } })
    symbols = @('r1', 'r2', 'r3' | ForEach-Object { [ordered]@{ name = $_; min = 0; max = 4 } })
    sort    = 'Int'
    arms    = @(
        (& $arm 'scattered_adapter' 'scattered.exe' 'histories_through_the_adapter' $adapter),
        (& $arm 'one_place_adapter' 'one-place.exe' 'histories_through_the_adapter' $adapter),
        (& $arm 'planted_adapter' 'planted.exe' 'histories_through_the_adapter' $planted),
        (& $arm 'scattered_plain' 'scattered.exe' 'histories_through_a_plain_check' $plain),
        (& $arm 'one_place_plain' 'one-place.exe' 'histories_through_a_plain_check' $plain))
    compare = @(, @('scattered_adapter', 'one_place_adapter')) + @(, @('scattered_adapter', 'planted_adapter')) + @(, @('scattered_plain', 'one_place_plain'))
    case    = [ordered]@{ id = '{id}'; replies = @('{r1}', '{r2}', '{r3}') }
}
$target | ConvertTo-Json -Depth 8 | Set-Content "$Run\target.json" -Encoding utf8NoBOM
"wrote $Run\target.json"
