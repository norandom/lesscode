# Brief: deconvolution-native-core

Written 2026-11-06 at the owner's request ("start a spec ... so that we don't produce much code"), after R26 and
the run report. It precedes requirements. Nothing here is approved for implementation, and the owner has said
they are not ready to commit implementation effort yet; the purpose of this spec is to make effort and porting
visible.

**Amendment 2026-11-07 (owner request).** The OpenUltraCode fusion run of 2026-11-07 (critique-revise-vote;
panels A/B; decider = the selected model) was arbitrated into an implementation plan and drafted into
[requirements.md](requirements.md) amendment 2026-11-07 (generated, NOT approved). New spec files:
[gates.md](gates.md) (1:1 gate map + phase-gate checks), [verification.md](verification.md) (E2E verification of
the .kiro workflow, CLI-driven, no Playwright), [spikes.md](spikes.md) (S1-S9), [breath.toml](breath.toml)
(scope guard — editing it is the only way to change scope). Project-level process files added:
[../../breadth.toml](../../breadth.toml) (capability/evidence map, QOpenBB convention),
[../../steering/complexity.md](../../steering/complexity.md) and [../../complexity-budgets.json](../../complexity-budgets.json)
(complexity caps: per-file/per-function limits plus spec-level budgets <= 3,500 / <= 4,600 hand-written lines,
<= 12 subcommands, machine-enforced by a checker foundation task). Still plan-only: no Rust code until the
pre-kiro owner gate (the four open questions below + recorder-imports ratification via spike S9) passes.

## Problem

The deconvolution loop works as a measured research instrument on two Python scopes (R25, R26), but it exists as
hand-assembled Python scripts per experiment, about 1,400 lines of apparatus per scope, of which the recorder
(534 lines for E4) is rewritten for every scope. It cannot be reused on another codebase, another language or
another machine without repeating that work.

## Current State

- The agnostic contract (acceptance rule, receipts, budgets, round control, turn rule, kill-rate convention,
  residue) survived two scopes and a change of regenerator without a change of text (run report section 8).
- The language-bound parts (surface, measures, recorder, mutation, solver) exist for Python only, plus one Rust
  in-process recorder (R20).
- The existing native crate `lesscode-core` (`native/`) already uses ast-grep-core, tree-sitter, serde, sha2 and
  clap on Windows, Linux and macOS.
- Two surveys (2026-11-06, see research.md) map every component to existing crates and tools and evaluate
  debugger, dynamic-instrumentation and store-level capture.

## Desired Outcome

A specification of a cross-platform Rust tool suite (one library with a reusable core, a set of CLIs,
purpose-built input generation and instrumentation) that runs the loop on specimens written in Python,
TypeScript/JavaScript and Rust, where:

- every component names the existing crate or tool it is built on, and the hand-written part is small and
  stated per component;
- per-language and per-scope knowledge is data (rule packs, alphabets, normalisations), not code;
- the effort and the porting cost per language are visible before any code is written;
- every claim of feasibility is tied to evidence or to a named spike.

## Approach

Reuse first: assemble the suite from existing crates and tools (research.md section 2), write only the
agnostic decision logic and thin adapters. Instrumentation is store-level and driver-level, not debugger-based
(research.md section 1): the driver observes the public surface directly; SQL is captured at the store
(PostgreSQL wire proxy, SQLite trace) with in-process hooks as fallback. Debuggers are out of the core. The core
is regression-tested against the committed E3 and E4 artefacts. Spikes precede design approval where a reuse
choice has no evidence.

## Scope

- **In**: the reusable core library; CLIs for surface, record and replay, accept, measure, mutate and score,
  differential, residue and reports, receipts; per-language adapters for Python, TypeScript/JavaScript and
  Rust; a declarative format for scope alphabets and normalisations; the regenerator turn program; native
  Windows first, Linux and macOS; spikes for the unknowns.
- **Out**: scope selection (rejected in R21, unbuilt); a generic rewrite engine; MCP or plugin surfaces
  (steering tech.md); performance targets (steering product.md); debugger-based recording in the core; a symbolic
  rung for TypeScript/JavaScript (none exists).

## Boundary Candidates

- Core (language-agnostic): receipt, guard and budgets, tree hashing and edit scope, acceptance and round
  control, turn program and model client, sensitivity scoring and sampling, residue and reports.
- Language adapters: surface and measures rule packs, driver shim, mutation-engine reader, solver bridge.
- Store adapters: PostgreSQL wire proxy, SQLite trace, driver-hook fallback.
- Scope descriptors: alphabet, fixtures, clock, normalisations, as data.

## Out of Boundary

The specimens themselves; tests of specimens (owner stance: tests belong to the TDD workflow); adoption of any
candidate into a specimen; the research prototype under `docs/experiments/` (kept frozen as evidence).

## Upstream / Downstream

- Upstream: `deconvolution-loop` (algorithm-agnostic requirements and the prototype design; this spec realises
  it natively); `capability-compression-core` (the existing native crate and its conventions).
- Downstream: a third preregistered scope, possibly run on the native suite; later, prevention at generation time.

## Existing Spec Touchpoints

- Extends: `deconvolution-loop` (requirements 1 to 12 are the behavioural contract; the four amendments proposed
  in R26 should be adopted there first).
- Adjacent: `capability-compression-core` (shares `native/`; decide in design whether to extend that crate or
  add a workspace member).

## Constraints

- Native Windows first; WSL is CI-only; no symlink privilege or elevation; Linux and macOS supported.
- Permissive licences; Cargo-installed tools pinned with `--locked` (steering tech.md).
- Minimal hand-written code: a component with a maintained crate or tool must use it unless the requirements
  give a reason not to.
- Byte-compatible artefact formats with the prototype where the committed E3/E4 artefacts serve as regression
  fixtures (hashes over bytes as stored).
- Prototype-first steering applies: spikes before design approval for every reuse choice without evidence.

## Open questions for the owner (Q&A)

1. Extend `native/lesscode-core` or start a separate workspace member for the loop?
2. Which specimen and store should the first Rust-target scope use (mmhmem with SQLite is the cheapest: built-in
   trace hook, R20 recorder exists)?
3. PyO3 embedding or a subprocess shim for Python (embedding is faster; a subprocess keeps the specimen's own
   interpreter and isolation)?
4. Should the regenerator program move into the suite now, or stay a Python script until a third scope runs?
