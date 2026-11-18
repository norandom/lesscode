# R138 · Testing the E5 and E6 specimens against matched faults

18 November 2026, Europe/Berlin. Foreground experiment for [corrective 2.1](../../../.kiro/corrective-tasks.md) and the mutation-sensitivity portion of the [native closure requirements](../../../.kiro/specs/native-deconvolution-closure/requirements.md). [R134](../2026-11-17/21-audit-of-port-and-reduction-claims.md) found the earlier E5 and E6 adverse controls exercised examples in their evaluators. [R75](../2026-11-11/01-specimen-controls-rerun-through-driver.md) established the released-driver approach on other specimens.

## Result

Both versions of each specimen detected all four selected faults. Across the four arms, every one of the 16 mutated source lines executed in the worker copy. Each failure had a JUnit witness. There were zero survivors, invalid results, timeouts or reused outcomes in the completed runs. The clean opening and closing runs passed on every arm. The completed driver runs took place from 08:32:57 to 08:53:38.

| Trial and planted change | Baseline function and line | Candidate function and line | Detected on baseline / candidate |
| --- | --- | --- | --- |
| E5 · invert state-IC label selection | `monitor_label:50` | `monitor_label:48` | yes / yes |
| E5 · invert missing-baseline branch | `_baseline:74` | `signal_standing:93` | yes / yes |
| E5 · invert open-flag presence | `_flag:117` | `signal_standing:119` | yes / yes |
| E5 · invert standing-revision branch | `_state:130` | `signal_standing:84` | yes / yes |
| E6 · invert dictionary conversion | `as_canonical_json_value:39` | `as_canonical_json_value:30` | yes / yes |
| E6 · reject finite decimals in conversion | `_canonical_decimal:71` | `_canonicalize:63` | yes / yes |
| E6 · reject aware datetimes | `_canonical_datetime:77` | `_canonicalize:67` | yes / yes |
| E6 · reject finite decimals in encoding | `_decimal_token:165` | `_encode:105` | yes / yes |

E5's clean runs each had 109 passes and one skip. The replay completed 50 sequences and 317 operations with zero SQL differences on all four clean invocations. The changed monitor label produced the wrong display text. The inverted absence checks accessed a missing baseline or flag. E6's clean runs each had 56 passes and one skip. For example, the hydration round-trip test exposed a dictionary being rejected during conversion. Within each trial the two arms had identical collected test IDs, test-tree hashes and pack hashes. [Per-fault hashes, lines, failing tests and clean controls](D:/BuildArtifacts/lesscode/corrective-audit/task64/validated-results.json)

Selection was deliberate and limited to one changed file per trial. The driver left 60 and 65 other valid mutants untested in the E5 files, and 278 and 293 in E6. These results establish sensitivity to eight specified changes on both sides of the rewrite. Whole-specimen sensitivity and bounded equivalence remain open. The withdrawn historical controls retain their original status.

## Method

The hypothesis was that the existing acceptance suites would detect four specified branch inversions on both sides of each rewrite. At 08:24 the selection file pinned the operation, expected effect, source line, enclosing function and mutant hash for every arm, before running the batteries. Selection followed behavior across the rewrite, including branches that moved into another function. The panel covers lifecycle projection in E5 and canonical data conversion and encoding in E6.

The Windows `lesscode-core 2.0.0` executable matched its release archive member byte for byte. On each frozen input file, `deconvolute mutate` enumerated mutations from the shipped Python pack. The four configs use `deconvolute run --config ... --work ... --only ... --no-reuse --out ...`. One mutation worker ran two pytest-xdist workers. E5 used the historical seed list, exclusions and golden replay. E6 used the historical reporting, kernel and root suites. The harness put the specimen copy on the import path, checked the imported module, redirected replay output to the attempt directory and recorded JUnit results and executed lines from that copy. A passing mutant with a reached line would count as missed. A failure needed an actual failed specimen test.

| Arm | Selected mutant numbers | Source file |
| --- | --- | --- |
| E5 baseline | 8, 27, 57, 59 | `src/facdrone/shells/lifecycle_projection.py` |
| E5 candidate | 8, 42, 66, 37 | Same relative path |
| E6 baseline | 7, 35, 38, 99 | `packages/facdrone-kernel/facdrone_kernel/canonical.py` |
| E6 candidate | 7, 58, 64, 128 | Same relative path |

An interrupted E5 baseline attempt produced no final driver receipt. Its logs exposed a harness classification error. With `-x`, pytest-xdist returned 2 for an intentional stop after a recorded assertion failure. The corrected harness accepts this case only with failed JUnit cases, no JUnit errors and the explicit xdist fail-fast message. The driver recovered its journal and reran the entire panel on unchanged inputs. Only the completed command windows contribute results.

The [replay directory](D:/BuildArtifacts/lesscode/corrective-audit/task64) contains `method.md`, four configs, commands and receipts, selected source diffs, clean and mutant logs, execution traces and suite XML. `controls.json` records the selections. `verify.py` independently checks the driver classes against the attempt times, source hashes, line hits and test failures.
