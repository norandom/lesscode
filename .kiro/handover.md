# Handover: start the state-consolidation spec; model-simplification is superseded

Reset 2026-11-17, 21:05 Europe/Berlin (from the clock). This file is the start for the next session.
Earlier versions: this morning's is in Git (`a416598`); older ones are
[handover-archive-2026-11-17.md](handover-archive-2026-11-17.md),
[handover-archive-2026-11-12.md](handover-archive-2026-11-12.md) and
[handover-archive-2026-11-11.md](handover-archive-2026-11-11.md).

## Start here

1. Read [specs/state-consolidation/brief.md](specs/state-consolidation/brief.md). It is one page and
   states what the owner wants, in the owner's words.
2. Read [R133](../docs/research/2026-11-17/20-model-simplification-closed.md) for what went wrong
   today, so it is not repeated.
3. Ask the owner the four questions at the end of the brief. Then write requirements
   (`$kiro-spec-init`, `$kiro-spec-requirements`), in the foreground.

Do not start from anything under `specs/model-simplification/`. Those files are a record.

## What the owner wants

Find a hidden state machine that is spread across files (scattered state), then consolidate it into
one place. Build on the solver approach: an SMT solver checks the consolidated machine against the
scattered one within stated bounds.

**No proof is needed.** The owner knows that a solver result within bounds is not a proof of
equivalence and not a minimal solution, and has said so. State that once in a report. Never give it as
a reason to stop, to defer or to hedge a result.

## State

- Releases 1.2.0 and 2.0.0 are published, signed and checked (2026-11-17); 2.0.0 is the latest. Nothing
  about them is open. Details: [task_52_handover.md](task_52_handover.md).
- `origin/main` is at `a5df503`. Local main is ahead by the commits of today (25 before this reset),
  all documents, experiments and spec files; none changes `native/`, `model/`, `tools/` or `scripts/`.
  Nothing is pushed. Pushing is the owner's decision.
- The spec `model-seam` is complete. `model-simplification` is superseded (phase in its `spec.json`).
- Handovers 53 to 62 belong to `model-simplification`. They are a record and are not needed to continue.
- Retained evidence of today, outside the repository: `D:\BuildArtifacts\lesscode\fitness60-*`,
  `fitness61`, `fitness62`.

## Useful from today

- R130's definition of the machine (thing, event, two histories with different outcomes, where the
  state is stored, where the guard is read, who makes the move).
- The job machine written twice (scattered, and as one table) with a checker over 31 histories:
  `D:\BuildArtifacts\lesscode\fitness60-concept`.
- The stale-worker fault of the mutation driver as a real specimen: source before the fix `60f79f3`,
  the fix `aa9d7d7`.
- Earlier solver work to build on: R14 (automata learning on Lesscode's snapshot lifecycle), R24
  (solver-found inputs), the spec `constraint-guided-compression`, and the untracked experiment
  directories `docs/experiments/constraint-modeler/` and `docs/experiments/cegis/`.

## Rules for the next session

- Define a term with the owner before building on it. "Scattered state" was read as "where a field is
  written" for a whole day.
- When the owner corrects the goal, stop and rewrite the requirements. Do not add the correction as an
  amendment under requirements that contradict it.
- One statement per fact. Replace superseded text; history is in Git and in the research entries.
- Use the configured endpoints through `lesscode-model` for model calls. No small local models.
- Storage gate before a task group: at least 50 GiB free on D:, at most 40 GiB of lesscode build
  caches (`AGENTS.md`).
- Subagents only on the owner's word. PowerShell only; nothing is run in a WSL distribution. No Python
  tooling. Stage your own files by path. No documents-only pushes. Open items are offered as menus.

## Open, small, not part of the next spec

- `install.ps1` with `LESSCODE_WITH_MODEL=1` puts both archives in one directory; the second archive's
  license notices overwrite the first's.
- `step_guard::the_installed_hook_stops_a_push_without_a_matching_verdict` failed once on 2026-11-12
  and once on 2026-11-17 and passed when run again.
- The owner has not answered on design Amendments 3, 5, 6, 7, 8, 10, 11 and 12 of `model-seam`.
- Slice B is closed; drift of one model over time is the one thing named as unmeasured.
