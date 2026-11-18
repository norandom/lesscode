# Write the bound-check target for the reply-checking machine (R154).
# Usage: make-target.ps1 RUN_DIR Z3_EXE
# RUN_DIR holds scattered.exe, one-place.exe and planted.exe: the test binary
# of tools/guide-optimizer with harness.rs inserted, built from the tree as
# it was at 45aff76, with the one-place change, and with that change minus
# its refusal arm.
param([string]$Run, [string]$Z3)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\loop-model.ps1"

# adapter.rs at 45aff76, lines 198 to 205: only a complete reply is judged.
$adapter = Loop { param($r) "(= $r 0)" }
# The planted copy judges a refused reply.
$planted = Loop { param($r) "(or (= $r 0) (= $r 3))" }
# check.rs line 71 and measure.rs line 165: the text is judged, however the reply ended.
$plain = Loop { param($r) "(distinct $r 1)" }

$exe = { param($name) "$Run\$name.exe" }
$arms = @(
    (Arm 'scattered_adapter' (& $exe 'scattered') 'histories_through_the_adapter' $adapter),
    (Arm 'one_place_adapter' (& $exe 'one-place') 'histories_through_the_adapter' $adapter),
    (Arm 'planted_adapter' (& $exe 'planted') 'histories_through_the_adapter' $planted),
    (Arm 'scattered_plain' (& $exe 'scattered') 'histories_through_a_plain_check' $plain),
    (Arm 'one_place_plain' (& $exe 'one-place') 'histories_through_a_plain_check' $plain))
$pairs = @(, @('scattered_adapter', 'one_place_adapter')) + @(, @('scattered_adapter', 'planted_adapter')) + @(, @('scattered_plain', 'one_place_plain'))
$exes = 'scattered', 'one-place', 'planted' | ForEach-Object { & $exe $_ }
Target $Z3 $exes $arms $pairs | ConvertTo-Json -Depth 8 | Set-Content "$Run\target.json" -Encoding utf8NoBOM
"wrote $Run\target.json"
