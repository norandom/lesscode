# Task 52 handover: group 8 of the model seam, release, documentation and final validation

Written 2026-11-16, 18:14 Europe/Berlin (from the clock). Spec `model-seam`, tasks 8.1 to 8.4. The
spec's tasks are all done.

## What was done

- **8.1 The seam in the pipeline** (`05190db`, then `0c1686c`). `tools/check_native.sh` runs `model-fmt`,
  `model-clippy`, `model-test`, `model-doctest` and `model-budgets`. After the owner's decision of the
  day they run in the Linux job only (`LESSCODE_CHECK_SEAM=1`, set by the Dagger container), together
  with `separation-model-source`; `separation-suites` only asserts that no `lesscode-model` is present;
  the seam and the guide optimiser share the target directory `model/target`.
- **8.2 Release** (`214e11c`). `release.yml` builds and packs `lesscode-model-<target>` beside
  `lesscode-core-<target>`; `publish-release.ps1` expects ten archives. `lesscode-model --version`.
  `scripts/license-inventory.ps1` writes `LICENSES.md` and `THIRD_PARTY_NOTICES.txt` for a crate;
  `model/` has its inventory and `model/licenses/`.
- **8.3 Documentation** (`43c6b38`). `docs/model-endpoint.md`; `.env.sample`, `README.md`,
  `docs/README.md`. `install.sh` and `install.ps1` take the second binary with `LESSCODE_WITH_MODEL=1`.
- **8.4 Final validation.** `.kiro/specs/model-seam/validation-report.md` (GO),
  [R113](../docs/research/2026-11-16/02-model-seam-release-and-final-validation.md), design Amendment 11,
  `docs/experiments/model-seam/run-released.ps1` with its receipts under `released/`.
- Everything in this group ran in the main context.

## Added after the owner's choices by menu (18:20 to 18:50)

- The guide optimiser's steps run in the Linux job only; the Dagger container mounts the guide file its
  dry run reads. No other platform compiles the client library.
- `native/.config/nextest.toml` and `model/.config/nextest.toml`: a test that runs longer than five
  minutes is ended and named.
- Criterion 3.8 has its test (`model/tests/waiting.rs`, the stub counts open requests). `model`: 82 of 82.
- The whole pipeline script passed locally with the Linux-only block on (372 s, every step 0). The
  hosted result for these commits is not in this file; look at the latest run of
  `Native core correctness`.

- Hosted run 43922104983 on `d5d7d92`: success on six jobs. Linux through Dagger 8 min 14 s (it now
  also lints and dry-runs the guide optimiser; every step 0), ARM Linux 2 min 0 s, Windows 4 min 5 s,
  macOS 2 min 32 s, macOS Intel 7 min 46 s. Before, on `0c1686c`: 5 min 5 s, 5 min 49 s, 13 min 16 s,
  10 min 9 s, 20 min 7 s.

## Added 2026-11-17 after a second menu (design Amendment 12)

- `6b72837`: `regenerate run` refuses `--effort` and `--thinking-display`; the refine schedule steps no
  effort. `docs/model-endpoint.md` says so.
- `43b4410`: a failed call's reason is on the terminal once (`seam::Failure` carries the kind of an
  answered failure). The regenerator's budget ceiling is 19,591 tokens, 829 units, 2,280 in a function.
- `8ffb096`: a reply cut off after a repair is asked again with that repair and its reasons.
- `f027ad7`: a reply the model refused ends the repair loop (`check`, `measure`, the guide tool).
- Local: `native` 401 of 401, `model` 84 of 84 and its doctest, `tools/guide-optimizer` 3 of 3, lints
  and both budget checks clean. One test of `step_guard` failed once in a first full run and passed
  alone and in the second; its message was not kept.
- `scripts/license-inventory.ps1 -Check`: `native` 83 packages, `model` 294 packages, nothing changed.
- Amendments to R111 and R112 record these; the items "for the owner" below are closed by them.
- Hosted run 44052850297 on `f61118c` (these four changes): success on six jobs; Linux through Dagger
  5 min 38 s, ARM Linux 2 min 1 s, Windows 4 min 25 s, macOS 2 min 55 s, macOS Intel 8 min 39 s.
- Release, found while preparing it: 1.2.0 was prepared on 2026-11-11 (`db175e5`) and never tagged; the
  latest release is v1.1.1. The owner decided: tag `v1.2.0` at `db175e5` first (run 43087717445 was
  green there), then `v2.0.0` on main. Main is prepared as 2.0.0: `native/Cargo.toml`, the three lock
  files, `CITATION.cff`, `.zenodo.json`, the README badge, `docs/releases/2.0.0.md`. Both license
  checks say nothing changed. Tagging and the approval are the owner's; the steps are at the end of
  `docs/releases/2.0.0.md`.
- The next work, by the owner's choice, is the simplification spec.

## Evidence

- Pipeline run 43911624473 on `0c1686c`: success on six jobs. Linux ran the seam's steps (80 of 80) and
  the core's suite (399 of 399). Windows 13 min 16 s (19 min 5 s before the cuts), ARM Linux 5 min 49 s.
- Release dry run 43904108284 on the branch `model-seam-release-dry-run` (since deleted): both archives
  for the five targets.
- `regenerate run` through the dry run's Windows binaries: six of six first cases, exit 0, a candidate
  written, `"mode": "seam-1"`. One call on the Anthropic endpoint, on the owner's word.
- Clean directory with release builds: `check` usable, no file written; the message of the missing
  `lesscode-model` with exit 3.
- `kiro-cli check --feature model-seam`: 0 errors, 0 warnings.
- Local: `native` 400 of 400, `model` 80 of 80, `tools/guide-optimizer` 3 of 3; the whole pipeline
  script with the seam's steps on, 445 s, every step 0.

## Current state

- Everything is pushed with the commits of the section above; the documents went out with that code
  change, as the owner chose.
- Branches deleted on the owner's word: `model-seam-gate-p4` (`37effda`), `model-seam-gate-p2`
  (`99c394c`, local only), `model-seam-release-dry-run`.
- Not found: why the ARM Linux job ran past its 30 minutes on `5aac5a1` and `05190db`; no log, no
  evidence. It passes in about 6 minutes on `0c1686c`.
- Not shown: the publish stage with ten archives; the install scripts' path for the second binary; the
  archives of the four other targets executed.
- Local Dagger: four of five runs in the main context were killed with exit 137. WSL has 8 GB in the
  owner's `.wslconfig`, which was not touched. Use the hosted pipeline as evidence.
- Containers: `mvibe-proxy-proxy-1` is the owner's and runs. `dagger-engine-v0.21.11` is left running.
- For the owner, not acted on at the time (all three closed on 2026-11-17, see the section above):
  - the refine loop's schedule steps an effort that is not sent; `--effort` and `--thinking-display` are
    accepted and not sent (R112);
  - from R111: whether a refusal ends a trial of the guide tool; a reply cut off after a repair is asked
    again without the reasons.
- The owner has not commented on design Amendments 3, 5, 6, 7, 8, 10 and 11.
- Slice B (drift hardening) is unchanged: measured through R79, the next step is the owner's choice.

## Next steps

1. The release is the owner's: version in `native/Cargo.toml` and the files named in
   `docs/ci-validation.md`, the tag, the approval of the `release` environment. Run
   `scripts/license-inventory.ps1 -Crate native -Check` and `-Crate model -Check` before tagging.
2. After the release: run `install.sh` and `install.ps1` with `LESSCODE_WITH_MODEL=1` once.
3. The simplification spec, then the onboarding spec.
