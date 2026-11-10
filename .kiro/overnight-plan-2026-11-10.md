# Plan for an unattended session, night of 2026-11-10 to 2026-11-11

Written 2026-11-10, late evening (Europe/Berlin). State at writing: main at `7912e1c`, CI green on five
platforms through `70919c0`, 389 native tests passing locally, model spend 5.89 of 20 USD at list prices plus
the optimisation run that is in progress.

## Rules for the night

- Work only on what is approved: the driver task plan (`.kiro/specs/agnostic-annotation-driver/tasks.md`) and
  the Slice B protocols (R66 and R70 with their amendments). Anything that needs a decision goes to the list
  at the end and is not decided by the agent.
- One commit per completed task, pushed to main. After each task group: full local checks (format, lints,
  nextest, doctests, budgets), then wait for the CI run and read its result. A red CI run is fixed before any
  further feature work.
- A handover file after each task group (`.kiro/task_N_handover.md`), and an mmhmem checkpoint. The context
  cannot be cleared by the owner during the night, so each group starts by rereading its handover.
- Model calls: only through the two preregistered tools, each with its own stop. Night budget: at most
  8 USD in total, and never past the 20 USD project stop. A refusal is an outcome and is not reworded.
- Never print, log or commit a key. `.env` stays git-ignored.
- No release, no tag, no version bump, no deletion of the old adapters (task group 7), no change to the
  shipped packs' behaviour, no force push.
- Containers: none are needed for the planned work. If one is started it is removed when its batch ends. The
  Podman machine is not managed.
- Stage files by name. The about 200 untracked owner files stay untouched.
- Stop conditions: the same failure twice in a row with no new information; a test that can only pass by
  weakening it; a result that contradicts a preregistered claim in a way the protocol does not cover (it is
  recorded, and that line of work stops); budget reached.

## Order of work

### A. Slice B (first, because it is running and cheap)

1. Finish the guide optimisation on `claude-opus-5-5` (`results/slice-b/optimise1-opus`, in progress). Record
   the result as a new research entry: frontier (quality per language, stability, tokens), discarded
   proposals, the final judgement against H1 to H3 of R70, cost. If the acceptance rule passes, save the guide
   as `pack-guide-v4.md` and say so; do not regenerate or change any shipped pack.
2. Second model arm of R66 on `mistral-large-4` (price 0.70 and 2.10 USD per million tokens from Mistral's
   API pricing page): the baseline with the current controls, 5 runs per language, at most two repairs. The
   baseline runner speaks only the Anthropic style through `annotate.py`'s OpenAI branch; check that branch
   against Mistral with one call before the batch.
3. Guide optimisation on `mistral-large-4` with the same parameters as the Opus run, starting from guide
   version 3. Record both arms side by side.
4. Only if 1 to 3 leave budget and a question it can answer: a comparison run of `MIPROv2` through the full
   `dsrust` crate in a second research package. Otherwise it stays open.

### B. Driver task group 4: `analyze` and `mutate` through the driver

Tasks 4.1 to 4.3 as written. This replaces the Python refusal with a real result and changes existing CLI
tests on purpose; each changed expectation is listed in the handover. The old adapters stay in the tree
(group 7 removes them, not tonight).

### C. Driver task group 5: workspace and safety

Tasks 5.1 and 5.2: copies, refusal rules, restore journal, recovery after an interrupted run. Tests on toy
trees only.

### D. Driver task group 6: batch runner, outcome store, `deconvolute run`

Tasks 6.1 to 6.5, on toy projects that need no network. This is the largest group; it is started only if B
and C are complete and green in CI. It may end the night unfinished, at a task boundary.

### Not tonight

- Group 7 (deleting the six adapter files, rerunning the specimen experiments, documentation) and group 8
  (full batches per language, which need containers for Java and a working container network).
- Any release.

## What the owner should expect in the morning

- A short state report at the top of `.kiro/handover.md`, the handover files, and research entries for every
  model run.
- For each planned item: done with evidence, not done with the reason, or stopped with the stop condition.

## Open tasks (collected)

### Driver (`agnostic-annotation-driver`)

- Task groups 4 to 8 of the plan; groups 1 to 3 are done (R67).
- `mutants()` reparses the file for every site; the batch runner should parse once per file.
- A control that checks what an operator writes, not only that it exists (the `?` rewrite that only removes
  the operator passes today).
- A control for the `throw` operator in Java, which three of five model packs lacked (R69).
- Parity tests take about nine seconds in a debug build.

### Slice B (`constraint-guided-compression`)

- Optimisation on Opus: running. Mistral arm: baseline and optimisation, not started.
- `MIPROv2` comparison through the full `dsrust` crate: not started.
- A language whose grammar is not compiled in, as a real held-out test of transfer.
- The annotation layer (run-to-run agreement 0.60 to 0.94 at the gate): untouched.
- Drift over time: the same protocol repeated on another day.
- Whether the shipped packs should be regenerated from an adopted guide.

### Project

- CI does not build `tools/guide-optimizer/`.
- Local Dagger cannot reach the network from the Podman VM, so the Linux check cannot be reproduced locally.
- Installers are untested on Linux aarch64 and macOS; the Linux adopter check against 1.0.2 was not rerun.
- About 200 untracked experiment files that documents link to.
- No release contains the driver; `README.md` and the adoption guide still describe the old behaviour
  (task 7.4).
- The Mistral key was pasted into a chat session and is in that session's transcript.

## Decisions that are the owner's (not taken tonight)

1. `match` guards as conditions in the Rust pack: adopt, as the Python pairs were, or reject.
2. Adding the Java `throw` control and an "operator output" control: these encode intended behaviour.
3. Regenerating shipped packs from an optimised guide.
4. Raising or removing the 20 USD project stop.
5. Whether `tools/guide-optimizer/` gets its own CI job.
6. Rotating the Mistral key.
