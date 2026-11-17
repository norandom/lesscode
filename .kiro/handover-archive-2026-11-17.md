# Handover: the model seam is specified and approved; implementation starts with group 1

Reset 2026-11-12, 20:54 Europe/Berlin (from the clock), at the end of the day's session. This file is the
start for the next session. The file as it stood this morning, with the onboarding decisions in full, is
[handover-archive-2026-11-12.md](handover-archive-2026-11-12.md); the one before is
[handover-archive-2026-11-11.md](handover-archive-2026-11-11.md). Nothing in them was changed.

## Start here

1. Read the spec `model-seam`: [requirements](specs/model-seam/requirements.md) (approved, with Amendment 1),
   [design](specs/model-seam/design.md) (approved after the prototype gate),
   [tasks](specs/model-seam/tasks.md) (generated, **not approved**),
   [prototype gate](specs/model-seam/prototype-gate.md) (assessed as proceed).
2. Then the research entries of the day, newest first:
   [R107](../docs/research/2026-11-12/14-model-seam-gate-p4.md),
   [R106](../docs/research/2026-11-12/13-model-seam-gate-p2-p3.md),
   [R105](../docs/research/2026-11-12/12-model-seam-gate-p1.md),
   [R103](../docs/research/2026-11-12/10-model-seam-library-prototype.md),
   [R102](../docs/research/2026-11-12/09-product-path-two-models.md).
3. Update of 2026-11-13: the owner approved the tasks and asked for an indicator of whether a model is
   good enough, direct or with the repair loop (Requirement 10, tasks 4.3 and 5.3). The planning group is
   closed in [task_44_handover.md](task_44_handover.md); read it, then start group 1 (the crate `model/`)
   in the foreground, one commit per task.

## State

- `main` is at `85d63ae`, pushed. CI is green for it (and for `79aee0b`, `ea10cd7`, `64b76b2`).
- Release 1.2.0 is still prepared and **not tagged**. The owner holds the tag until simplicity is shown on
  a real step ("we are not ready"). The code on `main` has changed since "prepare 1.2.0": hook message,
  endpoint reasoning level, the endpoint's message in a failed call.
- Remote branch `model-seam-gate-p4` (`37effda`) holds the core launcher prototype of gate P4. Task 7.1
  starts from it; task 8.4 deletes it. `model-seam-gate-p2` is deleted.
- README opens with the picture `docs/images/before-after-lesscode.png` (owner request). The character in
  it is recognisably from The Simpsons; the owner was told of the copyright risk.
- Model spend on the Anthropic endpoint: about 33.0 of 40 USD (two small calls today). Two of the three
  calls the owner allowed for the seam's first cases are used; the third is reserved for task 5.1.
- No container of this project is running. No background job is running.

## What the day decided (owner)

- **Dogfooding first.** Two real steps in a Python project (R94, R95); findings in those entries.
- **Effectiveness is simplicity**, not lines or tokens: fewer branch conditions, less scattered state.
  Nothing is tuned for one target; a fix belongs in the language pack's data.
- **Language packs are published once** with the tool. They are the knowledge base and are not
  regenerated per repository or with a cheaper model. Pack authoring is closed (R104).
- **Adopters must be able to run GLM or another model** behind the tool. The core and the driver are model
  agnostic; the fence around a model's output stays because inference is not deterministic.
- **Use an existing model library**, not a hand-written client. Chosen after the gate: `genai` 0.7.4.
- **Horizontal compatibility with an estimated wait time**: every endpoint on the same footing, the local
  one included; when a call has to wait, the tool waits and says for how long it expects to.
- The affordable model is `zai-glm-5-3` at reasoning level "high"; prices to calculate with: 1.19 EUR per
  million input tokens, 0.119 cached, 3.74 output. For OpenAI: `gpt-6.1-sol` at "medium".
  `mistral-large-latest` is Mistral Large 4.0 (`mistral-large-4` is an alias).
- Wait limits (requirements Amendment 1): ten minutes for refused attempts and pauses; sixty minutes for an
  accepted call to answer; at most two repairs; the check command makes at most three calls.
- No Python tooling in this project. Endpoints, models and keys come from endpoint files, never from code.

## The model seam in five lines

- A second binary `lesscode-model` (new crate `model/`) is the only place that talks to a model service.
- The core starts it for each model request: one JSON request in, one JSON reply out, notices on the
  terminal. The core links nothing of the library and loses its hand-written request and `ureq`.
- On top of the library we build: waiting with an estimate, the check command, the repair loop (propose,
  judge, repair in a fresh call with the reasons), the record of calls.
- Gate result: one library completes the turn on all six first cases; it builds for the five release
  targets; the waiting rule passes on a stub; `regenerate run` completes through the process boundary.
- Carried into implementation: one real refusal waited out (task 5.2), and a receipt that states only what
  was sent (task 7.3).

## Endpoints (files outside the repository; never copy a key anywhere)

| File | Endpoint | Model by default |
| --- | --- | --- |
| `D:\Source\lesscode\.env` (git-ignored) | the owner's local endpoint | `zai-glm-5-3`, level "high" |
| `%USERPROFILE%\.lesscode\mistral.env` | Mistral's API; also serves `zai-glm-5-3`, much faster than the local endpoint | `mistral-large-latest`, level "high" |
| `%USERPROFILE%\.lesscode\openai.env` | OpenAI's API, **temporary key** | `gpt-6.1-sol`, level "medium" |
| `%USERPROFILE%\.lesscode\model.env` | Anthropic's API, under the 40 USD stop | `claude-opus-5-5` |

The model and the level can be set per run with `LESSCODE_MODEL_NAME` and
`LESSCODE_MODEL_REASONING_EFFORT`. The Mistral and OpenAI keys were pasted into the chat on 2026-11-12 and
are in that transcript; the owner should revoke the OpenAI key after task 5.1 and rotate the Mistral key
if the transcript is shared.

## For the owner

1. Done on 2026-11-13: the tasks of `model-seam` are approved.
2. The tag `v1.2.0` stays yours to set; see `docs/releases/1.2.0.md`.
3. Making the `Step guard` job required is a repository setting.

## Order of work after the seam

1. A spec in which a model proposes a simplification where branch conditions and state concentrate, the
   step flow judges it, and the verdict reports simplicity. Generic through the language packs: fix the
   Python pack's `state_write` rule (it counts local assignments), let `step plan` rank by branches and
   state. Real steps on `x_likes_scraper` (`exporter.py` for state, `tools.py` for branching). The tag
   follows a step that shows simplicity.
2. The onboarding spec (`/lesscode start`, levels 1 to 5, shipped as a plugin). Its decisions are in the
   archive of this morning: levels and wording, hooks (pre-push at level 3, pipeline at level 4, no
   pre-commit), disk and cleanup requirements, `step check` refusing a change to an unprotected function,
   the working name "nobreak gate". Levels 3 and up will use the seam's check command.

## Open, without a spec

- 21 test files besides `tests/regenerator.rs` use the temp folder and were not checked for directories
  they leave behind.
- One full test run on 2026-11-12 had a single failure in
  `step_guard::the_installed_hook_stops_a_push_without_a_matching_verdict` that did not repeat.
- An out-of-reach mark is still not shown on a real step.
- Per-test results in the evidence file for openidea-kiro stay deferred.
- What model-written packs that differ from the shipped pack do to a gate (R100) matters only if a fifth
  language is added by a model.
- The burst of HTTP 400 answers on `mistral-large-latest` through the local endpoint (R98) is unexplained.

## Where things are

- Prototypes of the gate: `docs/experiments/model-seam/` (`prototype`, `prototype-llm`, `prototype-wait`,
  `run-first-cases.ps1`, `run-p4.ps1`, `p4/`). Their build directories are under
  `D:\BuildArtifacts\cargo-target\model-seam-prototype*`.
- Results of the model runs of the day: `docs/experiments/model-diagnosis/results/`.
- Python steps: clone `D:\BuildArtifacts\lesscode\steps-foreign\x_likes_scraper`, branches
  `lesscode-escape-html-1` and `lesscode-checkpoint-clear-1`. The owner's repository is only read.
- Spec audit: `tools\bin\kiro-cli.exe check --feature model-seam`.

## Rules that caught someone out today

- Read the earlier research entries on a topic before proposing a change. The node kind list was proposed
  as new although R78 had tested it the day before; R97 carries the amendment.
- No Python: a Python prototype was patched and a Python probe written before the owner stopped it.
- Times in documents come from the clock or the commit, not from an estimate. Several entries had to be
  corrected.
- Do not run two jobs against one upstream at once. A run of ours caused the rate limit that a test then
  met (R102).
- A branch name is `lesscode-<step>-<run number>` for a step.
- PowerShell commands that remove something or carry nested quotes go into a `.ps1` file; the command
  guard misreads them otherwise.
- `cargo build --examples` does not rebuild the executable; build with `--bins` and copy the executable
  before a long run.
- Before pushing README or guide text, run it through the humanizer skill (owner, 2026-11-11).
