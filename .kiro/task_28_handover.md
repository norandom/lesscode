# Task Group Handover: Agnostic Annotation Driver, Prototype Gate Attempt 1 (outcome: revise)

Written 2026-11-10, about 13:30 Europe/Berlin. Start the next session here.
Predecessor: [task 27](task_27_handover.md) (v1.0.1 release, signing, R55).

## 1. Where things stand in one paragraph

The owner's top priority is an agnostic driver: mutation (and later
analysis) that applies horizontally to any code and language, with no
hard-coded, specimen-bound runs. A prototype gate was run today. One engine
with a small data pack per language works on Rust, Python and Java and fails
on TypeScript, so the gate outcome is `revise`. A second attempt is proposed
and has not started. Nothing in `native/` was changed.

## 2. Read these, in this order

1. [Gate](specs/agnostic-annotation-driver/prototype-gate.md): status
   `revise`. Amendments 3 to 8 govern where they differ from the original
   text.
2. [R61](../docs/research/2026-11-10/07-annotation-driver-gate-result.md):
   the result, its limits and the proposal for attempt 2.
3. [Prototype README](../docs/experiments/annotation-driver/README.md).
4. Only if needed: R55 to R60 in `docs/research/2026-11-10/`.

## 3. Result of attempt 1

| Language | Role | D1: rules at every site, hand pack | A: model annotates sites | D2: model-written pack |
| --- | --- | --- | --- | --- |
| Rust | development | all rows met | all met, 2 runs | not run |
| Python | development | all rows met | all met, 2 runs | not run |
| TypeScript | held-out | kind coverage not met | validity (71%, 76%) and kind coverage not met | kind coverage not met |
| Java | held-out | meaningful 67% (target 70%) | all met, 2 runs | all met (78%) |

- The model wrote the TypeScript and Java packs from the Rust and Python
  ones; both were admitted by deterministic checks on the first reply and
  are equivalent to the hand-written packs.
- Recommendation so far: the D form (rules as data, model as offline pack
  author) as the base; annotation as an optional layer that ranks sites.
- Run-to-run agreement of annotated sites (baseline for the drift work):
  Rust 0.93, Python 0.78, TypeScript 0.95, Java 0.64.
- 21 model calls, 125,390 input and 56,242 output tokens; at most about
  6.10 USD at an assumed 15 and 75 USD per million tokens (actual price not
  looked up). Cap in the gate: 20 USD.

## 4. Decisions waiting for the owner

1. **Go or no-go for attempt 2**, and whether to accept the proposal in R61:
   type-preserving operators as pack data; an operator for `return`-based
   failure sites; analysis of the TypeScript annotation rejections and a
   second TypeScript scope; all eligible sites instead of a sample of 40
   where the scope is small; independent check of the survivor labels. It
   must be recorded as a dated gate amendment before any run.
2. **Survivor labels.** All were made by the agent that designed the
   mechanism. They decide the Java arm D verdicts.
3. Still open from task 27: release signing key setup
   (`scripts\setup-release-signing.ps1`), without which no new tag can be
   published; the Python `analyze` stub that reports `passed: true`; the
   pre-push hook's refactor-branch check that does nothing.

## 5. Owner rules given today (also in agent memory)

- Agnostic driver first, hardening against LLM inference drift second. Both
  rest on annotation guidance with bounded, verified `MIPROv2` (`GEPA` if
  needed). Opus endpoint now, Mistral later. One task at a time.
- "Hard-coded arms" are non-agnostic runs that do not apply horizontally.
  They are removed, not kept as controls. The hard-coded operators in
  `native/src/adapters/*/mutation.rs` must go once a design passes the gate.
- The agent decides strategy directly and records it. Only clearly
  specified implementation is delegated, to Opus at xhigh effort.
- **Do not manage the Podman machine**: no `podman machine stop/start`, no
  `wsl --shutdown`, no edits to `~\.wslconfig`. Only run containers. If the
  Docker pipe is down, ask the owner to start the machine from Podman
  Desktop. (`~\.wslconfig` was set to 8 GB and 8 processors today on
  request.)
- The owner finds the runs too slow. Keep batches short, reuse outcomes, and
  report results without waiting on long runs.

## 6. How to run things

Prototype: `docs/experiments/annotation-driver/`. From that directory, in
PowerShell:

```powershell
$env:UV_PROJECT_ENVIRONMENT = 'D:\BuildArtifacts\lesscode\annotation-driver\venv'
$scopes = 'D:\BuildArtifacts\lesscode\annotation-driver\scopes'
uv run --no-sync python run_controls.py --scopes-dir $scopes --out results\controls-receipt.json   # 76 of 76
uv run --no-sync python run_arm.py --scope NAME --scopes-dir $scopes --out-dir OUT --all-sites      # arm D
uv run --no-sync python annotate.py --scope NAME --scopes-dir $scopes --guide guide-v1.md --out-dir OUT   # model call
uv run --no-sync python run_arm.py --scope NAME --scopes-dir $scopes --out-dir OUT --annotations OUT\annotations.json
```

- Scope names are in `scopes.json`. The scopes are isolated clones under
  `D:\BuildArtifacts\lesscode\annotation-driver\scopes`; pristine read-only
  copies are in `...\scopes-ro` (use these for checks while a batch runs).
- `AD_TABLES_DIR` points the engine at another pack directory (the
  model-written packs are in `results/d2/packs/`). `AD_RESULT_CACHE=off`
  disables reuse of mutant outcomes (store: `...\annotation-driver\result-cache`).
- Model endpoint: `%USERPROFILE%\.lesscode\model.env` (Anthropic style,
  `claude-opus-5-5`). Every real call appends to a `ledger.jsonl` next to
  its output directory. Real calls cost money.
- Java tests run through `docker compose -f compose.yaml run --rm java-tests`
  with `SCOPES_DIR` set; each run starts a fresh Maven container and takes
  20 to 60 seconds. Rust tests for the `mmhmem` scope must use the restricted
  target list in `scopes.json` (other test targets do not compile at that
  revision).

## 7. Known problems and traps

- TypeScript: dropped statements and negated type guards often fail the type
  check (10 of 28 mutants); the scope has no adjacent checks to swap.
- No operator exists for failure signalled by a `return` value, which is
  most failure sites in Rust and Java.
- Java arm D sits at the threshold: two draws of 40 gave 67% and 78%.
- The endpoint refused one pack-authoring request (`stop_reason: refusal`,
  category `cyber`) until the guide stated the purpose. Treat a refusal as an
  outcome; do not reword to get around it.
- A stopped batch leaves the clone mutated and may leave a test container
  running: restore with `git checkout -- <file>` in the clone and remove the
  container. The harness only restores on a normal exit.
- TypeScript arm A run 2 has a receipt whose code hashes are newer than the
  code it ran (files were replaced mid-batch); noted in R61.
- A detached `podman machine start` process from this session may still be
  open. Leave it; the Docker pipe depends on it.
- About 200 untracked experiment files in the working tree are the owner's;
  do not add them. Stage files by name.

## 8. Repository state

- Branch `main`, pushed. Last research commit `0be2a5b` (gate result), plus
  the commit that adds this handover.
- `v1.0.1` is published and unsigned. No tag since.
- Spec `agnostic-annotation-driver`: phase `initialized`, prototype status
  `revise`; no requirements, design or tasks.
- Cross-agent checkpoint: mmhmem project `lesscode`.
