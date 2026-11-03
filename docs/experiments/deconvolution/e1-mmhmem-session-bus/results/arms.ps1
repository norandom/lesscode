# E1a: run Arm A (seed tests) and Arm B (seed tests + trace replay) via the receipt recorder.
# Usage: pwsh arms.ps1 <label> [<copy_root>]
param([string]$Label, [string]$Root = "D:\Source\lesscode\.runs\e1-mmhmem\original")
$R = $PSScriptRoot
$filterA = 'binary(=mmhmem) & test(/^services::session_bus::tests::/) | binary(=handoff) | binary(=multiprocess) & (test(=abrupt_exit_recovers_checkpoint_not_unsaved_work) | test(=committed_checkpoint_survives_without_reading_its_reply))'
$filterB = "$filterA | binary(=e1_trace_oracle)"
$targets = @('--lib', '--test', 'handoff', '--test', 'multiprocess', '--test', 'e1_trace_oracle')
py -I "$R\rec.py" "$Label-arm-a" $Root -- cargo nextest run --locked --no-fail-fast @targets -E $filterA
py -I "$R\rec.py" "$Label-arm-b" $Root -- cargo nextest run --locked --no-fail-fast @targets -E $filterB
