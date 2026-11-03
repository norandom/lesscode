# E1a Arm B completion (owner decision 2026-11-03): only the now_ms mutants, -j 2, Arm B test set.
$R = $PSScriptRoot; $O = "D:\Source\lesscode\.runs\e1-mmhmem\original"
$filterA = 'binary(=mmhmem) & test(/^services::session_bus::tests::/) | binary(=handoff) | binary(=multiprocess) & (test(=abrupt_exit_recovers_checkpoint_not_unsaved_work) | test(=committed_checkpoint_survives_without_reading_its_reply))'
$filterB = "$filterA | binary(=e1_trace_oracle)"
py -I "$R\rec.py" s8-mutants-arm-b-nowms $O -- cargo mutants -f src/services/session_bus.rs --re now_ms --timeout 120 --test-tool nextest -j 2 --no-shuffle -o "$R\mutants-arm-b-nowms" -- --lib --test handoff --test multiprocess --test e1_trace_oracle -E $filterB
