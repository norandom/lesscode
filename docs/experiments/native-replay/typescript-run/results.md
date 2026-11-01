# TS01 observed result

The native core accepted this isolated candidate against the pre-edit source
target. Keep the upstream baseline: adoption was outside this trial. The
production replacement removes 19 physical lines, 11 logical statements and
seven aggregate decisions without changing function count or introducing a
helper, adapter, table, wrapper, configuration or dependency.

| Native TypeScript metric v1 | Complete owned capability before → after | Wider production before → after |
| --- | --- | --- |
| Physical lines | 400 → 381 | 2,450 → 2,431 |
| Logical statements | 93 → 82 | 823 → 812 |
| Aggregate decisions | 16 → 9 | See complete machine report |
| Functions | 13 → 13 | See complete machine report |
| Internal call layers | Unavailable → unavailable; lower bound 2 both | Unavailable |

Owned scope is whole `src/degradation.ts` plus its imported `src/types.ts`,
including every notice constructor, summary operation, internal helper and useful
type. The sole source change replaces the terminal `slugify` implementation.
The [complete diff](recorded/candidate.patch) shows the custom loop replaced by
two regular-expression replacements and empty fallback. Names and exports remain
intact, with no new indirection. All other copied production source is identical;
the wider boundary detects exported implementation cost.

Five unresolved owned calls remain in each side's call-layer measurement;
reviewed standard Array/String/Date interactions are retained in the scope
annotations. Their full depth remains unavailable, never zero. The target was
explicitly frozen on four native source metrics before observations, with call
layers contextual. It does not reinterpret task 6.1's stricter target. Definitions
are `lesscode-core@0.1.0;ast-grep@0.51.0`,
`tree-sitter-typescript@0.23.2`, revision `v1`; these counts are not historical
Lizard/Radon scores. Configuration files have no TS source counts; a displayed
zero in that language/category is not a physical-line count of the configuration.

| Frozen check | Baseline | Candidate |
| --- | --- | --- |
| Existing strict project `tsc --noEmit` | Exit 0 | Exit 0 |
| Existing Node/tsx `tests/degradation.test.ts` | 3 passed, 0 failed/skipped | 3 passed, 0 failed/skipped |
| Public API differential corpus | 5,542 observations | 5,542 matched, 0 mismatches |

The added frozen corpus exercises unsupported-capability and permission-denied
identifiers over ASCII letters/digits/separators, whitespace, Unicode lowercase
expansion, combining characters, emoji, a lone surrogate and empty input. It
retains notice fields and exercises remaining notice constructors and all 16
notice subsets in enabled/disabled summaries. An independently adverse separator
mutant, changing the repeated-separator regexp to one-character replacement,
produced 4,158 differing observations. The raw
[negative-control receipt](recorded/mutant-raw/negative-control.json) records its
actual execution. The mutant is a sensitivity control, not a second candidate or
a native-admitted compression result.

Existing project tests remain unchanged: the wider copied test footprint is
2,847 physical lines / 1,067 statements on both sides; upstream script/research
support is 446 / 242 on both sides. The task's Python runner, JavaScript corpus,
two runner-boundary unit tests, archive and reports are additional research
apparatus outside specimen production. They are not credited as production
savings. No full asset/host/integration suite was run; existing strict project
types and the bounded unchanged capability suite were the declared checks.
Live opencode host behavior, actual permission execution, external services and
all possible runtime values remain unverified. Finite success is not universal
equivalence or evidence of lower maintenance effort.

Source revision is `8c70b37094e3e73938d5e2d08c8d98840137bf35` with clean upstream
status. The [receipt](recorded/receipt.json) has equal before/after hashes for all
copied source/test/config files and the native binary/runner identities. Installed
Node is 24.22.1, TypeScript 5.9.3 and tsx 4.22.4. The bound inventory records all
declared development dependencies and installed versions, source identity,
package/lock/config hashes and no proposed changes. No production library is
declared. No dependency sync, install, source edit or upstream adoption occurred.

Selection ran from 16:43:13 to no later than 16:44:41 local time (under the
30-minute budget). Final candidate editing/check execution ran 15:51:10–15:51:28
UTC on 2026-11-01; even including preparatory retries and the upstream evaluator
repair, the trial stayed well below two hours. The timing receipt honestly
identifies its retrospective transcription of actual tool timestamps.
From selection start to the final observation receipt, actual elapsed time was
8 minutes 14 seconds. Approximately four minutes (16:47–16:51 local) included
the diagnosed inventory-comparison blocker and parent-owned repair/review; that
time was included in the elapsed total rather than deducted from the budget.

The first attempted preparation omitted a transitive `.opencode/lib` source and
used a Windows path where Node requires a file URI; these were corrected before
final paired observations. A collector output-name collision was also corrected
and all final checks rerun. These are preparation failures, not behavior evidence.
The first corrected paired run then exposed a real native bug: comparison applied
the baseline-bound inventory identity to changed candidate source. Its exact
[error](recorded/first-inventory-blocker.log), [execution](recorded/first-inventory-blocker.json),
[freeze](recorded/first-freeze.json), [inventory](recorded/first-inventory.json) and
[partial paired receipt](recorded/first-paired-check-receipt.json) are retained.
No valid native result existed for that attempt. Parent-owned core fix `155f17b`
was independently reviewed; this final trial retained the supplied inventory,
froze fresh checks and reran observations against the repaired binary. Inventory
was never dropped or rebound to conceal the bug.

Denominators: eight discovered local manifests in a bounded manual survey; one
source project inventoried; one capability candidate evaluated; two concrete
alternatives (retain original/native strings); one validated candidate; zero
upstream adoptions. Preparatory retries and corpus cases are not independent
experiments. Adopted net production reduction is zero; isolated measured reduction
is 19 lines / 11 statements. This demonstrates a modest useful local replacement
and an exercised typed workflow, with no general success rate or broad maturity
claim. Windows-only native execution and finite check coverage remain explicit.
