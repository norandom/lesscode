# Brief: deconvolution-loop

Written 2026-11-04 at the owner's request, after R25, as documentation of
the loop that the deconvolution experiments prototyped. It precedes any
production architecture. Nothing here is approved for implementation.

## Problem

AI-generated codebases scatter one responsibility across many small
functions and files. Rewriting them smaller is easy; knowing that the
smaller version behaves the same is not. Tests alone miss stored
representation and error paths; static models of real code failed (H7,
R1 to R18); instructions to an agent without measurement produced bad
results on real code.

## Current State

A loop exists as research apparatus and ran end to end once on real code
(Facdrone, five evidence-publication modules, R25): derive the public
surface, record behaviour under a trace oracle, regenerate behind the
surface, accept on zero divergence, measure edges and size, review
independently, score sensitivity by mutation, and search the pure core for
oracle-blind differences. One round met the oracle with 16% fewer lines and
29% fewer edges. Sensitivity missed its bar and one oracle-blind difference
is real. The apparatus is Python-specific and hand-assembled per scope.

## Desired Outcome

An algorithm-agnostic specification of the loop, with each step's
feasibility status tied to the research entry that established it, and a
design that documents how the prototype realises each step. A later
implementation (Rust is the owner's intention) copies what the adopted
prototype projects proved and leaves out what they did not.

## Approach

Specify the loop as decision rules over observable artefacts: a public
surface, a recorded corpus with observation levels, measures with
whole-replacement accounting, a structural contract, bounded regeneration
rounds, kill criteria, sensitivity and differential checks, and a declared
residue. Keep language, runtime and tool choices in design.

## Scope

- **In**: the retrospective loop on one declared scope of an existing
  codebase; acceptance, measurement, review and sensitivity artefacts;
  provenance and budget discipline; the prototype's realisation.
- **Out**: automatic scope selection (rejected once, R21); prevention at
  generation time; equivalence proof; adoption into the specimen; test
  authoring, which the TDD workflow owns; languages other than Python
  until an oracle and measures exist for them.

## Boundary Candidates

- Own: surface derivation, corpus recording and replay, measures,
  structural contract, round control, kill criteria, sensitivity scoring,
  differential gating, residue declaration, receipts.
- Adjacent: the TDD workflow (Kiro, cc-sdd, spec-kit) owns tests and
  consumes the missing-test report; the native comparison core owns
  size measures for non-scope accounting; the symbolic-execution tooling
  note owns the verifier rules.
- Explicitly not: proving equivalence; choosing scopes; changing specimen
  behaviour.

## Upstream / Downstream

- Upstream: constraint-guided-compression (research protocol, H8),
  capability-compression-core (native measures), efficiency-baseline
  (reporting semantics).
- Downstream: a future native implementation spec, not yet created.

## Constraints

English. Native Windows for research runs; no WSL execution; no elevated
privileges assumed. Preregistration before execution; dated amendments;
failed predictions reported as failed. Adaptive budgets, never hard caps.
Frozen inputs are never modified during a run.
