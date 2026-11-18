# Task 63 handover · foreground recovery of the overnight prototype

Completed corrective 2.2, 1.1 and 1.5 on 18 November 2026. The frozen Windows executable and patch are ready for review. The Rust changes live in a detached prototype worktree based on `b08adf1`. Main contains the approved plan, results and checklist updates. Production adoption remains an owner decision.

| Item | Evidence |
| --- | --- |
| 2.2 | Both Rust measurement definitions match their Python references on E5 and E6. The scoped function, edge and write totals reproduce R42 and R45. Both planted source changes produced their predicted deltas. The pinned Windows suite passed 402/402 and the Linux Dagger native suite passed 401/401. The exported pipeline status was 0. [R135](../docs/research/2026-11-18/01-rust-scope-measurements.md) |
| 1.1 | The guarded E6 acceptance run exited 0. JUnit counts were 56 passed and one operator-database skip. [R136](../docs/research/2026-11-18/02-e6-acceptance-rerun.md) |
| 1.5 | The full inventory returned seven scripts. The report lists literal results and their consumers. [R137](../docs/research/2026-11-18/03-gate-result-literals.md) |

The next task group is corrective 2.1. Use the released 2.0.0 mutation driver on the E5/E6 baseline and candidate pairs, with matching packs and tests, clean controls and preserved specimen copies. Establish the denominator and which function each planted change exercised. The two source changes in R135 tested measurement only. Task 2.3 still needs the specimen runtime decision and its own bounded solver prototype.

## Local replay and storage

The [foreground run directory](D:/BuildArtifacts/lesscode/corrective-audit/foreground-20261118) contains `2.2-prototype.patch`, `lesscode-core.exe`, the detached `prototype/`, and `replay.py`. The patch applies to the original native source with `git apply --check`. Its SHA-256 is `276e9857a6d415e7aa332e30419f46efecfdf3860832d7ca6bf76ee3cb119cb4`. R135 gives the replay command. The earlier overnight summary remains in its own directory.

Reused the shared Cargo target. The measured lesscode footprint, including evidence and identifiable files in the shared target, went from 26.53 to 27.57 GiB. D: free space went from 203.21 to 198.74 GiB. No isolated Cargo target was created. The run directory held 165,524,408 bytes at close, including 116,293,231 bytes of detached review source, 21,920,821 bytes of binary, evidence and replay inputs, and 27,310,356 bytes in the pinned nextest download directory. A workstation policy rejected cleanup of the latter after the run. It may be reused for pinned Windows verification. Retain the patch and evidence through adoption review.
