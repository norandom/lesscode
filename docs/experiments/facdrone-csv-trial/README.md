# FCSV01: CSV projection compression is too marginal to adopt

The isolated Facdrone prototype removes **3 of 1,056 physical lines**. It removes
local construction scaffolding but does not remove a responsibility or enough
maintained code to justify a new field-selection helper. **Retain the original.**
This is a real before/after trial with a complete diff, not an unimplemented
baseline. Facdrone itself is unchanged.

The candidate uses ordered dictionary union for 15 row-construction sequences
and one typed helper for nine consecutive canonical fields. All 18 normalized
table projectors remain. Schemas, columns, units, financial calculations,
missing-data policy, joins, canonical conversion and sort order remain explicit.
Ruff's project formatter is applied before counting; apparent unformatted
line savings are not reported.

| Complete accounting scope | Before | Candidate |
| --- | ---: | ---: |
| Changed module physical lines | 1,056 | 1,053 |
| AST statements, including definitions/imports/docstrings | 369 | 341 |
| AST nodes | 5,560 | 5,408 |
| Functions | 42 | 43 |
| Reporting production Python subtree, 129 files, excluding testing | 42,296 | 42,293 |

The final counter uses decoded source `splitlines()` consistently; the initial
manual estimate of 1,057 lines/four removed was off by one. This table and the
machine result supersede that preliminary figure. AST statements are not LLOC.

## Same CSV evidence, but no full publication pass

Three existing report fixtures—small EUR, flagship EUR and USD—produce **54
identical rendered CSV artifacts containing 3,175 rows** before and after.
The actual renderer and its validators run; this is not a hand-reimplemented
serializer. A wrong-field mutation changes real output and is detected. A
targeted adverse check preserves rejection of an invalid unrelated dataclass
field: `_field` canonicalizes the whole object, so a `getattr` shortcut would
weaken its contract.

The paired renderer/adverse checker was added after the candidate was written;
it is exploratory follow-up evidence, not a preregistered oracle. Existing test
bodies predate the rewrite and are unchanged. The hypothesis's row-construction
amendment was added before rewriting but after the initial snapshot receipt.

The four selected existing test modules are byte-unchanged. Both copies report
**33 passed, 13 failed, zero skipped** with identical failing test identities.
All 13 failures reach the existing directory-fsync operation, which raises
PermissionError on this Windows host. The first setup attempts additionally
failed on missing dependencies; those outcomes remain in `.runs/facdrone-csv-trial/`.
No publisher fix, fsync bypass or weakened assertion was used to make a pass.

The focused suite uses `--noconftest`: Facdrone's global fixture plugin provisions
PostgreSQL and imports the full application. These selected tests use their own
in-memory fixtures and pytest's built-ins; no test body is changed. This is a
declared reduced reporting experiment, **not the repository's complete gate**.
No broker, production database, simulation or upstream deployment ran. The
directory-fsync failures remain an acceptance limit even though paired CSV
outputs agree. We reject the candidate on usefulness independently of that limit.

**No SMT solver ran.** Canonical dataclass conversion, Decimal encoding and
validation exceptions are outside the earlier arithmetic CEGIS model. An
arithmetic abstraction would not prove this exporter equivalent.

## What this teaches the selection stage

The exporter looked promising because it is large and repetitive. Much of that
repetition is the required output schema, not replaceable machinery. Temporary
assignments can disappear while field declarations and policy remain. That is
why fewer AST statements did not become a useful physical-code reduction.

A bounded read-only follow-up also inspected provider-series construction,
canonical serialization, configuration parsing and render-model conversion.
Several already use `dataclasses.asdict`, Pydantic or standard CSV operations.
Provider descriptors would retain most differing fields and introduce support
cost. These are deferred observations, not additional rewrite trials or proof
that Facdrone lacks larger opportunities. No forced deletion quota follows.

The next selection should identify a mechanism and its concrete replacement,
and estimate the whole replacement *before* paying for another environment or
general abstraction. The successful but modest [mvibe trial](../proxy-routing-trial/README.md)
is the positive comparison from this round.

## Evidence and reproduction

- [Pre-edit hypothesis](hypothesis.md), [snapshot hashes](snapshot.json)
- [Complete diff](candidate.patch), [whole-module/package counts and paired test identities](result.json)
- [All 54 artifact identities](oracle.json), [selected transformation sites](rewrite-selection.json)
- [Actual resolved environment](environment-final.txt)
- [Final source-preservation and test receipt](../proxy-routing-trial/verification.json)
- [Rewriter](rewrite.py), [paired renderer check](check.py), [test command](test.ps1), [measurement](measure.py)

Use uv 0.13.5 with CPython 3.13.16. The shared [snapshot copier](../proxy-routing-trial/prepare.py)
accepts specimen root, run directory and record directory. It copied 2,657
tracked working-tree files to before/after, recording revision and every hash.
Install the exact final environment into the separate experiment venv. The
two facdrone-data wheels come from the release URLs in the specimen lockfile;
their identities remain there. This is a reduced import/test environment,
not a claim that every Facdrone optional dependency is installed.

Run `test.ps1 before`, then the rewriter and Ruff 0.15.17 formatter, then
`test.ps1 after`. The script uses two workers, excludes `port_baseline` and
`live_broker`, and saves real exit codes. Set `OPENBLAS_NUM_THREADS=1` and
`OPENBLAS_CORETYPE=Haswell` as the existing reporting test guidance specifies;
these preserve numerical reproducibility, not performance measurements.
Run `check.py` through the same uv-selected interpreter, then `measure.py`.

Research scripts and fixture checks add separately disclosed apparatus; they
are not hidden in the production denominator. No tests or imported production
helpers are deleted. Only normalized_csv.py differs in the candidate copy.

## COBESY composition blueprint

Audience and adoption anchor: the owner and the agent choosing the next trial.
Target behavior: reject large-file candidates when a concrete full replacement
does not remove enough implementation. Working assumption: visible repetition
can be mistaken for removable responsibility; no diagnosis of author competence
or team culture follows. Disproof: changed rows, unavailable reasons, canonical
values, exception behavior, or hidden replacement support reject equivalence.

| Headline | Evidence | Reader action |
| --- | --- | --- |
| A large exporter did not yield a large reduction | 1,056 → 1,053 lines after formatting | Evaluate replacement cost before expanding |
| Real artifacts agreed within the tested boundary | 54 CSVs, 3,175 rows, mutation caught | Separate output evidence from full deployment gates |
| Structural improvement is not the goal by itself | Fewer temporary assignments, same mappings | Require useful maintained-code removal |
| Keep both positive and negative trials | mvibe's 23 lines, this marginal rejection | Improve selection without inventing an efficacy rate |

Lead future prose with the result and limits. Do not present these cases as an
SMT experiment, a random corpus sample, or measured maintenance-effort savings.
