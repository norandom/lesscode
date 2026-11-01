# NS01 self-application result: reviewed and adopted

Task 7.2, requirements 5.3–5.8. The frozen native evaluator returned **accepted**
with no reasons or check findings. The isolated candidate is tested and measured;
the exact reviewed contracts file was adopted after independent approval. The
controller verified the live baseline hash before copying and the reviewed hash
after copying. Fresh live contract checks pass: 70 tests and 104 subtests. See
[adoption.json](adoption.json). No external specimen repository was changed.

The hypothesis reuses the existing baseline `_fail`, `_object`, `_string`,
`_choice`, `_hash` and `_sequence` validators. Three small wrappers preserve
`reason=`, `fields=` and the policy choice annotations. Policy number, relative
path and UTC implementations remain intact. Both complete contracts modules,
including the shared implementation and all wrappers/imports, are counted.

| Native owned Python measure | Baseline | Candidate | Delta |
| --- | ---: | ---: | ---: |
| Logical statements | 1599 | 1570 | -29 |
| Physical lines | 2342 | 2310 | -32 |
| Decisions | 437 | 420 | -17 |
| Functions | 183 | 180 | -3 |

The fixed minimum was 25 statements, with no physical-line, decision or function
increase. All bounds pass. Dynamic internal call layers remain unavailable;
three forwarding adapters introduce calls and no call-depth improvement is
claimed. The 707 baseline and 736 candidate unresolved capability call locations
have explicit ownership/contract reviews; reviewing them does not resolve dynamic
dispatch. Wider production, tests, fixtures, dependency and research totals are
retained in [counts.json](counts.json).

## Paired checks and identity

- Both sides: unchanged relevant unittest suites, 171 passed.
- Both sides: final pytest-xdist full regression, 515 unchanged identities,
  504 passed and 11 existing Windows/platform skips. Baseline 777.94 seconds;
  candidate 767.35 seconds in the pytest summaries. Time is operational evidence,
  not a compression acceptance measure.
- Both sides: seven independent test methods covering all 24 frozen JSON
  obligations plus signatures, types, order, errors, serialization and additional
  numeric/path/time/container boundaries.
- Both sides: deletion of the appropriate duplicate-identity guard produced four
  expected failures. Malformed fingerprint and unknown-field controls also pass.
- Existing native checks: 102 passed with locked Cargo dependencies.

The final command uses the pinned CPython interpreter, pytest 9.1.1, xdist 3.8.0,
four loadfile workers per side, explicit external config/root/pythonpath,
disabled pytest cache and external JUnit output. Frozen source guards verified
the package and both contracts modules in the controller and every worker.
All four workers on each side collected the same 515 identities as preflight.
Raw commands, logs, JUnit, worker origins and hashes are bound by
[evidence.json](evidence.json); the final check definition is
[parallel-v3-check-definition.json](parallel-v3-check-definition.json).

[integrity.json](integrity.json) verifies all 924 snapshot files: the baseline is
unchanged and the candidate changes only policy contracts. Original dirty
evaluation code/tests remain intact. The live checkout differs only in the exact
authorized controller task-progress and parallel-test-tooling paths enumerated
there. Frozen checks and copied/actual analyzer tool bytes remain unchanged.
The evaluator executable and runtime source remain unchanged; its live README
has the separately authorized test-tooling documentation change.

## Chronology and retained failures

The independent expected runner and corpus were frozen before any candidate edit.
The meaningful pre-edit RED control removed duplicate checking and failed four
checks. The temporary flag-OFF import failure is protocol scaffolding, not the
substantive adverse test. The first candidate accidentally removed still-used
`re`, causing seven existing-test errors; restoring the import repaired the same
hypothesis without changing the target or weakening a check. Its original source,
patch and raw failure log remain available.

Located ownership reviews were completed after that first candidate edit and
honestly re-frozen. The initial full unittest pair each found one identical
execution-context failure: copied roots lacked their frozen `.gitattributes`.
The single failing test passed after restoring that input. A subsequent initial
pytest attempt discovered live ancestor configuration and was not admitted.
The final fresh trial fixes external config/root/pythonpath, hashes a worker
source guard and repeats every paired check. Earlier results were not relabeled.
The user explicitly authorized the separately frozen parallel-runner change.
The first native admission invocation had ambiguous duplicate artifact roots;
that error is retained and the successful invocation uses one explicit root.

## Artifacts, cost and scope

- [result-summary.json](result-summary.json): native accepted status and identities.
- [native-result.zip](native-result.zip): complete unchanged emitted result JSON
  and Markdown, including every metric and limitation.
- [candidate.patch](candidate.patch) and [candidate-contracts.py](candidate-contracts.py):
  corrected replacement; SHA-256
  `65e939cc9e1d3acac3b0aed289d0283e9a160f28875a5f5a0d79cdb509a8dcfd`.
- [candidate-source.zip](candidate-source.zip): changed-file overlay only. Extract
  the unchanged sibling `../baseline.zip`, then overlay this archive to reproduce
  the complete 924-file candidate with identity
  `727540da559b9fd1b477a20466151e26dda865ab7f8b9c4dd3bb55fea2c941e4`.
- [preliminary-freezes.zip](preliminary-freezes.zip),
  [incomplete-preliminary-native.zip](incomplete-preliminary-native.zip) and raw
  preliminary logs preserve rejected/incomplete execution phases.
- [footprint.json](footprint.json) separates new check and one-off research
  apparatus from production. These additions and evidence records mean this is
  not a claim of net whole-repository shrinkage. The candidate adds no production
  dependency; the user-requested final test runner has separately pinned test-only
  dependencies. No maintenance-effort reduction is inferred from counts.

One manually selected responsibility and one reuse hypothesis were trialed,
with an initial defect and a repaired source revision. Mutants are controls;
repeated runs are not independent efficacy experiments. Zero replacements have
been adopted by this implementer.

This completes the technical Python self-application execution and supplies a
measured candidate for independent review. It does not close the final maturity
review, demonstrate general compression efficacy or establish whole-project
maturity. Rust and TypeScript self-compression are untested here. Checks are finite
and Windows/CPython-specific; arbitrary hostile subclasses, monkeypatch behavior
and all Python objects are not proved equivalent. The independent runner lacks a
separate empty-string literal case; whitespace/NUL/non-string cases and unchanged
existing tests cover the broader string boundary. Hashes detect changes and do
not provide hostile-writer isolation.
