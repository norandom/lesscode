# R56: Direction: an annotation-guided driver that is agnostic to code and language, tested by a prototype gate before any design

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: not run
- Status: planned
- Spec: [.kiro/specs/agnostic-annotation-driver/](../../../.kiro/specs/agnostic-annotation-driver/brief.md) ([prototype gate](../../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md)); related: [constraint-guided-compression Amendment 5](../../../.kiro/specs/constraint-guided-compression/requirements.md)
- Hypothesis ID and predecessor: H-AD1, following [R55](01-released-binary-adopter-dogfood.md)
- Type: preregistered (targets fixed in the gate before execution)

## Decision

The owner set two priorities on 2026-11-10: first an agnostic driver, so the
process applies horizontally across codebases and languages; second,
hardening against LLM inference drift. Both are to rest on an
annotation-guidance process for the model, later optimised with bounded and
verified `MIPROv2` (`GEPA` if needed). The model endpoint stays on Opus for now
and moves to Mistral later.

This permits one thing next: preparing and, on the owner's go, running the
prototype gate of the new `agnostic-annotation-driver` spec. It permits no
design approval, no production tasks and no change to released commands.
Nothing has been executed; there is no extraction, fidelity, behavioral,
compression or adoption result to report.

## Hypothesis and falsification

R55 showed that mutation works only where hand-written operators match the
specimen text. The change proposed here is to move the code-specific
knowledge out of the tool: the model supplies located annotations from a
fixed taxonomy and writes no code; a deterministic validator and generic
operators do the rest; the scope's real tests classify each mutant.

H-AD1 is rejected if, on scopes outside the specimens, the frozen guide does
not reach the gate's validity, yield and meaningfulness targets on at least
three of four languages including one language the guide was not developed
on, or if an existing mutation tool gives equal or better results at lower
cost on all scopes. The targets, arms, bounds and the development and
held-out split are in the gate and may only change by a dated amendment made
before the run concerned.

## Frozen inputs and method

Not yet frozen. The gate lists what must be recorded before the first model
call: admitted scopes with revision, license and dirty-tree hash; the split;
taxonomy, validator rules and node-kind tables; guide hash; model identity
and request parameters; tool versions for the comparison arm. Scope
candidates are named in the gate and are not admitted yet.

## Evidence and results

None. Supporting evidence for the problem statement is R55 (36 of 38 Rust
files and 40 of 40 Python files without mutants; Python analysis absent).

## Limits and interpretation

This entry records a direction and a plan. The mechanism is untested. Two
known risks are stated so they are not discovered late: generic operators may
still need more per-language knowledge than a node-kind table, and model
annotations may cluster on easy sites and miss the failure paths where earlier
experiments found the real gaps (R25, R26). The gate measures the first
through the table-size limit and the second through the kind-coverage and
meaningfulness rows; neither is assumed away.

## Tool/spec consequence and next gate

- New spec `agnostic-annotation-driver` at phase `initialized`, with brief and
  prototype gate.
- `constraint-guided-compression` Amendment 5 now points to that spec for the
  driver; `MIPROv2` or `GEPA` work stays there as the second priority and takes
  its baseline (run-to-run agreement) from this gate.
- Next gate: the owner's go to screen scopes and run the gate's checklist.

## COBESY check

Outcome first: a direction and a plan, nothing run. Replay: not applicable
yet; the gate states what a replay will need. Disproof: stated above and as
gate rows, including a stop outcome in favour of existing tools. A negative
result is an accepted outcome. Facts (R55 counts) are separate from the
proposal. No claim of feasibility is made.
