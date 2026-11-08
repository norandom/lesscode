# R39: refinement can accept the wrong files and lose its counterexamples

- Report date/time and timezone: 2026-11-08, UTC; written after the probes below.
- Execution interval: retained diagnostic run 16:13:16.8608761Z to
  16:13:24.7288819Z. Earlier exploratory probes in this session found the same
  defects; the retained run includes all cases. Existing-suite corroboration
  was run afterwards in the same session; exact wall-clock endpoints were not
  separately captured, so only its raw runner durations are reported.
- Status: completed diagnosis; correction not implemented.
- Spec: [refinement-evidence-integrity](../../../.kiro/specs/refinement-evidence-integrity/requirements.md);
  corrects [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md)
  requirements 4.2-4.6, 5.5-5.8, 6.3-6.7 and 7.11.
- Hypothesis and predecessors: E3 [R25](../2026-11-04/02-e3-evidence-regeneration.md),
  E4 [R26](../2026-11-06/01-e4-lifecycle-regeneration.md), G1-R R34 and the
  Rust port R36/R37; [R38](05-u5-ceiling-amendment.md) changed accounting only.
- Type: exploratory adversarial investigation, not preregistered efficacy research.

## Decision

The owner's correctness doubt is supported by reproducible software defects.
The existing controller can report a passing best candidate while leaving a
different, failing candidate on disk. It drops runtime feedback before the next
prompt and marks a still-failing counterexample discharged. The Rust process
adapter can turn a timeout or failed process into a full pass. Additional
outcome bugs permit validity without runtime checking and report compression
when no targets or measurements were supplied.

Correct the existing decision and recovery boundaries before relying on them.
The affected general correctness/recovery interpretations of G1-R require
revalidation under parent requirement 7.11. Historical receipts remain unchanged
and valid descriptions of their narrower executed checks. R38's cap and retired
reduction claim are unchanged. No compression success or adoption follows from
this diagnosis.

## Hypothesis and falsification

Hypothesis under challenge: a passing test/replay history and Python/Rust parity
are sufficient evidence that refinement delivers the evaluated best candidate,
uses runtime feedback and reports failures honestly. Falsify it by executing
actual core APIs with distinguishable candidates and deliberately adverse oracle
processes, and observing a contradiction with the approved parent requirements.

No model calls, external services or specimen modifications are needed. The
investigation uses bounded two-attempt fixtures and one-second timeout probes.
Tests assert the required behavior and deliberately remain red on the baseline.
A normal completed-success process is the positive control.

## Frozen inputs and method

- Source: `a7557122295ea859fd0ea786677406b640423ae2`, native Windows at
  `D:/Source/lesscode`. Production modules under test match HEAD; owner WIP and
  pre-existing untracked files are preserved. Source/probe/WIP SHA-256 values
  and before/after equality are in [provenance.json](../../experiments/deconvolution/refinement-correctness/results/provenance.json).
- Owned scope examined: native `regenerator.rs`, its oracle process call and
  tests, Python `regenerator.py`, supplied evaluator callbacks and G1-R test
  drivers. This is not an exhaustive audit of all ported modules.
- MIT repository code; new probes are repository research support. Production,
  tests and experiment support are separate populations. No replacement-size
  result is measured in this investigation.
- Tools: rustc 1.98.1; installed cargo-nextest 0.9.150; CPython 3.13.16;
  pytest 9.0.3; xdist 3.8.0. Actual versions differ from the documented runner
  install/test lock; this is recorded exploratory evidence, not release CI.
- The isolated probe crate depends on the actual native library. It starts from
  native/Cargo.lock and resolves offline; the resulting
  [probe lock](../../experiments/deconvolution/refinement-correctness/results/probe-Cargo.lock)
  is retained. No package installation or production manifest change occurred.
- Models, seeds and live inference cost: none. Candidates and diagnostics are
  deterministic. Development is native Windows; no WSL execution.
- Reproduction: from the repository root in PowerShell, run
  `& docs/experiments/deconvolution/refinement-correctness/run.ps1`.
  It creates a fresh `.runs/r39-*` directory, records identities and commands in
  the script, runs nextest and pytest-xdist, and returns nonzero when either
  probe suite fails. Never run `rust_u5_gate.py` for this diagnosis: it rewrites
  a frozen receipt and is unnecessary here.

## Evidence and results

| Check | Fresh result | Meaning |
|---|---|---|
| Existing native regenerator suite | 45 passed, 0 skipped; exit 0 | Existing assertions hold on current code |
| Existing Python regenerator suite | 59 passed; exit 0 | Reference assertions also hold |
| New native adversarial suite | 1 positive control passed, 8 failed, 0 skipped; nextest exit 100 | Required behavior is violated by actual native code |
| New reference adversarial suite | 5 failed; pytest exit 1 | Five controller defects are shared with Python |

Raw logs: [existing native](../../experiments/deconvolution/refinement-correctness/results/existing-native.log),
[existing reference](../../experiments/deconvolution/refinement-correctness/results/existing-reference.log),
[native probes](../../experiments/deconvolution/refinement-correctness/results/native.log),
[reference probes](../../experiments/deconvolution/refinement-correctness/results/reference.log).
Probe sources: [Rust](../../experiments/deconvolution/refinement-correctness/probe.rs),
[Python](../../experiments/deconvolution/refinement-correctness/test_reference.py).

| Finding | Reproduction and root cause at a755712 | Required correction |
|---|---|---|
| F1: process failure becomes pass | Write `ok=true`, then exit 2 or sleep past timeout: adapter returns 0 and reward 1.0. An `ok=true` payload with 0/1 passed and a failure also passes. `native/src/regenerator.rs:2018-2038` captures process code but only mentions it on missing payload; `ok` overrides execution and counts. | Admit a result only with protocol-consistent execution and observations; retain infrastructure failures. |
| F2: runtime feedback never reaches the next turn | First candidate fails with `R39_known_counterexample`; next submitted prompt lacks it. Rust `2125` resets feedback; `2337-2343` only audits it. Python `651` erases the assignment at `741`. | Carry applicable feedback to actual dispatch and check the submitted payload. |
| F3: best record and candidate bytes disagree | Width 2: first `f()` returns 1 and passes; second returns 2 and fails. Outcome says accepted, best first rollout, score 1.0, validity true; delivered `cand/src/mod.py` returns 2. Rust `2137-2138` deletes the prior tree; `2300-2359` selects metadata only. Python has the same lifecycle. | Retain and deliver the exact selected candidate with matching evidence. |
| F4: failure discovery counted as discharge | One failed witness ends with unresolved obligations `["kill"]`; its concrete counterexample identity is absent. Rust `1819-1824` calls a known obligation discharged if its text differs from the generic failure-kind string. Python `_discharged_obligations` does the same. | Discharge only with positive resolution/applicability evidence for the selected candidate. |
| F5: stopping threshold establishes validity | Set threshold 0.5, omit runtime evaluator: tier-1 score 0.5 yields accepted and validity true. Rust `2327`, `2348`; Python `733`, `750`. | Separate search stopping from behavioral validity. |
| F6: absent targets imply compression met | No targets or measurements yields `compression_met=true`. Rust `2357` tests `static_targets.is_none()`; Python `751` tests `not static_targets`. | Compute the outcome from actual whole-replacement evidence; absence is unmeasured. |

The three F1 probes plus one probe each for F2-F6 explain the eight Rust
failures. Five Python probes cover F2-F6. No claim is made that these exhaust all
defects. F1's process-exit issue also appears by inspection in the Python
research adapter `g1r_live.py:295-315`; the fresh process probes target Rust.

## Why the earlier evidence did not prevent this

E3/R25 already demonstrates an in-domain public `capture_payload` mismatch:
after serialization fails, the original retains an `AuditedAbsence` and the
candidate does not. Its frozen oracle cannot call that operation. E4/R26 records
F1-F7, including F3's lost partial CLI output and F5's changed binding time.
Some E4 cases are outside declared input types; the E3 witness is not. These
historical findings were read, not re-executed during R39.

The current native E4 transition test at `native/tests/regenerator.rs:1488-1498`
explicitly replays the tier-2 result with `evaluate_tier2_runtime(0, None, {})`.
Its positive result does not exercise the process adapter or observe fresh
candidate behavior. The BestOfN tests check best score/label, but the new probe
checks distinguishable delivered bytes. Shared expected outcomes copied from
the Python reference cannot expose defects shared by both implementations.

The specs' intent remains useful. Their coverage mapping overstated what the
selected tests established. Hash integrity establishes identity of recorded
evidence, not truth or completeness of the claim inside it. Passing traces
establish agreement on the observation boundary, not on an unobserved effect.

## Limits and interpretation

The positive control and red probes establish these deterministic defects at the
recorded source identities. Existing-suite green is narrowly scoped to
regenerator; no full repository suite, new E3/E4 differential run, live model
efficacy trial, mutation campaign or cross-platform CI ran. Synthetic controller
probes establish machinery defects, not compression potential. No production
source was corrected, and the owner WIP was not committed as part of this work.

## Tool/spec consequence and next gate

The [corrective requirements](../../../.kiro/specs/refinement-evidence-integrity/requirements.md)
and [adjusted concept](../../../.kiro/specs/refinement-evidence-integrity/concept.md)
make candidate/evidence identity, causal feedback, obligation state and honest
outcomes explicit. The [verification proposal](../../../.kiro/specs/refinement-evidence-integrity/verification.md)
adds independent transition properties and evaluator fault injection, followed
by real-code qualification. Routine bug correction and a later compression
efficacy claim are separate gates.

Requirements review is inline and approval remains pending; design and
production tasks are not generated. Parent specs receive a dated notice of
affected claims without retroactively editing frozen receipts or approvals.
Slice B adoption remains unscheduled. No-growth cap 7,004 and U7 remain in force.

## COBESY check

Conclusion first: the doubt has concrete support. The falsifier is executable;
logs and source identities allow replay. Negative results are useful evidence,
not a reason to weaken assertions. Historical evidence and new observations are
separate. The proposed recovery preserves the original and failed attempts;
neither a green suite nor this diagnosis is inflated into universal correctness.
