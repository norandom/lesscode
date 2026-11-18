# Write the bound-check target for the rule "a reply that is not whole is
# never accepted" moved into the repair loop (R155).
# Usage: make-target-whole.ps1 RUN_DIR Z3_EXE
# RUN_DIR holds before.exe and after.exe: the test binary of
# tools/guide-optimizer with harness.rs inserted, built at 48010b5 and with
# the loop-level rule.
param([string]$Run, [string]$Z3)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\loop-model.ps1"

# Only a complete, admissible reply is accepted: the adapter before and
# after, and every caller after.
$whole = Loop { param($r) "(= $r 0)" }
# Before, a check that sees the text alone accepts whatever text passes.
$text_only = Loop { param($r) "(distinct $r 1)" }

$arms = @(
    (Arm 'before_adapter' "$Run\before.exe" 'histories_through_the_adapter' $whole),
    (Arm 'after_adapter' "$Run\after.exe" 'histories_through_the_adapter' $whole),
    (Arm 'before_plain' "$Run\before.exe" 'histories_through_a_plain_check' $text_only),
    (Arm 'after_plain' "$Run\after.exe" 'histories_through_a_plain_check' $whole))
$pairs = @(, @('before_adapter', 'after_adapter')) + @(, @('before_plain', 'after_plain'))
Target $Z3 @("$Run\before.exe", "$Run\after.exe") $arms $pairs | ConvertTo-Json -Depth 8 | Set-Content "$Run\target.json" -Encoding utf8NoBOM
"wrote $Run\target.json"
