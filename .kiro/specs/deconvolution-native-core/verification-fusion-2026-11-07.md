# End-to-end verification of the .kiro workflow: deconvolution-native-core

Written 2026-11-07 (owner request). The end-to-end verification process is ported from
`../QOpenBB/verification/` (runner pattern) and `../QOpenBB/.kiro/breadth.toml`
(evidence ledger), adapted to a CLI-only workflow: **no browser, no Playwright**
(owner, 2026-11-07). The .kiro workflow here is command-line driven, so the E2E runner
drives the CLI and the spec state, step by step, fail-closed.

## E2E process (steady, step-by-step, receipted)

1. **One phase at a time, serial, stop-on-first-failure.** The runner walks the kiro
   phases in order (pre-kiro owner gate -> requirements -> prototype gate/spikes ->
   design -> tasks -> per-task implementation+review -> final validation) and refuses
   to start a phase unless the previous phase's gate checks passed (spec.json approvals
   plus the executable phase checks in gates.md). The failing gate is named in the
   receipt — fail-closed, same discipline as loop req 3.
2. **Runner shape** (mirrors `../QOpenBB/verification/runner.py`): a Python runner via
   `uv.exe run --no-project --no-python-downloads --python 3.13` with hard per-command
   timeouts (process-group/taskkill cleanup on Windows), fixture-only scope, and an
   `evidence.json` per step recording revision, exact command, environment, exit code,
   counts, timestamp. No credentials in evidence; hashes over bytes as stored.
3. **Per phase, the runner executes the phase's checks**: phase-gate checks
   (`requirements_traceability`, `spike_verdicts_complete`, `design_evidence_coverage`,
   `task_smoketest_coverage` — all reading a committed machine-readable evidence
   registry, failing on any unmapped requirement or missing verdict), the per-task
   smoketests via `cargo nextest` (JUnit via quick-junit 0.8.0), `scope verify` hash
   checks, and the fixture-replay suite (insta 1.49.0) against committed E3/E4
   artefacts. Receipts and artefact hashes are verified at every step.
4. **Per-task independent review** (`kiro-review` protocol): fresh reviewer context,
   plan + diff + two trees only, no regenerator rationale; the task's named smoketest
   must pass on both trees; review verdict APPROVED/NEEDS_FIXES recorded in the
   evidence registry.
5. **Final validation gate**: full suite green on native Windows; the five-target
   release matrix green in CI (native runners); byte-for-byte E3/E4 replay; complexity
   caps pass (`complexity-budgets.json` checker); release archives + SHA256SUMS.
6. **Evidence ledger**: `.kiro/breadth.toml` (capabilities -> owning spec criteria ->
   planned checks -> actual test paths -> status/evidence). Planned checks are not
   completed tests; status changes only with reviewed evidence. Reconcile it in the
   same commit as the owning spec, per the QOpenBB completion-checklist rule.

## Verification techniques supporting the 1:1 strategy

Each technique below verifies specific gates 1:1 (mapping in gates.md); all are
executable in CI on Windows/Linux/macOS and receipted.

| Technique | Tooling | Verifies (1:1) |
|---|---|---|
| Snapshot / golden replay | insta 1.49.0 against committed E3/E4 artefacts | every artefact-producing gate: surface, measures, receipts, residue, missing-test report, held-out replay (byte-for-byte) |
| Exit-code contract tests | per-subcommand tests asserting the preregistered 0/1/2/3 decision table | acceptance, void rules, gate refusals, invalid input — the fail-closed semantics |
| Schema / contract validation | schemars 1.2.3 + jsonschema 0.60.2 at every artefact boundary | artefact formats stay byte-compatible with the prototype; new alphabet => new corpus version |
| Hash / receipt chains | sha2 over bytes as stored; append-only atomic receipts (fs4) | provenance (req 11), edit-scope verify, mutant restore, CRLF-byte identity |
| Property-based generation | proptest 1.12.0 seeded, corpus JSON | sequence generation beyond seed tests (req 2.6); held-out corpora with fresh seeds (T7) |
| Mutation testing | cargo-mutants 27.1.0 / cosmic-ray 8.7.0 / StrykerJS 10.1.1 | sensitivity scoring (req 8) and the protection number behind "zero divergence" |
| Verifier-gate refusal tests | four per-pair gates + failed-control tests replaying E4's void | differential step (req 9): controls must fail closed; inconclusive never = equivalent |
| Traceability lint over the evidence registry | phase-gate checks parsing the committed registry JSON | kiro phase gates (AC-2): no phase opens without its evidence; no unmapped requirement |
| Cross-OS CI matrix | native runners, 5 targets, `cargo build --release --locked --target` + nextest | cross-platform operability of every smoketest (AC-4); buildability (spike S8) |
| AI-generated probe suites ("AI testing capabilities") | a model generates concrete probes from plan + diff + two trees only (E4's 128-probe review pattern), executed on both trees; every probe's agreement/disagreement receipted | independent review (req 7) and oracle-blind difference discovery (T6/F1-F7 pattern); probes never edit the trees |
| Missing-test report -> AI-written tests | `score --missing-tests` output drives test writing in the TDD workflow (owner stance: the loop reports, it does not write specimen tests) | req 8.7; closes survivor gaps with named distinguishing inputs |
| Held-out corpora | new seed after acceptance, recorded twice, replayed on original first | T7 trust row: acceptance cannot have been tuned to held-out inputs |

## Explicitly not used

- Playwright / browser E2E: not applicable to a CLI workflow (owner, 2026-11-07).
- Live stores, model endpoints or heavy jobs inside smoketests: those live only in
  spikes (spikes.md), gated on their verdicts; the smoketest suite is pure fixture
  replay and runs in seconds-to-minutes.
- Performance/hardware metrics as acceptance gates (steering product.md): out of scope.
