# Preregistration: usefulness check of the E3 missing-test report

Written 2026-11-05, before any test for this check exists. Spec: deconvolution-loop task 2.1
(requirements 8.7, 10.1, 10.3, 11.1, 11.2). This check runs outside the Loop. The Loop never adds tests
to the specimen. The tests written here are an experiment artefact and are deleted afterwards.

## Question

Does `missing-tests.md`, as written, let a fresh agent write tests that close the gaps it names?

## Procedure

1. **Test writer.** The writer is a fresh agent with no context from this session. It receives only
   `results/missing-tests.md` and read access to the ORIGINAL tree (`.runs/e2-facdrone/source`, or the
   check copy `.runs/e3-facdrone/orig-check`). It does not see the candidate, the mutants, the mutant
   sample, the T5 drivers or this file's predictions. It writes one pytest file that would sit in `tests/unit/`
   of the facdrone tree and runs under the same interpreter.
2. **Scoring runner.** The controller scores that file with `missing_tests_check.py --tests FILE`. The
   runner stages the file as `tests/unit/test_zz_missing_tests_experiment.py` in a copy, never in a frozen
   tree. It runs the tests on:
   - (a) the original's check copy, whose five regenerated files are hash-verified against the frozen copy;
   - (b) the unmodified candidate copy `.runs/e3-facdrone/mut`, hash-verified against `.runs/e3-facdrone/cand`;
   - (c) the same copy with each real T4 gap's mutant hunks applied, one at a time (T4-17, T4-21, T4-29).
     After each run the copy is restored and verified by sha256.

   Each run goes through `guard.py` and then `receipt_atomic.py`. Its deadline is `adaptive.scaled(600)`.
   Its receipt is `results/receipt-missing-tests.json`, and the per-test outcomes are in `results/logs/<label>.xml`.
3. **Discard.** After scoring, the controller deletes the test file and any copies of it. The runner
   removes the staged file after every run. The receipt keeps the file's sha256.

## Scoring rules (fixed now)

- **Valid tests.** A new test counts only if it passes on the original (a). The tests must pass on the
  original. Tests that fail there are reported and excluded from scoring.
- **Gap killed.** A real T4 gap counts as killed if, with its mutant applied in the isolated copy (c), at
  least one new test fails or errors. Attribution refinement, fixed before any test exists: that test
  must be valid and must also pass on the unmodified candidate (b). A test that already fails on the
  candidate cannot attribute a failure to the mutant, because the exception-path difference is
  present in every mutant copy.
- **Exception-path item exposed.** T5-capture (review H4) counts as exposed if at least one valid new test
  fails or errors on the unmodified candidate (b). The controller reads the failure message and checks
  that it concerns `capture_payload`/`capture.absences`. A failure caused by any other difference is
  reported as such and does not count.

## Prediction

All 3 real T4 gaps are killed (T4-17 `compact_capture`, T4-21 `money_evidence_payload`, T4-29
`_seal` strict validation), and the exception-path item T5-capture is exposed.

## Kill criterion

If fewer than 2 of the 3 gaps are killed, the report is not actionable as written. The result is
recorded as failed and the report is not revised to rescue it. Revising the report would start a new
preregistered check.

## Budget

- At most 8 pytest runs for the check, each through `guard.py` and `receipt_atomic.py`. One full scoring
  is 5 runs (original, candidate, three mutants), so at most one full scoring plus three single reruns
  fit. A rerun is allowed only for a timeout under load (factor > 1.5, rule 11.4) or for an apparatus
  fault recorded in the receipt, never because a test outcome was unwelcome.
- The writer gets one attempt and no feedback from any scoring run.
- The `dry-*` entries in the receipt come from the runner's mechanics check, which used a trivial test
  (`assert True`) before this check. They are apparatus validation, are not counted against the 8 runs,
  and are not evidence for the prediction.

## Frozen inputs (sha256)

Recorded on 2026-11-05 after the runner's dry run and before any test was written.

| Input | sha256 |
| --- | --- |
| `results/missing-tests.md` (what the writer gets) | `c2dfe54ad90e74299ccda1b4eaf5f6a0b744481b0396aa7c69df9cd9cd0c69d1` |
| `results/missing-tests.json` (the gaps the runner scores) | `80c658a4818f6bf7da96e977527806cda4d2263bcdfbe46b11d6bf1729eb94a1` |
| `results/t4-survivor-readings.json` | `90c05ea6d61a037f41aaf02d6b66b3f29ee4b1699be59d7329478743224cb1eb` |
| `missing_tests.py` | `14127ae5347fe4f37ce5dbf42a22f688103913650fee00102f2e805254f44655` |
| `missing_tests_check.py` | `86357fd0bb82baa641a8fbd6f20c778c224cb3345e26eabd6d250822140dbe87` |
| `results/t4-mutant-sample.json` | `12e49dbd28417d3ddb870edbb87e0a58db50040954d3e7ec9a733eb0172a273b` |
| original `decision_evidence_publication.py` | `81f3448c47c6b2fc0bb0b946a839a3b495ccf3be5879ce66853b224fe9d9eb41` |
| candidate `decision_evidence_publication.py` | `302204c6e3bd8f8f49968f63c56c4ecebeee1876202784a105a9839266e83da9` |

The runner records the test file's sha256 and its own sha256 in each `<label>-start` note.

## Result (2026-11-05, written by the controller after scoring; nothing above was changed)

- Test writer (per the controller; the receipt records only the file hash and path, not the writer's
  transcript): a fresh sonnet agent given only `missing-tests.md` and the original check copy. It
  wrote 17 tests (file sha256 `5bee7d0a732cc8a0a670563249ce2bc5573a28cb887b88a48a1abc2e1ef006e1`,
  recorded in the receipt note `mt-start`); all 17 pass on the original (`mt-orig`, exit 0, 18.3 s).
- Scoring (`missing_tests_check.py --tests ... --label mt`, 5 runs, receipt `receipt-missing-tests.json`,
  junit in `logs/mt-*.xml`; every copy hash-verified before and restored after):

| Run | Exit | Failures | What failed |
| --- | ---: | ---: | --- |
| `mt-orig` | 0 | 0 of 17 | nothing |
| `mt-cand` (unmodified candidate) | 1 | 3 | the three `test_t5_capture_unscored_failure_still_records_scoring_absence` cases: `AuditedAbsence('scoring', ...)` not in `[]` |
| `mt-m17` | 1 | 4 | `test_t4_17_compact_capture_keeps_reference_with_foreign_uri` plus the 3 candidate failures |
| `mt-m21` | 1 | 5 | both `test_t4_21_ledger_*` tests plus the 3 candidate failures |
| `mt-m29` | 1 | 11 | all 8 `test_t4_29_lax_typed_field_in_capture_is_refused` cases plus the 3 candidate failures |

- Gaps killed under the attribution rule (valid test, passes on the candidate, fails on the mutant):
  T4-17 by 1 test, T4-21 by 2, T4-29 by 8. **3 of 3.**
- Exception-path item T5-capture: **exposed** by 3 valid tests that fail only on the candidate; the
  failure message concerns `capture.absences` after `capture_payload` raises, as required.
- **Prediction held.** Kill criterion not fired. Budget: 5 of 8 runs used; no rerun; adaptive factors
  recorded per run in the receipt.
- Discard: the test file and the scratch directory were deleted after scoring; the runner removed every
  staged copy (verified: none under `.runs/e3-facdrone/*/tests/unit/`). The specimen gained no test.
- Reading: on this scope the missing-test report is actionable as written. One writer, one attempt,
  one scope; nothing about other reports or writers.
