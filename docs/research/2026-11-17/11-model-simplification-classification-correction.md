# R124: Runner error classification corrected and controlled

- Written and executed: 2026-11-17, Europe/Berlin.
- Predecessor: [R123](10-model-simplification-infrastructure-classification.md).
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md).
- Status: correction controls passed; full P1 qualification pending.

## Hypothesis and authorization

The owner approved the prepared narrow correction, paused work, then explicitly
resumed it. The falsifiable claim is that runner setup/execution errors become
error/incomplete while genuine assertion failures remain killed. This does not
claim to repair Podman transport or demonstrate model simplification.

The preregistered correction window was 16:18:50.8470396–16:33:50.8470396.
All five controls finished by 16:29:01.8511462. The existing P1 deadline remains
18:25:47.6266045; no elapsed time or consumed starts were refunded.

## Method and evidence

Two runner lines emit MS1_INFRA_FAILURE on setup exceptions or exit 125/126/127.
Each separate case declaration recognizes that marker through the core's existing
error_output_patterns support and points to the new frozen runner. Core, specimen
source, pins, profiles and packs are unchanged. Earlier freezes remain intact.

Artifact root: `D:/BuildArtifacts/lesscode/model-simplification/ms1/`.
Freeze 3 SHA-256: `EB22F3EB9AA93DAD8721DE7D62DE34C1E3CDC8307A261D4DCA3530C6518F85BA`.
The compact [control receipt](../../experiments/model-simplification/classification-v3-receipt.json)
records per-result hashes and completion times. Full evidence is under
`classification-controls-v3/`; preregistration and freeze receipts are under
`infrastructure-revision-3/`.

| Control | Native result |
| --- | --- |
| Injected Podman exec 125 | error, incomplete |
| Injected Podman exec 126 | error, incomplete |
| Injected Podman exec 127 | error, incomplete |
| Injected setup failure 125 | error, incomplete |
| Genuine specimen mutant 50 | killed, exit 1, complete; closing baseline passed |

The four injected controls use a synthetic fixture and replace the Podman command
inside the actual frozen runner. They establish classifier behavior, not specimen
behavior. The positive control runs the real 96-test suite against the genuine
and-to-or mutation with opening and closing baselines.

## Budgets and next gate

43/80 outer starts are charged; model calls remain 0/18. Apparatus totals 1,358/1,400
lines including the 12-line injection wrapper. Specimen pins remain 336 lines;
conservatively including prior structural fixtures and this four-line fixture gives
357/600. Freeze metadata's 1,346 count predates that auxiliary wrapper; this receipt
adds it explicitly. Three authorized apparatus versions have executed.

Fresh S1/S2/B1/B2 qualification checks now use freeze 3, four native workers and
two pytest workers, at most 2,400 seconds each within the unchanged P1 deadline.
No prior partial result or mutant cache is admitted. P1–P4 remain unverified;
production design/tasks remain unapproved. Independent review has not occurred.

## Interpretation

The control hypothesis passed. It supports reliable classification of these error
paths only. Podman responds, and no RAM shortage or general service outage has
been established. The earlier captured command failure and erroneous label remain
valid negative evidence. Full qualification and model efficacy are still separate
unresolved questions.
