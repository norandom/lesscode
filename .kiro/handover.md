# Handover: releases 1.2.0 and 2.0.0 are published; the model seam is done; next is the simplification spec

Reset 2026-11-17, 11:20 Europe/Berlin (from the clock). This file is the start for the next session. The
file as it stood before, written on 2026-11-12 for the start of the model seam, is
[handover-archive-2026-11-17.md](handover-archive-2026-11-17.md); the ones before are
[handover-archive-2026-11-12.md](handover-archive-2026-11-12.md) and
[handover-archive-2026-11-11.md](handover-archive-2026-11-11.md). Nothing in them was changed.

## Start here

1. Read [task_52_handover.md](task_52_handover.md): the last task group of `model-seam`, with everything
   added on 2026-11-17 in its dated sections.
2. Update 2026-11-17: discovery is recorded and `model-simplification` is initialized. Read its
   [brief](specs/model-simplification/brief.md) and [analysis](specs/model-simplification/research.md).
   The owner selected simplicity reporting separate from the behavior verdict. Next:
   `$kiro-spec-requirements model-simplification`, in the foreground. No requirements, design,
   prototype or implementation tasks are approved yet.
3. Research index: [docs/research/README.md](../docs/research/README.md). Latest entry R114 is static
   analysis of measurement and attribution; no experiment ran. Prior entry R113, with
   amendments of 2026-11-17 in R111 and R112.

## State

- `origin/main` is at `a5df503`. This reset is committed locally and not pushed: it is a documents-only
  change and goes out with the next code change (owner rule of 2026-11-16).
- **Release 1.2.0** is published (tag `v1.2.0` at `1c2af6b`, 2026-11-17). Checked from a fresh download:
  the signature of `SHA256SUMS`, all 12 files against it, and for each of the five archives the embedded
  signature, the `.sha256` file and the build attestation. Zenodo: `10.5281/zenodo.24304919`.
- **Release 2.0.0** is published (tag `v2.0.0` at `5f2363b`, 2026-11-17) and is the latest. The same
  checks pass for its 22 files and ten archives. Zenodo: `10.5281/zenodo.24305214`.
- The install scripts of 2.0.0 were run once each with `LESSCODE_WITH_MODEL=1`: `install.ps1` on this
  machine into a directory under `D:\BuildArtifacts`, `install.sh` in an `ubuntu:24.04` container. Both
  installed `lesscode-core 2.0.0` and `lesscode-model 0.1.0`, and both binaries answer `--version`.
  The macOS path of `install.sh` was not run.
- The spec `model-seam` is complete: validation GO, design Amendments 1 to 12.
- Pipeline: the last run on main (`5f2363b`, run 44055400314) is green on six jobs. The run for
  `a5df503` was not looked at.
- No container of this project is running. `mvibe-proxy-proxy-1` is the owner's and runs.
  `dagger-engine-v0.21.11` is left as it was. No background job is running.

## Found today, not acted on

- **Windows install overwrites license notices.** With `LESSCODE_WITH_MODEL=1`, `install.ps1` unpacks both
  archives into one directory. `LICENSES.md` and `THIRD_PARTY_NOTICES.txt` have the same names in both,
  so the second archive's files replace the first's. `install.sh` keeps them apart
  (`~/.local/share/lesscode-core` and `.../lesscode-model`).
- **The `step_guard` test has now failed twice.**
  `the_installed_hook_stops_a_push_without_a_matching_verdict` failed once in a full run on 2026-11-12
  and once on 2026-11-17, each time passing when run again. Six runs of the five slowest test binaries
  side by side did not reproduce it. The message was not kept either time.
- **ARM overruns of 2026-11-16.** In both jobs the check step never ended, the evidence upload that runs
  on every outcome did not run, and no log exists. That is how a runner that stopped responding looks.
  It fits memory exhausted by repeated builds of the client library, which no platform but Linux x86_64
  does any more. Not proven; it has not returned in five runs.
- `request_kwargs` and `hash_set` in `native/src/regenerator.rs` are called only by a test. A candidate
  for the simplification work.
- `CITATION.cff` in the 1.2.0 tag says `date-released: 2026-11-11`; the release is of 2026-11-17.

## For the owner

1. Design Amendments 3, 5, 6, 7, 8, 10, 11 and 12 of `model-seam` were summarised one line each in the
   chat on 2026-11-17 and have no answer yet. They are in
   [specs/model-seam/design.md](specs/model-seam/design.md).
2. The OpenAI key of 2026-11-12 was called temporary; whether it is revoked is not known here.
3. Making the `Step guard` job required is a repository setting.

## What was decided on 2026-11-16 and 2026-11-17 (owner)

- The build time of the pipeline was not justified: the client library is compiled in the Linux job
  only; the seam's and the guide optimiser's steps run there and nowhere else.
- A test that runs longer than five minutes is ended and named.
- No documents-only pushes; documents go out with the next code change.
- Open items are offered as menus.
- `regenerate run` loses `--effort` and `--thinking-display`; a failed call's reason is said once; a
  refusal ends the repair loop; a reply cut off after a repair is asked again with the repair's reasons.
- 1.2.0 is tagged at the last commit before the seam, then 2.0.0 on main. This replaces the decision of
  2026-11-12 to hold the tag until simplicity was shown on a real step.
- **Slice B is closed** (owner, 2026-11-11, `docs/reports/slice-b-drift-hardening.md`; reopened for cost
  on 2026-11-12 and closed again with R104). Reports of 2026-11-16 and 2026-11-17 that called its next
  step open were wrong. One thing is named as unmeasured: drift of one model over time.
  `lesscode-model measure` could repeat one task set on later days.

## Order of work

1. **The simplification spec.** A model proposes a simplification where branch conditions and state
   concentrate, the step flow judges it, and the verdict reports simplicity: fewer branch conditions,
   less scattered state, never lines or tokens removed. Generic through the language packs: fix the
   Python pack's `state_write` rule (it counts local assignments), let `step plan` rank by branches and
   state. Real steps on `x_likes_scraper` (`exporter.py` for state, `tools.py` for branching). The model
   is reached through `lesscode-model`.
2. **The onboarding spec** (`/lesscode start`, levels 1 to 5, shipped as a plugin). Its decisions are in
   [handover-archive-2026-11-12.md](handover-archive-2026-11-12.md). Levels 3 and up use
   `lesscode-model check`.

## Endpoints (files outside the repository; never copy a key anywhere)

| File | Endpoint | Model by default |
| --- | --- | --- |
| `D:\Source\lesscode\.env` (git-ignored) | the owner's local endpoint; needs the container `mvibe-proxy-proxy-1` | `zai-glm-5-3`, level "high" |
| `%USERPROFILE%\.lesscode\mistral.env` | Mistral's API; also serves `zai-glm-5-3` | `mistral-large-latest`, level "high" |
| `%USERPROFILE%\.lesscode\openai.env` | OpenAI's API, key called temporary | `gpt-6.1-sol`, level "medium" |
| `%USERPROFILE%\.lesscode\model.env` | Anthropic's API; every call needs the owner's word | `claude-opus-5-5` |

Guide for a user: [docs/model-endpoint.md](../docs/model-endpoint.md).

## Open, without a spec

- 21 test files besides `tests/regenerator.rs` use the temp folder and were not checked for directories
  they leave behind.
- An out-of-reach mark is still not shown on a real step.
- Per-test results in the evidence file for openidea-kiro stay deferred.
- What model-written packs that differ from the shipped pack do to a gate (R100) matters only if a fifth
  language is added by a model.
- The burst of HTTP 400 answers on `mistral-large-latest` through the local endpoint (R98) is unexplained.

## Where things are

- Release checks of today: `D:\BuildArtifacts\lesscode-release-check\` (`v1.2.0`, `v2.0.0`,
  `install-win`). Disposable.
- Release notes: `docs/releases/1.2.0.md`, `docs/releases/2.0.0.md`. Signing: `docs/release-signing.md`.
- The seam's live cases and receipts: `docs/experiments/model-seam/`.
- Spec audit: `tools\bin\kiro-cli.exe check --feature model-seam`.
- Pipeline script: `tools/check_native.sh`; the hosted workflow `Native core correctness` is the
  evidence. Local Dagger runs were killed with exit 137 (WSL has 8 GB; `.wslconfig` is the owner's).

## Rules that caught someone out

- A status carried in memory goes stale. Slice B was reported as open for two days after the roadmap
  said closed. Read `.kiro/steering/roadmap.md` before naming a status.
- Check that a release exists before writing "since release X". 1.2.0 was prepared and not tagged for
  six days, and a version question was asked on the wrong premise.
- Read the earlier research entries on a topic before proposing a change.
- No Python. PowerShell only; nothing is run in a WSL distribution.
- Times in documents come from the clock or the commit.
- Do not run two jobs against one upstream at once.
- PowerShell commands that remove something or carry nested quotes go into a `.ps1` file.
- Before pushing README or guide text, run it through the humanizer skill.
- Stage your own files by path; the working tree holds many untracked files of other work.
