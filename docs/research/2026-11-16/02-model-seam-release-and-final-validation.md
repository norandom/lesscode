# R113: The model seam is released in form and validated: both binaries are built for five targets, all six first cases complete through them, and the pipeline is green after its duplicate builds were cut

- Report date/time and timezone: 2026-11-16, 18:12, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-16, 16:00 to 18:09 (task 8.1: to 16:58; task 8.2 with the dry run: to 17:15; task 8.3 with the clean-directory try: 17:10 to 17:17; the first cases: 17:18 to 17:20 and 17:41; the trimmed pipeline: 17:35 to 18:09)
- Status: completed
- Spec: [model-seam](../../../.kiro/specs/model-seam/requirements.md), [tasks 8.1 to 8.4](../../../.kiro/specs/model-seam/tasks.md), [validation report](../../../.kiro/specs/model-seam/validation-report.md)
- Hypothesis ID and predecessor: migration step 5 of the design ("release both binaries; update the docs"); [R112](01-model-seam-core-on-the-seam.md), [R108](../2026-11-14/01-model-seam-first-cases-live.md)
- Type: release dry run, live runs of the first cases, pipeline runs; the pass conditions are the tasks' (written 2026-11-12)

## Decision

Group 8 is done and the feature is validated with the verdict GO. The release workflow produces
`lesscode-core-<target>` and `lesscode-model-<target>` for the five targets. `regenerate run` completes
on all six first cases through the two executables of the dry run's Windows archives. The pipeline is
green on all six jobs for `0c1686c`.

Two things did not go as the tasks say. The pipeline's ARM Linux job ran past its limit on two commits
and the cause was not found. And the seam's own steps run on Linux only, by the owner's decision of
today, where task 8.1 says "on the pipeline's platforms".

## Hypothesis and falsification

The second binary can be built, packed with its license notices and released beside the first for the
five targets, a reader can configure and check an endpoint from one page, and the released form does on
the six first cases what the built form did. Rejected when the dry run lacks an archive, when a case
that passed in R108 fails through the released binaries, or when the pipeline is not green with the
seam's steps in it.

## Frozen inputs and method

- Commits: `05190db` (8.1), `214e11c` (8.2), `43c6b38` (8.3), `0c1686c` (the trimmed pipeline script).
- Dry run: `release.yml` started by hand on branch `model-seam-release-dry-run` at `214e11c`, run
  43904108284. A manual run builds and packs; it signs and publishes nothing.
- First cases: `docs/experiments/model-seam/run-released.ps1` with `lesscode-core 1.2.0` and
  `lesscode-model 0.1.0` unpacked from the dry run's Windows archives, side by side, no
  `LESSCODE_MODEL_BIN`. The turn of R102, one `regenerate run` per case, one case after the other.
  Endpoint files outside the repository, the repository's `.env` for the local endpoint (container
  `mvibe-proxy-proxy-1`, started by the owner today). The Anthropic case: one call, on the owner's word.
- Clean directory: release builds of both crates in a new directory, the endpoint file made from
  `.env.sample`, the steps of `docs/model-endpoint.md` in order against the local endpoint.
- Pipeline: the hosted workflow `Native core correctness`. Local Dagger runs are not evidence here: four
  of the five made in the main context were killed with exit 137 (WSL has 8 GB on this machine).
- Everything in this group was done in the main context.

## Evidence and results

**Release (8.2).** The dry run succeeded. Ten archives: `lesscode-core` and `lesscode-model` for
`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`,
`x86_64-apple-darwin` and `aarch64-apple-darwin`. Each holds the binary, `LICENSE`, `LICENSES.md` and
`THIRD_PARTY_NOTICES.txt` of its crate. `lesscode-model` answers `--version`.

The seam had no license inventory. `scripts/license-inventory.ps1` writes it from locked Cargo metadata.
Run on the core at the commit before `ureq` left, it reproduces all 83 sections of the committed notices;
on the core today it reproduces both files byte for byte. The seam's inventory lists 294 packages. Five
publish no license file (`jni`, `jni-macros`, `jni-sys-macros`, `r-efi`,
`rustls-platform-verifier-android`); their texts are kept under `model/licenses/` from the crates'
recorded source commits. None of the five is in the dependency tree of a released target (checked for
Windows, Linux x86_64 and macOS Apple Silicon). Every license expression has a permissive option.

**First cases through the released form (8.4).** Receipts and candidates:
`docs/experiments/model-seam/released/`.

| Case | Exit | Seconds | Tokens in, out | Candidate written |
| --- | --- | --- | --- | --- |
| 1 GLM 5.3, Mistral's API | 0 | 1 | 645, 226 | yes |
| 2 Mistral Large, Mistral's API | 0 | 73 | 650, 1,427 | yes |
| 3 `gpt-6.1-sol`, OpenAI's API | 0 | 16 | 641, 507 | yes |
| 4 `claude-opus-5-5`, Anthropic's API | 0 | 6 | 887, 525 | yes |
| 5 GLM 5.3, local endpoint | 0 | 1 | 645, 175 | yes |
| 6 Mistral Large, local endpoint | 0 | 27 | 650, 1,022 | yes |

- Six of six complete. Every receipt's turn has `"mode": "seam-1"` and, under `sent`, the host, the
  model and the output limit; five have a reasoning level, the Anthropic case none; none has a
  temperature. No kept file holds a key.
- No call waited for a rate limit. Case 2 showed one notice, "still waiting for the answer of
  api.mistral.ai: 60 s so far".
- This is the first live run of `regenerate run` through the launcher of task 7.1.

**Documentation (8.3).** `docs/model-endpoint.md`; `.env.sample`, `README.md` and the docs index point
to it. In the clean directory: `check` found the endpoint usable with `--endpoint-env` and with
`LESSCODE_MODEL_ENV` (one call each, 7.6 s and 29.5 s), wrote no file, and named a missing endpoint
file with exit code 2. `measure --quick`: first reply passed 4 of 5, passed within two repairs 5 of 5,
1.2 calls and 433 output tokens per accepted answer, verdict "good enough so far" (task set: built-in;
check: the core's parsing and static checks; model `zai-glm-5-3` on the local endpoint). With only the
core in a directory, `regenerate run` ended with exit code 3 and the message that names where
`lesscode-model` was looked for.

**Pipeline (8.1).**

| Run | Commit | Result |
| --- | --- | --- |
| 43902113944 | `5aac5a1` (group 7) | cancelled: ARM Linux ran past 30 minutes; the other five jobs passed |
| 43903666288 | `05190db` (8.1) | cancelled for the same reason; the other five jobs passed |
| 43911624473 | `0c1686c` | success, six jobs |

In run 43903666288 the client library's dependency tree was compiled four times per platform. On
Windows: `separation-model-source` 2 min 43 s, `model-clippy` 1 min 37 s, `optimizer-clippy`
1 min 46 s, `optimizer-dry-run-gepa` 2 min 25 s, of a job of 19 min 5 s. The owner judged that build
time not justified and chose three cuts, all in `0c1686c`:

1. `separation-suites` asserts that no `lesscode-model` is present before the core's suite and runs no
   test itself (it had run 52 tests a second time).
2. The seam and the guide optimiser share one target directory.
3. The seam's own steps run in the Linux job only.

| Job | `05190db` | `0c1686c` |
| --- | --- | --- |
| Linux through Dagger | 6 min 25 s | 5 min 5 s |
| Linux ARM | over 30 min, cancelled | 5 min 49 s |
| Windows | 19 min 5 s | 13 min 16 s |
| macOS Apple Silicon | 11 min 37 s | 10 min 9 s |
| macOS Intel | 17 min 33 s | 20 min 7 s |

In run 43911624473 the Linux job ran the seam's steps (80 of 80 tests) and the core's suite (399 of
399).

**Spec audit.** `kiro-cli check --feature model-seam`: 73 criteria, 0 errors, 0 warnings, 37 info.

## Limits and interpretation

- The cause of the ARM Linux overruns is not known. Neither cancelled job left a log or evidence. The
  job took about 6 minutes after task 6, when it already built the seam for the optimiser, and takes
  about 6 minutes now. That it passes after the cuts does not show that build time was the cause: the
  cuts also removed a second run of 52 tests and a separate build of the seam's tests on that platform.
- macOS Intel did not get faster. It still compiles the client library twice for the guide optimiser
  (lints, then the dry run). One run per commit says little about a hosted runner's speed.
- The seam's suite is no longer run on Windows, macOS and ARM Linux in the pipeline. It ran and passed
  on all of them once, in run 43903666288 (ARM excepted).
- Only the Windows archives were executed. The other eight were built and listed.
- The publish stage (signing, checksums, attestation of ten archives) runs only for a tag.
- The install scripts take `lesscode-model` with `LESSCODE_WITH_MODEL=1`. Both scripts pass a syntax
  check and were not run: no release carries the second archive yet.
- Criterion 3.8 (no second call to shorten a wait) holds by the construction of the wait loop; no test
  counts the open requests, as the design had promised.
- One turn per case. Nothing here measures a model.

## Tool/spec consequence and next gate

- The feature is complete by its tasks. The validation report has every criterion with its evidence.
- The gate branches are deleted on the owner's word: `model-seam-gate-p4` (`37effda`) and
  `model-seam-gate-p2` (`99c394c`), each one prototype commit that is not on main.
- For the owner, not acted on: a time limit per test in nextest would name a hanging test if the ARM
  overrun returns; the guide optimiser's steps could run on one platform, which removes the remaining
  double build; the refine loop's schedule still steps an effort that is not sent (R112).
- Next: the release itself is the owner's (version, tag, approval of the `release` environment). Then
  the simplification spec and the onboarding spec. Slice B (drift hardening) is unchanged: measured
  through R79, the next step is the owner's choice.

## COBESY check

The title says "released in form": a dry run, no release. The ARM overrun is reported as unexplained,
and the passing run is not read as proof of its cause. The pipeline table shows the job that got
slower beside those that got faster.
