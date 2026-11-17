# R125: Both unchanged responsibilities qualify under the corrected runner

- Written and executed: 2026-11-17, Europe/Berlin.
- Execution: 16:34:17–17:43:45, within the existing 18:25:47.6266045 P1 deadline.
- Predecessor: [R124](11-model-simplification-classification-correction.md).
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md).
- Status: P1 qualified; model efficacy remains untested.

## Hypothesis and method

The operational hypothesis was that the corrected frozen evaluator could produce
complete, reproducible native observations for both unchanged responsibilities.
Four fresh private repositories used freeze 3, four native fault workers and two
pytest workers per suite. No partial result or mutant cache from an earlier run
was reused. The 2,400-second check bound and existing global deadline were retained.

## Results

| Check | Seconds | Mutations per side | Detected / surviving | Verdict |
| --- | ---: | ---: | --- | --- |
| S1 | 1,110.53 | 64 | 53 / 11 | accepted |
| S2 | 1,144.68 | 64 | 53 / 11 | accepted |
| B1 | 916.36 | 51 | 50 / 1 | accepted |
| B2 | 914.69 | 51 | 50 / 1 | accepted |

All eight mutation runs are complete. All 32 worker closing baselines passed.
For each case, all four populations match on mutation number, operator, source
hash, classification, reason and exit code. Only timing and worker metadata are
excluded from that comparison. No error/invalid/timeout classification is hidden
inside these totals. Baseline acceptance is not a candidate success.

The [admission receipt](../../experiments/model-simplification/p1-v3-admission-receipt.json)
links the full evidence index and admission supplement under
`D:/BuildArtifacts/lesscode/model-simplification/ms1/`. Freeze 3 and its retained
P0 evidence were reverified. The original specimen remained at
`3a214ff13ee768535481f71258ac2b3529306870` with a clean tracked tree.

## Controls and admitted measurement

The five new classification controls passed in R124. Earlier structural, lifecycle
and behavioral controls remain linked with their original execution dates. The
core, measurement profile/code, source and pins are byte-identical across freezes.
The only runner change is two marker lines for setup exceptions or exit 125/126/127;
ordinary exit 0/1 paths are unchanged and were exercised by the new assertion
control and full native runs. No old incomplete qualification run is admitted.

The repeated raw measurement files match exactly. The admission supplement now
admits the earlier explicitly conditional baseline record under the bounded
contracts, without rewriting that historical record:

- S target self.tweets: W=4, S=3, D=1; required ceilings W=3, S=2, D=1.
- B complete four-validator responsibility: B=7, P=17; required ceilings B=5,
  P=15. E/W/S/D remain zero. Whole-module companion limits remain mandatory.
- Whole-file envelopes, unresolved effects and all helpers/support remain counted.
  An unchanged unresolved-effect total is insufficient without manual review of
  arguments, order, reachability and external effects. No universal alias claim is made.

## Resource accounting and next gate

47/80 outer checker/control starts are consumed; model calls remain 0/18. Apparatus
is 1,358/1,400 lines; pins and synthetic fixtures total 357/600. Earlier failed
attempts and all consumed budgets remain recorded. Three authorized versions ran.

During qualification the owner requested bounded storage. Tool-native cleanup
removed approximately 25 GiB from five inactive lesscode-specific Cargo targets.
Shared caches, active output, frozen binaries and evidence were preserved; C: was
not used for storage changes. The separate AGENTS.md process records limits and
manual cleanup responsibilities. This did not change the experiment or reset its clock.

Proceed within the existing authorization to the six scheduled unedited model
trials, at most two repairs each and 18 calls total, with the original four-hour
candidate window. Use freeze-3 case bundles and retain all outcomes. P2–P4 remain
unchecked; independent review and production design approval are still pending.

## Interpretation

Reproducible qualification now supports attempting H-MS1. It does not establish
useful simplification, universal behavior preservation or a production capability.
Those claims require the separately recorded candidate outcomes and evidence review.
