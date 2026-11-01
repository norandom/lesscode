# ATS01 fresh native results

The separately frozen `ATS01-fresh-native-6.2` candidate is **accepted**. The
known mutant is **rejected-behavior** for 2251 observed mismatches; its smaller
implementation does not override required behavior. Both reporting versions
passed the unchanged subclass dispatch check. Baseline and candidate each
matched all 8820 frozen oracle outcomes. All three arms executed fresh isolated
observations, including exact exceptions. No historical execution result was
admitted as a native check result.

| Native full-module production metric | Baseline | Candidate | Delta |
| --- | ---: | ---: | ---: |
| Physical lines v1 | 815 | 634 | -181 |
| Logical statements v1 | 345 | 184 | -161 |
| Decisions v1 | 87 | 58 | -29 |
| Functions v1 | 8 | 10 | +2 |
| Internal call layers v1 | unavailable | unavailable | unavailable |

Every metric uses the native Python tree-sitter 0.25.0 grammar and
lesscode-core 0.1.0 / ast-grep 0.51.0 definitions in the frozen record. Scope is
the complete `ats_agent.py` module in each source snapshot, including retained
agent members, all report tables and nested helpers. Whole-module scope and wider
boundary are equal; no adapters or callers were omitted. All counted source is
production. Research runner, tests, archived observations and evidence are outside
the source roots and do not contribute to production reduction. Dependencies
were not changed. The same native counts apply to the mutant; behavior rejects it.

The 310 baseline and 178 candidate unresolved calls remain recorded. Their
resolved internal-layer lower bounds are one each, which are not total depth or
comparable reductions. Explicit ownership/API reviews retain every located call.
Full-module counts do not imply full-module behavioral test coverage. The report
checks only finite rendering and dispatch contracts, without OpenAI or upstream
imports, hardware, throughput or universal-equivalence claims.

| Historical extracted-renderer label | Original | Candidate |
| --- | ---: | ---: |
| physical | 384 | 202 |
| lloc (Radon) | 298 | 135 |
| collector AST decisions | 76 | 47 |
| functions | 3 | 5 |
| ccn_sum (Lizard) | 76 | 51 |

Historical labels and their smaller extracted-renderer scope are preserved;
neither their values nor historical collector success are native definitions or
authenticated historical provenance. Fresh source identity, actual runner/check
definition, oracle and raw observation bindings appear in
[recorded/frozen.json](recorded/frozen.json),
[recorded/check-definition.json](recorded/check-definition.json),
[recorded/receipt.json](recorded/receipt.json), and the paired evidence files.
Original archive hashes are identical before and after this execution.

The new source target was frozen before fresh behavioral execution. It requires
a positive production statement reduction of at least one plus available source,
decision and function definitions. Call layers are contextual. Task 6.1's stricter
preparation target also required available call layers and remains incomplete;
this acceptance does not claim to meet it. Original preparation inputs are intact.

Machine and human reports:
[candidate JSON](recorded/candidate-comparison/result.json),
[candidate Markdown](recorded/candidate-comparison/report.md),
[mutant JSON](recorded/mutant-comparison/result.json),
[mutant Markdown](recorded/mutant-comparison/report.md).
Raw observations retain every case digest and the first eight mismatch witnesses:
[baseline](recorded/baseline-raw/raw.json),
[candidate](recorded/candidate-raw/raw.json),
[mutant](recorded/mutant-raw/raw.json).
