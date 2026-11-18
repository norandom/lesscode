# Task 74 handover · Audits 1.2, 1.3 and 1.4

The three audits of group 1 are done, each with its own commit and research entry. No model was called and no product code changed.

**1.3, adapter specs** (`ba340fb`). The TypeScript, Java and Rust adapter modules are wrappers of 44 to 62 lines that only tests reach. The driver covers static analysis and fault runs. Nothing covers bounded comparison or runtime observation. By syntax all three candidates are `marginal` (5.6%, 18.8% and 7.0% fewer tokens). The three specs are now `not-delivered` with a finding per requirement. [R148](../docs/research/2026-11-18/14-adapter-specs-audit.md) · [Evidence](D:/BuildArtifacts/lesscode/corrective-audit/task74/1.3/evidence-sha256.json)

**1.2, E3 and E4** (`e6c3f47`). Every figure has a script that takes the candidate's root. The acceptance runs gave the recorded counts (E3: 1,403, 39 and 92 passed, 64 sequences; E4: 108 passed, 50 sequences, held-out replay without a difference). Twelve measure files and both reviewer-probe outputs repeat exactly. The mutation runs, E3's solver step and the model turns were not rerun. R25 and R26 need no amendment. [R149](../docs/research/2026-11-18/15-e3-e4-figure-audit.md) · [Evidence](D:/BuildArtifacts/lesscode/corrective-audit/task74/1.2/evidence-sha256.json)

**1.4, six completed specs** (`6683f4c`). Budgets, admission, the E4 three-turn replay and R39's nine Rust probes run today and agree. The Python claims of four specs were supported when made and are retired with the code (`4f0aed4`). `dogfood-self-application` is `not-delivered` for requirements 5 and 7.4. The other five keep their phase and carry an `audit` block. [R150](../docs/research/2026-11-18/16-completed-specs-sweep.md) · [Evidence](D:/BuildArtifacts/lesscode/corrective-audit/task74/1.4/evidence-sha256.json)

## Two findings outside the plan

- **Three tests pass while skipping.** `e3_double_record_reproduces_frozen_lf_body_digest`, `e4_double_record_reproduces_frozen_lf_body_digest` and `frozen_traces_replay_without_divergence` return early because `src/lesscode_baseline/recording.py` is retired, and report PASS. Recorded as new corrective task 3.4.
- **A stale build in the shared Cargo target.** Test binaries compiled at 07:59 from the prototype copy `D:\BuildArtifacts\lesscode\corrective-audit\foreground-20261118\prototype\native` were taken as fresh for this repository; the 10:17 command-line binary came from that copy too. `cargo clean -p lesscode-core` removed 8.5 GiB and everything was repeated from the repository: 401 tests passed, R148's receipts are identical in bytes. Evidence produced on 18 November before 13:19 with this repository's test binaries may have used that build. The frozen selector binaries of tasks 70 to 73 were not examined. The first run's output is kept under `1.4\stale-build\` and `1.3\nextest-stale-build.log`.

## Decisions taken here that the owner may want to reverse

- The status of four specs was changed to `not-delivered` (three adapters, dogfood), following the wording of tasks 1.3 and 1.4. For the dogfood spec only requirements 5 and 7.4 fail; the note says so.
- Specs whose claims were real but whose Python is retired keep their phase. They are not treated as unsupported.

## Storage

D had 186.6 GiB free at entry and 190.83 GiB at close. Task 74 retains 3,111,600 bytes under `D:\BuildArtifacts\lesscode\corrective-audit\task74`; the formatter scratch copies were removed. `D:\BuildArtifacts\lesscode` holds 5.02 GiB. The shared Cargo target measures 46.23 GiB, of which 15.4 GiB is identifiably lesscode's (`guide-optimizer`, three `model-seam-prototype` directories, lesscode-named units in `debug\deps`); `.tauri`, `wasm32-wasip2` and `maturin` output belong to others. The identifiable lesscode total is about 20.4 GiB, under the 40 GiB limit. Counting the whole shared target gives 51.25 GiB, over it. Earlier handovers used a fixed figure for the shared target; it should be measured at the start of the port group, which builds.

## Next group

Per the owner's order: the ports into `lesscode-core`, starting with 2.2 scope measurement (patch at `D:/BuildArtifacts/lesscode/corrective-audit/foreground-20261118/2.2-prototype.patch`), pack-driven for all shipped languages, then the 2.3 Z3 check, state-machine discovery (R140) and the Java lowering (R146, R147), generic parts only. Before building: give every copy of the crate its own `CARGO_TARGET_DIR`, and decide whether the prototype copy under `foreground-20261118` can go once its patch is ported. Then 3.1 with 5.2, where task 3.4 and the unreachable adapter wrappers and `NativeSpecimenRunner::run` belong too. Local commits stay unpushed (`origin/main` at `a5df503`); 6.2 and 6.3 are open. Slice B drift hardening (MIPROv2/GEPA) is unchanged, measured through R79, and awaits the owner's choice. [Corrective tasks](corrective-tasks.md)
