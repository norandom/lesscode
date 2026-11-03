# E1a coverage: cargo-llvm-cov per arm (exact seed filter) plus the literal step-7 variants.
$R = $PSScriptRoot; $O = "D:\Source\lesscode\.runs\e1-mmhmem\original"
$filterA = 'binary(=mmhmem) & test(/^services::session_bus::tests::/) | binary(=handoff) | binary(=multiprocess) & (test(=abrupt_exit_recovers_checkpoint_not_unsaved_work) | test(=committed_checkpoint_survives_without_reading_its_reply))'
$filterB = "$filterA | binary(=e1_trace_oracle)"
$t = @('--lib', '--test', 'handoff', '--test', 'multiprocess', '--test', 'e1_trace_oracle')
New-Item -ItemType Directory -Force "$R\coverage" | Out-Null
py -I "$R\rec.py" s7-cov-arm-a $O -- cargo llvm-cov nextest --locked --no-fail-fast @t -E $filterA --json --summary-only --output-path "$R\coverage\arm-a-summary.json"
py -I "$R\rec.py" s7-cov-arm-b $O -- cargo llvm-cov nextest --locked --no-fail-fast @t -E $filterB --json --summary-only --output-path "$R\coverage\arm-b-summary.json"
# Literal step-7 commands: all lib tests (not only the seed filter), handoff, with/without harness.
py -I "$R\rec.py" s7-cov-literal-without-harness $O -- cargo llvm-cov --locked --lib --test handoff --json --summary-only --output-path "$R\coverage\literal-lib-handoff-summary.json"
py -I "$R\rec.py" s7-cov-literal-with-harness $O -- cargo llvm-cov --locked --lib --test handoff --test e1_trace_oracle --json --summary-only --output-path "$R\coverage\literal-lib-handoff-e1-summary.json"
