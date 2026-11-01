# Recorded NPU replay results

The whole-provider replacement reduces native logical statements from 264 to 246
and physical lines from 799 to 729, including the entire new typed helper. The
separate task 6.3 target requires a positive production statement reduction and
comparable source metric definitions. Internal call layers remain unavailable.
Task 6.1's stricter all-metric preparation target remains incomplete; task 6.3
neither repairs nor replaces that historical declaration.

| Native whole provider/helper metric | Baseline | Candidate | Delta |
| --- | ---: | ---: | ---: |
| physical_lines.v1 | 799 | 729 | -70 |
| logical_statements.v1 | 264 | 246 | -18 |
| decisions.v1 | 54 | 46 | -8 |
| functions.v1 | 36 | 34 | -2 |
| internal_call_layers.v1 | unavailable | unavailable | unavailable |

Native definitions are tree-sitter-python@0.25.0 with
lesscode-core@0.1.0;ast-grep@0.51.0. Selected scope identities and exact inventories
are retained in the native reports. Wider copied source and test accounting
reports physical lines 24,319 to 24,249; logical statements 9,388 to 9,370;
decisions 1,918 to 1,910; functions 1,108 to 1,106. Its unknown call layers remain
unavailable. Category totals distinguish production, tests and dependencies.
No dependency/configuration integration changes support the replacement.

The archived NPU01 selected execution/check-family scope used a different boundary
and analyzers: physical lines 171 to 101, Radon LLOC 68 to 47, AST decisions 16 to 8,
and functions 8 to 6. These are historical selected-family figures, not native
whole-module counts. The archived whole modules had the same physical lines
799 to 729, but their Radon LLOC 290 to 269 is not native logical_statements.v1.
The native report does not rename or substitute those definitions.

A first adapter run was incomplete despite passing software checks: four
candidate relationship reviews named the added helper path rather than its
baseline ownership key. [first-incomplete.json](first-incomplete.json) preserves
the exact reasons, source identities and check definition. The adapter now maps
that helper to the CPU ownership key whose replacement explicitly includes it.
A new trial freezes the corrected runner and repeats observations. No core or
upstream product change was needed.

Raw per-case digests and limited exact mismatch witnesses preserve the finite
observations. Every check result binds the actual source identity, frozen trial,
check definition and raw artifact hashes. A collector's zero exit is not used as
differential success. Hardware execution, accelerator validation and performance
claims are outside the evidence domain. Hashes establish consistency, not
externally authenticated execution or universal behavioral equivalence.

| Fresh observation | Baseline | Candidate | Dtype mutant |
| --- | --- | --- | --- |
| Frozen differential corpus | 3,168 / 0 mismatches | 3,168 / 0 mismatches | 3,168 / 572 mismatches |
| Unchanged software tests | 339 passed | 339 passed | not run |
| Selected mypy modules | 2 passed | 3 passed | not run |
| Shared-safety audit | passed | passed | not run |
| Native verdict | eligible | accepted | rejected-behavior |

The mutant report retains all three missing candidate checks alongside the known
behavior failure. Missing evidence cannot turn that failure into acceptance.

| Integrated manipulation | Native outcome | Exact admission reason |
| --- | --- | --- |
| Move helper into `support/moved_helper.py` | invalid, exit 3 | declared `checked_run.py` replacement source missing or excluded |
| Move production helper into undeclared test path | invalid, exit 3 | declared `checked_run.py` replacement source missing or excluded |
| Clear ownership replacements | incomplete | candidate owned implementation omitted |
| Remove required candidate type result | incomplete | `types/candidate`: required check result missing |
| Bind candidate differential to baseline identity | invalid, exit 3 | `differential/candidate`: source identity mismatch |

The helper/test-path variants physically move the source while retaining the
frozen categories and replacement declaration. Native admission rejects those
attempts before report generation; exact command, exit code, stdout and stderr
are retained instead of fabricated comparison results. Successful relocation
would require declaring and counting the replacement under a newly valid scope.
The fixtures deliberately test attempted omission/reclassification, not runnable
adoption of these altered copies.
