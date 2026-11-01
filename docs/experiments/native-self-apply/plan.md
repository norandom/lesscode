# NS01: reuse the existing policy primitive validators

Task 7.1 preparation only. No candidate implementation or adoption has occurred.
The workflow maturity gate remains open until task 7.2 supplies paired behavior,
adverse, regression and whole-replacement evidence. Requirements 5.1, 5.2, 5.3
and 5.8; design System Flows (self-application maturity review).

## Selection and deletion hypothesis

One manually selected efficacy candidate: the complete policy primitive input
validation responsibility in `src/lesscode_baseline/rules/contracts.py`, backed by
the existing `src/lesscode_baseline/contracts.py` implementation. Reuse the
already-present `_fail`, `_object`, `_string`, `_choice`, `_hash` and `_sequence`
operations instead of maintaining their second implementations. Keep policy
number, relative-path and UTC validation: their accepted domains and diagnostics
differ. Keep every record, field, public reader and safeguard.

The six duplicate policy functions contain 36 CPython AST statements, including
function definitions, before any candidate edit. This is an opportunity estimate,
not a native result or a promised saving. Reuse imports and necessary compatibility
wrappers count against savings. `_object(fields=...)` and `_fail(reason=...)`
have different keyword parameter names from the baseline helpers; preserve their
accepted calls with small wrappers where required. Do not claim annotation or
private-call compatibility merely from matching positional examples. No new
framework, dependency, upgrade or generalized validation engine is authorized.

The complete accounting boundary owns both whole contracts modules, so the shared
implementation is counted rather than treated as free. Unchanged snapshot glob
matching is a retained dependency of policy-scope validation; existing callers,
their modules, package data and configuration remain in the wider project snapshot.
Any changed caller or new helper must expand the candidate ownership map and be
counted. Whole-project totals detect exported implementation. Tests and research
are separate categories. None is a deletion target.

Alternatives are retain the original or reuse only genuinely compatible members.
Any partial reuse must still preserve and measure the complete responsibility.
The small atomic-write sharing idea was not selected: only a single-digit net
statement saving seemed credible after ownership safeguards. Earlier negative
glob, Pydantic leaf-record, report-graph and feature-path experiments remain
unchanged. This candidate uses an existing implementation of the same validator
family, rather than retesting those mechanisms.

## Frozen target and limits

Before any candidate edit, require at least **25 native production logical
statements removed** across the owned modules, including all added imports,
wrappers, adapters and changed callers. Freeze absolute maximum counts for
physical lines, decisions and functions at the measured native baseline values;
none may increase. The machine manifest records those numeric limits and exact
metric definitions. Internal call layers remain optional context because dynamic
Python dispatch prevents complete resolution. Keep unresolved relationships
visible and review them before admitting candidate evidence. A source-count win
alone cannot certify behavior, idiom or maintenance savings.

Selection/preparation is bounded to 1–3 hours; task 7.2 gets one bounded candidate
attempt and may reject it. Stop if compatibility wrappers consume useful savings,
if required behavior cannot be checked, or if the target is missed. Retain the
baseline and report rejection or incomplete evidence. There is no deletion quota,
feature cut, native Rust parsing requirement or automatic adoption.

## Checks fixed before editing

`check-definition.json` fixes the commands, interpreter, unchanged test-file
inventory and independently derived primitive outcome obligations. Both sides
must pass the same selected suites and **the existing project regression suite**:

```text
uv.exe run --no-project --no-python-downloads --python D:/Source/lesscode/.venv-task11/Scripts/python.exe python -m unittest tests.test_contracts tests.test_rule_contracts tests.test_rule_evidence tests.test_rule_evaluation -q
uv.exe run --no-project --no-python-downloads --python D:/Source/lesscode/.venv-task11/Scripts/python.exe python -m unittest discover -q
```

Task 7.1 probes the pinned interpreter and selected existing tests only; it does
not run the lengthy regression suite or count the old 515-test result as fresh
paired evidence. Task 7.2 uses external check execution trees built from frozen
tests/resources and the chosen side's complete `src` tree. Run with cwd at that
tree and `PYTHONPATH=<execution-tree>/src`; verify the imported package path.
This ensures tests that derive ROOT from `__file__`, spawn subprocesses, or build
wheels exercise the chosen implementation. Copy the already-installed analyzer
tree into each execution tree as regular files; retain its external identities.
Do not use junctions, alter tests, silently skip unavailable analyzer checks, or
change source under the tested identities. Before/after hashes must match the
external frozen check inventory. Execution artifacts and bytecode stay outside
source accounting. If check definitions need changes, start a new frozen trial.

## Independent expected outcomes and adverse case

Task 7.2 must add and freeze an external expected-result corpus before editing
the candidate. Derive its answers by hand from the declared contracts and the
frozen `expected-outcomes.json` obligations, independently of either evaluator's
verdict. A reviewer checks acceptance, values, exact ContractError diagnostics,
error precedence, duplicate positions and keyword call shapes. The corpus must
exercise object missing/unknown fields and order; empty/whitespace/NUL and
non-string inputs; choices; lowercase 64-character SHA-256 versus malformed,
uppercase and non-string values; lists and tuples versus strings/sets/mappings;
empty sequences, nested paths, reader-call order and duplicate identity positions;
bool/int distinctions where numeric readers are composed; unchanged finite
nonnegative-number, root/path and UTC policy boundaries. Compare typed public
records and serialized output, not only helper truth values. Preserve meaningful
signatures, annotations, exception classes and existing diagnostics.

Known adverse control: a mutant that removes `_sequence`'s duplicate-identity
check must fail a hand-authored repeated-identity case with the expected diagnostic
at index 1. Also include malformed fingerprint and unknown-field controls. Run
these controls against the independent checks; no candidate verdict may certify
its own correctness. If these controls are not detected, behavior evidence is
incomplete. Differential baseline/candidate agreement is supplementary evidence.
The selected corpus does not prove equivalence for all Python objects or hostile
custom subclasses; disclose finite-domain and platform limits.

## Provenance, isolation and coverage

`receipt.json` binds the actual dirty working tree, source inventory, HEAD,
evaluator executable SHA-256, evaluator source/lock hashes, checks and existing
Python/tool environments. Baseline and candidate start byte-identical under
`.runs/native-self-apply/ns01/`; the native executable and frozen checks live in
the sibling `external/` directory. `baseline.zip` preserves the exact source
snapshot independently of later repository changes. Runtime tool state is
fingerprinted separately and excluded from source benefits. Hashes detect
changes; they are not hostile-writer isolation or signatures.

Source edits already present in `rules/evaluate.py` and
`tests/test_rule_evaluation.py` are copied as-is and remain untouched. Their
hashes are explicit in the receipt. Snapshot identity is not HEAD identity.
External ATS/NPU/TypeScript accepted/adverse records establish the prior workflow
exercise; their hashes are retained as prerequisite references.

Only the selected Python reference capability and native Python source counts
are exercised. Rust production/tests, parser implementation, experiment scripts,
generated/tool state and unselected capabilities are excluded from self-compression
claims. Native production/tests footprint belongs in task 7.3's separate report.
This preparation does not establish whole-project maturity, cross-platform
behavior, accepted compression, adoption or reduced maintenance effort.
