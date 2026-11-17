# R117: Syntax census runs; observation pins detect alias and error-order changes

- Report: 2026-11-17, 13:10 Europe/Berlin.
- Execution interval: approximately 13:05-13:10 Europe/Berlin; session receipts retain command timings.
- Status: completed for census/pin preparation; P0 incomplete.
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md).
- Hypothesis: H-MS1 preparation; predecessor [R116](03-model-simplification-runtime-preparation.md).
- Type: exploratory instrument and pin preparation, not model trials.

## Decision

Retain the initial Rust census and proposed observation pins. They establish useful
preparation evidence but do not admit the simplicity profile or complete P0. Continue
with binding/alias resolution, atomic predicate accounting, remaining controls and
the full execution freeze. No model candidate or useful simplification exists yet.

## Hypothesis and falsification

The existing parser can expose relevant syntax with deterministic source locations,
and explicit observations can distinguish value-preserving but identity-changing
rewrites and changed validation precedence. Parse/selector errors must be rejected;
known-bad changes must fail the pins. Passing these limited checks cannot establish
full measurement fidelity or universal behavior preservation.

## Inputs and method

Source remains `3a214ff13ee768535481f71258ac2b3529306870`. Rust research crate:
[spike](../../experiments/model-simplification/spike/Cargo.toml), with existing
lesscode-core parser/tree functions and existing pinned serde dependencies. The
independent Cargo lock starts from native's lock; production files/locks are unchanged.
Build outputs use the configured `D:\BuildArtifacts\cargo-target` directory.

The profile is explicitly `syntax-census-draft-1`: grammar kinds live in data,
unknown kinds and parse failures are errors, and all selected effects are unresolved.
Owner identities include source offsets and therefore are not cross-revision matches.
Decision-expression counts are not atomic predicate counts. Every report declares
`simplicity_status: not_measurable` with these limitations.

The [pins](../../experiments/model-simplification/pins/test_state_observations.py)
and [filter examples](../../experiments/model-simplification/pins/test_filter_observations.py)
run on the frozen source inside the R116 runner image with network disabled, using
pytest-xdist with two workers. New tests are separate overlays; upstream tests remain
unchanged. Exporter tests use deterministic component fakes and observe aliases,
callback payload/order and partial failure state; they do not exercise construction.

## Evidence and results

- Native `cargo nextest run --offline --manifest-path docs/experiments/model-simplification/spike/Cargo.toml`:
  4 passed. Controls cover retained decision/effect syntax, parse failure, invalid
  selector and same-named methods in distinct owners. A locked offline build passed.
- Both real files parsed and emitted raw census reports under
  `D:\BuildArtifacts\lesscode\model-simplification\ms1\census\`.
  Their status remains not measurable; counts are not qualified simplicity baselines.
- Unchanged-source proposed pins: 26 passed (5 state observations, 21 filter cases).
- Alias control replaced `self.tweets.extend(unique_new)` with allocation/assignment:
  2 failed, 3 passed, pytest exit 1 as expected.
- Precedence control moved month-start validation ahead of year validation:
  3 failed, 18 passed, pytest exit 1 as expected.
- Both controls were restored from original file bytes afterward; the pin container
  `lesscode-ms1-pins` is stopped. Original tracked source remains clean.

Logs and control files are retained below `ms1/runtime/`: state-pins-baseline.log,
state-pins-alias-control.log, all-pins-baseline.log, filter-precedence-control.log,
exporter-alias-mutant.py and tools-precedence-mutant.py. These are targeted preparation
checks, not existing step-check verdicts or full-suite results. No model calls ran.

The first Rust build waited for another build's shared Cargo lock; no other process
was interrupted. All work ran in the foreground. No additional runtime/dependency
framework was introduced.

## Limits and accounting

The census lacks local/nonlocal binding classification, alias effects, stable
cross-revision identity and atomic predicates. Pins still need broader callback,
checkpoint, public-caller and custom-input coverage plus the full adverse-control set.
No baseline/test declaration has been frozen into private Git step histories yet.

Current maintained apparatus: 188 rustfmt-formatted Rust lines plus 40 PowerShell
lines, 228 of the 1,400-line bound. New specimen pins: 131 physical lines of the
600-line bound; canonical test formatting remains to be recorded before final freeze.
Profiles/manifests/locks are retained separately. No deletion refunds a later budget.

## Tool/spec consequence and next gate

P0 remains open. No production task group is marked complete, no design approval
is inferred and no model trial may start from these partial artifacts. Continue the
already-authorized bounded preparation without another permission request.

## COBESY check

The controls provide observable disproof of specific wrong rewrites. Unknown effects
are explicit; parser success and stronger pins are not credited as simplification.
Failure records are retained and the next missing capabilities are named.
