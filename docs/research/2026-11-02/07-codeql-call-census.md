# R07: keep every call site before attaching target information

Report/discussion date: 2026-11-02, Europe/Berlin. CG-G0b is an extraction
prerequisite, not a compression experiment. Predecessor: [R06](06-codeql-extraction.md).
Spec: [constraint-guided compression](../../../.kiro/specs/constraint-guided-compression/brief.md).

## Finding

The missing TypeScript callback was a query-join defect. CodeQL supplied a target
without a name. G0 required a name for its resolved row and the absence of any
target for its unresolved row. The callback satisfied neither projection.
G0b starts with syntax sites independently and attaches target information in
additional rows. Both languages now retain all seven invocation sites, including
the callback and exception constructor.

The callback's unnamed target is evidence about CodeQL's representation, not
proof of a particular runtime callback implementation. Likewise, a constructor
without a target under these APIs is not proof that CodeQL knows nothing about
the builtin. Preserve those distinctions in the modeler's input contract.

## Method and boundaries

[Plan](../../experiments/constraint-modeler/g0b/plan.md) was written before the
queries ran; [reproduction instructions](../../experiments/constraint-modeler/g0b/README.md)
and [command receipt](../../experiments/constraint-modeler/g0b-01/receipt.json)
retain the setup. G0's unchanged Python/TypeScript source and persisted databases
were reused. No source extraction or production rewrite was needed for this
revision. Frozen query packs remain Python 7.2.3 and JavaScript 2.9.0 on CodeQL
2.26.3; uv-managed CPython 3.12.10 and Node v24.22.1 run the harness.

The new value-flow query reaches the callback argument through write/read
helpers in both languages and excludes the unrelated constant input. This tests
one cross-function state chain. It does not test alias precision, overwrite
ordering, or arbitrary callback effects. The syntactic CFG query separately
tests forward and reverse connectivity around the finally write.

The fixture's throw condition is concretely false: `consume(7) < 0`. A CFG edge
is therefore not evidence of an executable exception path. This limitation was
recorded in the G0b plan before execution. Reachable exceptional behavior and
post-state need another control, rather than a stronger interpretation of these
results.

## Measured result

The [assessment](../../experiments/constraint-modeler/g0b-01/assessment.json)
passes **9/9 checks per language**: three output-presence checks and six semantic
fixture assertions. Do not compare this denominator directly with G0's 13 checks;
the questions changed. The initial G0b query set compiled without repair.

| Observation | Python | TypeScript |
| --- | ---: | ---: |
| Syntactic invocation sites / expected | 7 / 7 | 7 / 7 |
| Callback retained | Yes | Yes |
| Input flows through state helpers to callback | Yes | Yes |
| Unrelated constant excluded | Yes | Yes |
| Forward CFG connection / reverse excluded | Yes / Yes | Yes / Yes |

[Fresh-process replay](../../experiments/constraint-modeler/g0b-01/assessment-replay.json)
matches all six decoded result sets (three per language). Frozen fixture, query
and lock hashes match between runs. Runner-only metadata changed for replay to
automatically retain the plan/assessor and explicit database reuse path; the
first run retains those same files by a separate copy. Both runner versions and
command receipts are preserved. Replay demonstrates repeatability, not stronger
semantic coverage.

## Next decision

Keep the complete-site contract. Challenge its semantic boundary with an aliased
versus independent object pair, an overwrite, and a genuinely reachable
raise/throw with finally cleanup. Freeze expected observations and execute the
fixture as well as querying it. Missing/overapproximated relations must become
explicit model obligations, not inferred equivalence.

The overall [prototype gate](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md)
remains **revise**. Real-code G1, the LLM model/revision loop, SMT checks and
useful whole-replacement compression are unestablished. No compression savings
were measured here. No external specimen or production implementation changed.

COBESY framing: retain the failed G0 record, explain the specific defect before
the new evidence, and make the next falsifiable question visible. The audience
is the owner and the next research agent; a passing setup check must not become
an inflated claim that the compressor works.
