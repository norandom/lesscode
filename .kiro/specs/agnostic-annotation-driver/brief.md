# Brief: Agnostic annotation-guided driver

Written 2026-11-10 from the owner's direction of the same day. Status: brief
and prototype gate only. No requirements, design or tasks are generated.

## Problem

A maintainer who points `lesscode-core` at code outside the experiment
specimens gets little from two of its four language contracts.
[R55](../../../docs/research/2026-11-10/01-released-binary-adopter-dogfood.md)
measured it: the mutant generators replace fixed substrings taken from the
specimens (no mutants for 36 of 38 Rust files and 40 of 40 Python files), the
released `analyze` has hand-written rules for three languages and none for
Python. Every new language or codebase therefore needs new hand-written code.
That contradicts the product goal of a reusable way to produce less code with
the same behavior ([product steering](../../steering/product.md)).

## Owner direction, 2026-11-10

1. Top priority: make the process horizontally applicable, with a driver that
   does not depend on the code or the language in front of it.
2. Second priority: hardening against LLM inference drift.
3. The mechanism for both is an annotation-guidance process for the model,
   optimised later with bounded and verified `MIPROv2` (`GEPA` if needed).
4. Model endpoint: Opus for now, Mistral later. One task at a time.
5. Approved shape (owner "approved", 2026-11-10): a new spec, mutation as the
   first consumer of annotations, analysis second.

## Approach to be tested

The model annotates and the core acts.

- The model reads a scope and returns annotations only: a location and a kind
  from a small fixed taxonomy (failure state, branch condition, error
  ordering, state or environment write, pure/shell boundary). It writes no
  replacement code.
- An annotation guide is the driver's instruction set. It is a versioned text
  artifact, the thing a later optimiser changes.
- The core validates every annotation against the syntax tree and the scope,
  rejects what does not check out, and applies generic operators at the
  validated sites. The scope's real tests decide killed, survived or invalid.
- The same annotations are meant to feed `analyze` later (write sites and
  boundaries), which would also cover Python.

## Boundary candidates

- Annotation contract: taxonomy, location form, validation rules, rejection
  reasons.
- Annotation guide: versioning, what it may and may not contain (no
  scope-specific strings).
- Generic operators over validated sites, and the per-language surface they
  need (a node-kind table, nothing else).
- Driver seam: reuse of the existing OpenAI-compatible endpoint
  (`LESSCODE_MODEL_ENV`), retained request and response identities.
- Measurement of agreement between runs and models, as the baseline the drift
  work will need.

## Out of boundary

- `MIPROv2` or `GEPA` compilation, multi-turn `Refine`/`BestOfN` control and any
  drift mitigation. They are the second priority and stay with
  [constraint-guided-compression](../constraint-guided-compression/requirements.md)
  Slice B; this spec only supplies their baseline.
- Regeneration of code, equivalence checking, density measures.
- A general mutation-testing product. Existing tools (cargo-mutants,
  cosmic-ray or mutmut, StrykerJS, PIT) are a comparison arm, not something to
  rebuild.
- Changing the released `mutate` or `analyze` commands before a gate passes.
- The pre-push hook's refactor-branch check (open from R55, separate decision).

## Upstream and downstream

Upstream: the four carrier contracts in `native/src/adapter_traits.rs`
(`AdverseMutantGenerator` first, `StateFrameAnalyzer` second), the endpoint
seam in `native/src/regenerator.rs`, the tree-sitter and `syn` parsers already
locked in `native/Cargo.lock`, and
[refinement-evidence-integrity](../refinement-evidence-integrity/requirements.md)
on adverse controls. Downstream: constraint-guided-compression Amendment 5 and
the Slice B reward gate, which planned to use `deconvolute analyze`.

## Constraints

- [Prototype-first](../../steering/prototype-first.md): the
  [prototype gate](prototype-gate.md) must reach `proceed` before a design is
  approved or production tasks exist. `-y` does not waive evidence.
- The R41 apparatus ceiling (7,173 canonical lines) and the per-file
  complexity budgets apply to anything that later enters `native/src`.
- The model proposes and never decides. No result may be reported as passing
  without deterministic validation and a real test run.
- Native Windows execution; Linux-only tools run in containers, preferably
  through Compose. No execution inside a WSL distribution.
- Model calls cost money and need the owner's go per run.
