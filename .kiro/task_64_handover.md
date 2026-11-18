# Task 64 handover · released-driver fault checks on E5 and E6

Corrective 2.1 is complete for the selected faults. The released Rust binary ran four matched branch inversions per trial on baseline and candidate. All 16 executed at the selected line and were detected by failing specimen tests. All opening and closing clean controls passed. Baseline and candidate used identical test-tree hashes, collected tests and pack hashes within each trial. The result covers the selected changes only. [R138 and per-function results](../docs/research/2026-11-18/04-e5-e6-specimen-faults.md)

The first E5 attempt was interrupted before a final receipt. The revised harness distinguishes xdist’s witnessed fail-fast exit from an invalid run. The driver restored its journal and reran the complete E5 baseline selection with reuse disabled. The four final receipts have exit 0. Product code, releases and Kiro production approvals were outside this task.

## Replay and storage

The [run directory](D:/BuildArtifacts/lesscode/corrective-audit/task64) contains replay instructions, exact commands, catalog and binary hashes, the selected mutant diffs, test XML, line traces and restoration checks. Run `verify.py` there with the existing specimen interpreter to reconcile the four receipts. [Validated results](D:/BuildArtifacts/lesscode/corrective-audit/task64/validated-results.json)

At close, the catalog files occupied 8,560,637 bytes and the other evidence occupied approximately 3.2 MB. The four restored driver copies occupied 266,213,414 bytes. The execution policy rejected the scoped PowerShell cleanup request, so the copies remain in the four `*-work` directories. Their source hashes matched their originals and all worker journals were clear. The cleanup point was after receipt validation. No new build target was needed. The measured lesscode footprint went from 27.57 to 27.83 GiB and D: free space from 198.73 to 197.58 GiB. [Storage measurements](D:/BuildArtifacts/lesscode/corrective-audit/task64/storage-close.json)

## Next group

The remaining approved audit queue starts with corrective 1.2. Reconcile the E3 and E4 figures with their producers, actual specimen inputs and retained receipts. For new Rust capability, corrective 2.3 remains the separate bounded-solver prototype in the state-consolidation brief. Its specimen-runtime contract and ordinary spec gates are pending. Task 63’s measurement patch is available for owner review.
