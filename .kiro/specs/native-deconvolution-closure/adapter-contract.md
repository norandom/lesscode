# Shared adapter contract v1 — provisional design

Owner: native-deconvolution-closure. Consumers: Python, TypeScript/JavaScript,
Java and Rust adapter specs. Written 2026-11-09. This document makes the
[design](design.md) seam concrete; it is not an implemented or qualified protocol.
All profiles require the [prototype gate](prototype-gate.md).

## Ownership and dependency direction

The core owns request identity, execution limits, outcome aggregation, comparator,
canonical receipts and qualification policy. Adapters own source analysis,
runtime invocation, hooks and language-specific value codecs. A profile cannot
override the four outcome meanings or omit mandatory controls. PureCoreVerifier
and SpecimenTraceRunner implementations receive their language profile explicitly.
Use a separate internal ProfileId registry; do not add language variants to existing
public comparison schemas or modify CLI arguments without an approved compatibility
migration. Shared registration is owned by the core, adapter logic by its spec.

## Request and evidence records

| Record | Required fields and semantics |
|---|---|
| Identity | request_id, schema=1, profile_id/version, source_baseline_sha256, source_candidate_sha256, checks_sha256, adapter_sha256, runtime_identity, dependency_lock_sha256, configuration_sha256 |
| AnalysisRequest | Identity, owned source/caller inventory, frozen FrameSpec, EffectProfile |
| FrameSpec | named state roots, allowed shell entrypoints, read-only paths, permitted writes, success/failure invariants, required observer kinds; derived from baseline/contract, never candidate permission |
| EffectProfile | resolved callable/type identities, allowed effects, alias rules, unsupported forms, hook coverage and assumptions |
| DomainSpec | parameter names and ordered finite tagged values, constraint predicate identity, positive Cartesian case count, deterministic enumeration order, case-set digest |
| ComparisonRequest | Identity, baseline/candidate entrypoints, DomainSpec, comparator_profile, execution limits |
| MutationRequest | Identity, operator/version, source span/hash, control ID, independent expected fault and witness predicate |
| RunRequest | Identity, operation=record/replay/compare/control/regression, argv vector, working/output paths, limits, required hooks, case/step inventory |
| RunResult | events/artifact references, child exit, completion flag, tested-case digest/count, hook coverage, explicit failure classification |
| EvidenceRef / WitnessRef | relative path under owned run root, SHA-256, kind, source/case/step IDs; no arbitrary external path traversal |

Request IDs are unique per attempt; source identities bind the baseline/candidate
role. Validate each field before execution. Unknown schema/version/profile or
missing identity is execution-failed. Adapter diagnostic details cannot
override controller-computed identity or completion checks.

## Tagged values and outcomes

Wire JSON carries booleans/null plus tagged language values:
integer (signed,width,decimal text), float (width,hex bits), text (encoding,units),
bytes (hex), sequence (ordered elements), mapping (ordered key/value pairs),
record (type ID and ordered fields), variant (type/tag/payload), reference
(observation-local object ID). No JSON number carries an unbounded integer or NaN.
Language codecs distinguish undefined from null, float signs, error variants and
relevant alias relationships. Unregistered types/cycles/equality contracts are
unverified. Object IDs may establish within-case aliasing, never cross-run addresses.

A comparison returns exactly one of:
- verified: all required finite cases completed with matching observable outcomes,
  no coverage gaps, complete traces and compatible identities;
  domain ID and evidence digest are mandatory.
- divergent: an attributable behavioral mismatch with source/case/step witness;
  shrinking metadata says none, reduced, or minimal under a named finite strategy.
- unverified: unsupported semantics, unresolved effects or incomplete declared
  coverage/domain. Sampling may accompany this status but cannot upgrade it.
- execution-failed: timeout, crash without a behavioral witness, malformed protocol,
  build/dependency failure, changed checks, exceeded limits or infrastructure error.

Aggregate results retain individual failures/gaps: observed divergence wins the
overall behavioral verdict; otherwise execution-failed precedes unverified,
otherwise verified. Qualification is a separate conjunction of verified behavior,
mandatory control sensitivity, comparable complete metrics and useful compression.
No valid-but-uncompressed candidate is mislabeled behaviorally divergent.

## Transport, events and ordering

Parent stdin supplies one JSON request followed by LF. Adapter stdout is reserved
for UTF-8 JSON event frames terminated by LF; diagnostics use separately drained
stderr. Raw specimen output is captured by qualified runtime hooks and transported
as byte payload events. Bypasses cannot be silently ignored. Separate regression
runner stdout is diagnostic/test-result data, never this protocol.

Each event includes request_id, sequence (starting at zero, strictly increasing),
case_id, step_id, kind and payload. Kinds: ready, state-before, state-after,
sql, write, flush, log, exception, abort, case-complete and run-complete.
Only admitted observer kinds are required for a given profile, but a required kind
cannot be declared absent merely because its hook is unavailable.
run-complete includes expected/completed case counts, case-set digest, trace
coverage and artifact manifest. The controller also requires the expected child
exit and unchanged checks. A claimed completion with missing cases is rejected.

Serial profile v1 uses exact event order. Concurrent support requires a separately
qualified partial-order profile; arrival order across OS pipes is not causality.
Normalize only according to a preregistered comparator profile. Byte-exact golden
contracts retain their current serializer/digest identities; new tagged transport
does not redefine their body bytes.

## Execution limits and resource ownership

Prototype defaults: 10,000 cases, 60 seconds per case, 900 seconds per probe batch,
1 MiB per frame and 64 MiB aggregate protocol/output per batch; build/regression
stage deadline 1,800 seconds. A profile may freeze stricter limits. Increasing a
limit requires a dated pre-run amendment; overflow is execution-failed, incomplete
domain is unverified, and neither can satisfy the gate.

Launch argv directly, not shell-concatenated commands. Resolve cwd to repository/
isolated specimen and outputs beneath D:/BuildArtifacts/lesscode/<profile>/<run>.
Reject drive roots, escaping symlinks/reparse points and output/check overlap.
Honor cache variables. Drain stdout/stderr concurrently and bound capture.
Own process-tree and database lifecycle explicitly; on timeout/abort terminate
owned descendants and retain cleanup evidence. Do not kill unrelated processes.
Keep original checks outside candidate write scope and hash before/after.
This detects accidental corruption, not an OS security boundary against hostile code.

## Mutant and regression records

MutantResult includes control ID/source/operator, baseline health, exercised cases,
exit/evidence and status: killed-with-witness, survived, equivalent-with-evidence,
invalid, not-exercised, timeout, infrastructure-failure.
Freeze mandatory valid non-equivalent controls before testing the candidate.
Equivalent exclusions require evidence, not an after-the-fact scoring adjustment.
An empty mandatory set cannot pass. All mandatory controls must be killed with
independent expected witnesses; a generic test process failure is insufficient.

RegressionResult contains discovered/selected/executed/skipped test identities,
exit status, retries and raw report reference. Unexpected missing/skipped cases
block qualification. JUnit/runner reporters provide regression evidence only.
A retry never erases the first outcome; flakiness blocks deterministic attribution.

## Accounting and change control

Record three distinct quantities: specimen replacement, maintained apparatus,
and test/experiment support. Count all new wrappers, scripts, templates,
configuration and changed callers in the relevant maintained total.
Apply the unchanged R41 7,173-canonical-line ceiling to existing plus added
apparatus paths; do not reset the baseline independently per language.
Per-file defaults and measured exceptions remain separate constraints.
No adapter receives an implicit allocation.

Schema/profile/metric/runtime/hook changes require producer and every affected
consumer fixture revalidation. Conformance cases include unknown version,
wrong source, malformed/truncated event, wrong count, altered check, unsupported
value, signed zero/NaN, reordered flush, timeout with descendants and false
completion. The core owns shared fixtures in native/tests/adapter_contract.rs;
each language owns its own codec/runtime integration fixtures.
