# R133: The model-simplification spec answered another question than the owner's and is superseded; the day's useful parts are named

- Report date/time and timezone: 2026-11-17, 21:00, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-17, 20:40 to 21:00 (reading and tidying; no experiment, no model call)
- Status: completed
- Spec: [model-simplification](../../../.kiro/specs/model-simplification/requirements.md) (superseded); successor [state-consolidation, brief only](../../../.kiro/specs/state-consolidation/brief.md)
- Hypothesis ID and predecessor: closes [R114](01-model-simplification-measurement-attribution.md) to [R132](19-full-length-inference.md)
- Type: retrospective, written by another agent than the one that ran R114 to R132

## Decision

The owner supersedes the spec `model-simplification`. Nothing of today's work is deleted. The next work
starts from the brief `state-consolidation`, which states the owner's idea: find a hidden state machine
that is spread across files, consolidate it, and use an SMT solver as the check, without a claim of
proof.

## What went wrong

1. **The term was never defined.** "Scattered state" came into the spec from a handover line ("fewer
   branch conditions, less scattered state") and was read as "where a field is written": write sites,
   writers, their spread over files. The owner meant a state machine that nobody wrote down, whose
   states, guards and transitions sit in different files.
2. **The owner's meaning arrived late and was added as a footnote.** It entered at about 18:55, after
   fifteen entries, as one line under "Terms" that says the write-site measures cannot establish such a
   machine. Requirements 2, 3, 6 and 9 still define and score state simplicity by those measures. The
   approved requirements therefore contradict themselves.
3. **The spec scores a model's rewrite and finds nothing.** No requirement asks for the machine to be
   shown. Three of eleven requirements are about attribution and yield of trials.
4. **The solver was left out.** The owner's idea builds on the solver approach of
   `constraint-guided-compression`. The spec does not mention a solver. Statements that a result is "not
   a proof" were treated as reasons to defer, where the owner asks for a bounded check and no proof.
5. **The apparatus took the day.** The prototype gate has ten dated amendments about its own harness.
   In the entries read for this review, no model proposal was accepted or judged useful.
6. **The discovery trial used an unsuitable instrument.** R131 and R132 ran a local 8B model on CPU with
   18,313 input tokens; the runtime failed. Neither entry says anything about whether a machine can be
   found. The project's configured endpoints were not used.
7. **The documents are layered.** Findings were appended as amendments while superseded text stayed.
   A reader, or an agent restarting from these files, meets both versions as current.

## What is kept and worth using

- The working definition in [R130](17-hidden-state-machine-fitness.md): name the thing and an event,
  show two histories in which the same event has different allowed outcomes, and trace where the state
  is stored, where the guard is read and who makes the transition.
- The job machine written twice, scattered and as a table, with a checker over 31 histories. The
  write-site measure gave both versions the same numbers.
- The stale-worker fault of the mutation driver as a real specimen: before the fix `60f79f3`, the fix
  `aa9d7d7`.
- The storage rule in `AGENTS.md` and the fix `8013bf3` in the spike (a runner error is not a killed
  fault).

## Evidence and limits

- Read in full: the brief, the requirements, `spec.json`, `task-generation-readiness.md`, R129 to R132.
  Outlines only: the design and the prototype gate. Not read: R114 to R128; point 5 rests on R129's
  statement that the two proposals it names were unmeasurable.
- The 25 commits of the day changed 78 files under `.kiro/`, `docs/research/` and
  `docs/experiments/model-simplification/`. None changed `native/`, `model/`, `tools/` or `scripts/`.
  None was pushed when this was written.
- The Codex session files show the model changed from `gpt-6-astra` to `gpt-6.2-astra` at about 18:44,
  and six subagent sessions. Whether the model contributed to the contradictions was not examined.

## Tool/spec consequence and next gate

- `model-simplification`: phase `superseded`; each of its documents carries a note at the top.
- `.kiro/handover.md` and the first section of `.kiro/steering/roadmap.md` are rewritten.
- Next gate: the owner answers the four questions at the end of the new brief; then requirements.

## COBESY check

This entry judges documents, not code or a model. It names what it did not read. It does not say the
day's measurements were wrong; it says they measured something the owner did not ask for.
