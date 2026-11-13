# Task 44 handover: the model seam is specified and approved; implementation starts with group 1

Written 2026-11-13, 14:21 Europe/Berlin (from the clock). Planning group for the spec `model-seam`:
discovery, requirements, prototype gate, design, tasks. No production code was written.

## What was done

- 2026-11-12: two Python steps (R94, R95), the model diagnosis on Slice B's pack-authoring task (R96 to
  R101, R104), and the finding that the product's model path fails on three vendors (R102). Details and the
  owner's decisions of that day are in [handover.md](handover.md).
- Spec `model-seam` ([requirements](specs/model-seam/requirements.md), [design](specs/model-seam/design.md),
  [tasks](specs/model-seam/tasks.md), [prototype gate](specs/model-seam/prototype-gate.md),
  [research log](specs/model-seam/research.md)): all three phases approved by the owner.
- Prototype gate assessed as `proceed`: P1 one library on six first cases (R105), P2 build for five targets
  (R106), P3 waiting on a stub (R106), P4 the core through a process boundary (R107).
- 2026-11-13: the owner asked for an indicator of the driving model's performance, a measure of inference
  being good enough, and a way to tell whether a DSPy-style algorithm is needed around a model. Decided by
  menu and written into the spec as requirements Amendment 2, Requirement 10, the design component
  Measure, and tasks 4.3 and 5.3.

## The indicator, as decided

- Measured per task set and its check, over trials: the share of first replies that pass, the share that
  pass within two repairs, and calls, output tokens and time per accepted answer.
- Verdict: "good enough direct" at three quarters of first replies; else "good enough with the repair
  loop" at nine tenths within the repairs; else "not good enough as is", which says to try an optimiser
  around the instruction and measure again with the same command.
- Twenty trials by default; a quick mode of five that answers "good enough so far", "not good enough" or
  "undecided". Trials an endpoint failed are not counted against the model; more than a fifth of them
  gives no verdict.
- The seam ships a small built-in task set (four small modules, the core's static checks). The verdict
  that counts for adopters comes with the next spec, which supplies the real task (a model proposes a
  simplification) and the step flow as the check.

## Evidence

- Research entries R94 to R107 under `docs/research/2026-11-12/`; index in `docs/research/README.md`.
- Prototypes: `docs/experiments/model-seam/` (`prototype`, `prototype-llm`, `prototype-wait`, runners,
  `p4/`). Core launcher prototype: remote branch `model-seam-gate-p4`, commit `37effda`.
- Spec audit: `tools\bin\kiro-cli.exe check --feature model-seam`: 73 criteria, 0 errors, 0 warnings.

## Current state

- `spec.json`: phase `tasks-generated`, all approvals true, `ready_for_implementation` true.
- Nothing of the seam exists in `model/` or in `native/` on `main`.
- Two of three allowed calls to the Anthropic endpoint are used; the third is for task 5.1.
- Endpoint files and their defaults: see the table in [handover.md](handover.md). No key is in the
  repository.

## Next steps

1. Group 1 of [tasks.md](specs/model-seam/tasks.md): the crate `model/`, tasks 1.1 to 1.4, in the
   foreground, one commit per task, starting from the prototypes. Groups 1 to 5 change nothing in
   `native/`.
2. Tasks that can fail and go back to the design: 5.2 (a real refusal waited out), 5.3 (the built-in task
   set must tell models apart), 7.3 (a receipt that states only what was sent).
3. After the seam: the spec in which a model proposes a simplification and the step flow judges it, using
   `measure` with its own task set; then the onboarding spec.
