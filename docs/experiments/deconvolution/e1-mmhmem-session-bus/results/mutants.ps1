# E1a: cargo-mutants on src/services/session_bus.rs only, per arm. Usage: pwsh mutants.ps1 a|b
param([string]$Arm)
$R = $PSScriptRoot; $O = "D:\Source\lesscode\.runs\e1-mmhmem\original"
$filterA = 'binary(=mmhmem) & test(/^services::session_bus::tests::/) | binary(=handoff) | binary(=multiprocess) & (test(=abrupt_exit_recovers_checkpoint_not_unsaved_work) | test(=committed_checkpoint_survives_without_reading_its_reply))'
$filterB = "$filterA | binary(=e1_trace_oracle)"
$common = @('mutants', '-f', 'src/services/session_bus.rs', '--timeout', '120', '--test-tool', 'nextest', '-j', '4', '--no-shuffle')
if ($Arm -eq 'a') {
  py -I "$R\rec.py" s8-mutants-arm-a $O -- cargo @common -o "$R\mutants-arm-a" -- --lib --test handoff --test multiprocess -E $filterA
} else {
  py -I "$R\rec.py" s8-mutants-arm-b $O -- cargo @common -o "$R\mutants-arm-b" -- --lib --test handoff --test multiprocess --test e1_trace_oracle -E $filterB
}
