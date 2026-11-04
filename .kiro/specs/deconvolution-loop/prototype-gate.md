# Prototype gate: deconvolution-loop (retrospective assessment)

Written 2026-11-04. This is a retrospective assessment of evidence that
already exists; the experiments were preregistered in their own plans
(E1b, E1c-2, E3), not in this file. It follows the
[prototype-first](../../steering/prototype-first.md) sequence so that a
later design approval has a gate to point at.

## Frozen inputs used

- Facdrone frozen copy `.runs/e2-facdrone/source` at `d49f48bd…`.
- Scope: five evidence-publication modules, 1,274 lines, 660 statements,
  67 functions, 84 intra-set edges (E3 `measures-baseline.json`).
- Corpus: `golden-trace.json` (51 sequences), `golden-e1c.json` (2),
  `golden-e1c2.json` (11); 191 seed files with 24 applicable deselections.

## Observable behaviours and targets, as preregistered in E3

T1 zero divergence within 4 rounds; T2 statements ≤ 561, lines ≤ 1,083;
T3 edges ≤ 67, writes in shell, pure share ≥ 50%; T4 kill rate ≥ 0.70 and
M1 to M3 caught; T5 one realizable oracle-blind counterexample.

## Evidence

| Condition for `proceed` | Met | Where |
| --- | --- | --- |
| Real-code responsibility exercised end to end by the mechanism | Yes, one scope, one round | R25 T1 |
| Candidate meets its predeclared usefulness target after full accounting | Yes as measured; the reviewer's canonical-format estimate (about 572 statements) misses the statement bar | R25 T2, review step 2 |
| Frozen behavioural checks and adverse controls support the boundary | Yes: round 0 on the identical copy, qdiff controls, verifier controls | E3 README validation, R24 verifier |
| Manual interventions distinguished from exercised capability | Yes: regenerator and reviewer are agents; hashing, measures, arms are scripts; the host guard is a scratch script | R25 budget section |
| Remaining work bounded and supported | Partly: measures and reports are bounded; scope selection and other languages are not supported by evidence | this file |

## Decision

- `proceed`: the loop as a measured research instrument on one Python
  scope (requirements 1, 2, 3, 5, 6 for one round, 7, 8 as measurement,
  9 as finder, 10, 11, 12 on Windows).
- `revise`: the size claim until a canonical-format measure exists
  (req 4.2); the sensitivity target (req 8), missed at 0.643; the
  missing-test report (req 8.7) and test-infra detection (req 12.4),
  unbuilt.
- `not-run`: scope selection (out of scope after R21); multi-round
  convergence; any language other than Python for regeneration.

Design approval and production tasks wait on the owner's choice among the
options in R25 (second round, second scope, or prevention) and on the two
cheap items above being built and measured.

Amendment 2026-11-04 (owner: "proceed with the spec for the next steps"):
`tasks.md` holds bounded apparatus revisions and one preregistered
experiment (E4), which prototype-first permits before the gate. The
regenerator program and its turn rule (requirements 6.7, 6.8) are
prototype work whose evidence is E4; they are not a `proceed` claim. The
sensitivity bar of 0.70 is unchanged and is re-measured, not lowered.
Production (native) tasks remain ungenerated.
