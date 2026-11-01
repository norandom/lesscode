# Native supplied-pair workflow

Use this reference when evaluating an isolated Python/TypeScript capability pair with the native core. This verified recipe demonstrates replay consistency, not discovery efficacy or universal equivalence.

## Review before preparing

- Keep the experiment hypothesis, alternatives (including retain original), selection rationale and disposition in [experiment-record.md](experiment-record.md). Fix the target and check inventory before editing; label retrospective replay honestly.
- Review entry points, complete transitive owned implementation, wider accounting directories, category rules and explicit exclusions. Include replacement helpers, translation tables, adapters and changed callers outside original files. Review each unresolved relationship as owned or a concrete external contract; uncertainty stays unresolved, never dead code. Review candidate ownership separately without narrowing the baseline.
- Reuse the repository's shared installed-capability inventory. Record its source identity, manifest/lock references and external artifact hash in `capability_inventory`; proposed changes belong in evidence `dependency_changes`, with the repository decision. A missing/stale supplied inventory fails admission. No inventory supplied is a disclosed limitation, not evidence of dependency review. See the implemented [model](../../../../native/src/model.rs) (`InventoryReference`, `DependencyChange`) and [bound-inventory example](../../../../native/tests/workflow.rs) (`preparation_verifies_external_capability_inventory_bytes`) for exact shapes.
- Freeze unchanged check definitions outside candidate source. Run the same required checks against both versions using recorded runtimes: errors, outputs, ordering, dispatch, types, safeguards and interactions as applicable. Retain raw outcomes, source identities, trial/check digests, artifact hashes, tested domains and unverified behavior. Changed checks require a newly frozen trial and paired runs; do not relabel old results.

The [build guide](../../../../native/README.md) owns toolchain/prerequisites. For complete worked schema examples use the ATS [manifest](../../../../docs/experiments/native-replay/ats-run/recorded/manifest.json), [frozen trial](../../../../docs/experiments/native-replay/ats-run/recorded/frozen.json), [check definition](../../../../docs/experiments/native-replay/ats-run/recorded/check-definition.json) and [paired evidence](../../../../docs/experiments/native-replay/ats-run/recorded/candidate-evidence.json). Do not copy their identities or relationship assertions to another specimen. Native analysis is local and compiled; target checks retain their own runtime requirements.

## Commands for a new trial

Run the built `lesscode-core` executable (Windows: `native/target/debug/lesscode-core.exe`). Choose fresh external output paths. Repeat `--artifact-root` for each external directory containing referenced inventory/check artifacts; there is no implicit working-directory search.

```text
lesscode-core prepare --manifest trial.json --baseline BASELINE --artifact-root INVENTORY_DIR --out RUN/frozen.json
lesscode-core compare --frozen RUN/frozen.json --baseline BASELINE --candidate CANDIDATE --evidence checks.json --artifact-root INVENTORY_DIR --artifact-root CHECK_OUTPUT_DIR --out RUN/comparison
```

Prepare freezes source inventory, definitions, target and checks; it neither runs checks nor certifies semantic scope completeness. Compare writes `result.json` and `report.md`. Keep JSON and human reports for failed/incomplete alternatives too. Exit codes: 0 prepared/accepted; 1 rejected/not-demonstrated; 2 incomplete; 3 invalid input, baseline-ineligible or I/O failure. A report without a target or valid evidence cannot establish successful compression. Interpret status and reasons together with unavailable metrics, category/wider-boundary totals, dependency changes and tested-domain limits. Hash source and check artifacts before/after; the evaluator must leave them unchanged.

## Verified recorded ATS replay

Run from repository root in PowerShell after building per the build guide. Choose a new `$run` suffix each time. This re-admits task 6.2's already recorded paired observations; it does not claim to rerun their corpus. For fresh behavioral execution use [the existing ATS runner instructions](../../../../docs/experiments/native-replay/ats-run/README.md); do not invoke the archived collector directly.

```powershell
$ats = 'docs/experiments/native-replay/ats-run/recorded'
$run = '.runs/native-replay/skill-recipe-local'
$core = 'native/target/debug/lesscode-core.exe'
& $core prepare --manifest "$ats/manifest.json" --baseline "$ats/baseline" --artifact-root $ats --out "$run/frozen.json"
& $core compare --frozen "$run/frozen.json" --baseline "$ats/baseline" --candidate "$ats/candidate" --evidence "$ats/candidate-evidence.json" --artifact-root "$ats/baseline-raw" --artifact-root $ats --out "$run/candidate"
& $core compare --frozen "$run/frozen.json" --baseline "$ats/baseline" --candidate "$ats/mutant" --evidence "$ats/mutant-evidence.json" --artifact-root "$ats/baseline-raw" --artifact-root $ats --out "$run/mutant"
```

Expect prepare 0, candidate 0 (`accepted`), mutant 1 (`rejected_behavior`); inspect each exit code immediately. Artifact paths are relative to an explicit root: `$ats` resolves `baseline-raw/raw.json` and the candidate/mutant counterparts. The additional raw root demonstrates repeated-root admission without changing artifact bindings.

Compare the fresh frozen JSON with the recorded freeze before using old evidence. Same inputs must preserve its digest and identities; do not edit evidence to force a match. Compare full candidate and mutant `result.json` values with the recorded comparisons, including tested domains and reasons. Retain both human summaries and machine output with raw command outcomes and before/after input hashes.

The recorded target is the separately declared task 6.2 whole-module source target, not the stricter incomplete task 6.1 target. The candidate has 8820 matched fictional reporting observations and preserved subclass dispatch; the mutant has 2251 mismatches. External services and non-reporting behavior are unverified. Internal call layers remain unavailable. The recorded manifest has `capability_inventory: null` and no proposed dependency changes: it cannot demonstrate inventory admission or new-library suitability. This recipe's [verification record](../../../../docs/experiments/native-replay/skill-recipe/verification.md) retains the observed outcomes and immutable-input receipt. These manually selected results do not establish the skill's general effectiveness.
