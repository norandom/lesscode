# Overnight Python research port to Rust lesscode, 17–18 November 2026

Implement the behavior developed in the Python research in the Rust lesscode-core CLI. Tonight’s first deliverable is a runnable Rust prototype of the missing call-edge measurement from corrective task 2.2, with an E5/E6 comparison and a replayable patch. The broader work includes finding scattered state, consolidating it and checking behavior within stated bounds. See the [corrective checklist](corrective-tasks.md) and [state-consolidation brief](specs/state-consolidation/brief.md).

## Start from PowerShell in the lesscode repository

```powershell
$repo = git rev-parse --show-toplevel
if ($LASTEXITCODE -ne 0 -or (git rev-parse --show-prefix)) { throw 'Start at the repository root' }
$prompt = Get-Content -Raw -Encoding utf8 .kiro/overnight-plan-2026-11-17.md
$runDir = Join-Path 'D:\BuildArtifacts\lesscode\corrective-audit' (Get-Date -Format 'yyyyMMdd-HHmmss')
New-Item -ItemType Directory -Path $runDir -ErrorAction Stop | Out-Null
$OutputEncoding = [System.Text.UTF8Encoding]::new($false)
("Evidence directory for this invocation is $runDir. Execute the Session instructions below." + [Environment]::NewLine + $prompt) |
    codex.exe exec -C $repo --sandbox workspace-write --add-dir $runDir -c 'approval_policy="never"' -o "$runDir\morning-summary.md" -
if ($LASTEXITCODE -ne 0) { throw "Session exited with code $LASTEXITCODE. Inspect checkpoints in $runDir" }
```

## Session instructions

Begin here in one foreground session. After the storage preflight, create a detached prototype worktree at the recorded HEAD under the supplied run directory. Make the Rust changes there. Read the frozen specimens from the original checkout. Keep the patch and evidence for morning review. Production adoption, the disposition of the old modules, the Python specimen runtime contract for task 2.3, push and release remain owner decisions.

1. Record start time in Europe/Berlin, current HEAD, tracked changes, tool versions and resolved paths. Read AGENTS.md, the latest numbered handover, the research index, R133, R134, R75, the linked specs, prototype-first steering and docs/testing.md. Measure D: free space and lesscode-owned disposable caches, including attributable shared-target usage. Apply the 50 GiB free and 40 GiB cache gates before substantial jobs. Record uncertain ownership and proceed with static audit work if a heavy job cannot run. Honor configured developer cache locations. Put temporary files under the run directory.
2. **2.2 — port the missing measurement before the audit sweep.**
   - Recover `measure_scope_source` from `src/lesscode_baseline/state_boundary/density.py` at the parent of `4f0aed4`. Inspect its E5/E6 callers for the files and aggregation boundary. The Python code counts call occurrences whose bare name or final attribute matches a function name declared in the measured source. Repeated calls count. Preserve that definition.
   - Freeze the input identities and expected results from that historical function. On a copy of a real scope file, add one matching call inside an existing function and record the expected delta of one. Run the current Rust analyzer first and retain its output.
   - In the prototype worktree, reuse the Rust parser behind `lesscode-core deconvolute analyze`. Expose the count for Python source through that command, preserving existing fields and other language packs. Keep the measurement itself in Rust. Build with the configured compatible Cargo target and run the new binary on the same E5 and E6 baseline/candidate files and the planted control.
   - Compare the native and historical results per file, then compare totals with R42 and R45. Record the native function and write-site counts alongside them and explain any definition differences. Run the affected Rust tests with cargo-nextest. Retain the binary identity, CLI invocations, outputs and a replayable patch including new files. Lead the morning summary with that patch and the real-specimen results. If parity fails, preserve the smallest observed mismatch and partial patch. This slice establishes one measurement. State discovery, consolidation and bounded equivalence each need their own evidence.

### Supporting corrective checks, in remaining time

2. **1.5 — inventory literal results.** Enumerate every run_gate.py and *_evaluator.py under both named trees, including hidden .kiro. Trace each apparent constant to its consumer. Keep the file inventory and a table of file, line, claim, literal and computation it replaces. Distinguish fixture inputs from reported results. Record zero findings for a reviewed script where applicable.
3. **1.1 — rerun E6.** Use the absolute candidate path and the existing .runs/e2-facdrone/venv/Scripts/python.exe with docs/experiments/deconvolution/e6-report-envelope/run_e6.py. Preserve the source-import guard and apply the repository's pytest-xdist settings to child pytest invocations. Set PYTHONDONTWRITEBYTECODE and give each suite its own JUnit XML path under the run directory. The runner suppresses successful child output. One way to retain counts is a scratch copy of the runner with HERE explicitly bound to the original script directory, changing only output capture to print child stdout/stderr and pass a unique --junitxml per suite. Keep the diff, run the original command as well, and label which invocation produced each result. Count executed, passed, failed, skipped and errors separately. Record any collection failure or source mismatch beside the historical 46/46.
4. **2.1 — compare E5 and E6 controls.** Locate and identify the released executable, original packs, baseline/candidate roots and tests from the trial records. Save version, hashes, actual CLI help and exact commands. Run matched baseline/candidate pairs on disposable specimen copies using the released deconvolute run entry point. Confirm clean controls pass before interpreting a planted change. Retain each function, changed site, evidence that the change was exercised, verdict and raw output. Separate missed faults from invalid runs. If an input or released capability is missing, retain the diagnostic and continue.
5. **1.2, 1.3, 1.4, then 4.2 — complete independent audit rows while time permits.** For E3/E4 map every reported figure to its producer, specimen path and rerun result. For each adapter map every requirement ID to today's reachable implementation and evidence. For the six other specs in 1.4 trace each validation claim to its command, original input and present reproducibility. For 4.2 quote the affected .zenodo.json and docs/reports sentences with locations and proposed wording. These are proposed corrections for review. Defer the remaining corrective tasks, including all owner decisions.

Checkpoint after each item and before a long command. For every attempted check keep its command, cwd, input identity, stdout/stderr, exit code and elapsed time. Keep a running morning-summary draft with completed rows, partial rows, blockers and the next exact command. Resume a partial item from its evidence. If sandbox access, storage, dependencies or a decision blocks one item, document it and take the next independent item. Use existing binaries and environments. Preserve original evidence and the owner's untracked files.

Finish when the queue is exhausted or at 07:00 Europe/Berlin on 18 November, whichever comes first. Check the clock before each job and bound child runtimes by the remaining window. At close, report each task ID, what was actually run, evidence paths, retained bytes, free space, deviations and decisions for the owner. List any corrective checklist or spec edits that would follow from these findings. Return this summary as the final message for the launcher's morning-summary.md.
