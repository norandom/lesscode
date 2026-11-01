# FCSV01: Replace repeated field projection with named field selection

Recorded before candidate edits on 2026-11-01. Facdrone is an experimental
specimen; all mutations stay in Lesscode's isolated copy. No upstream edits,
commits, financial calculations, feature removal or Kiro changes.

The complete normalized CSV exporter is the responsibility:
`packages/facdrone-reporting/facdrone_reporting/library/normalized_csv.py`.
Its public bundle function projects all declared normalized tables; typed
projectors preserve schema, availability, provenance, joins, units and ordering.
All imported canonical encoding and table-schema code remains unchanged and
belongs to the wider accounting boundary.

Hypothesis: replace repeated contiguous same-record `_field(record, name)`
dictionary entries with named field selection through one small typed helper.
Reuse existing availability projection for repeated prefixed status triples.
Keep field names, output names, declaration order, joins, units and typed
projectors explicit. Preserve `_field`'s whole-object canonical validation,
including rejection of invalid unrelated fields. No direct `getattr` shortcut,
new dependencies, projection framework, descriptor classes or changed schemas.

Selection: manually ranked from a read-only review of grown modules. Alternative
provider-series factoring is deferred; keep the original if savings are marginal
or indirection makes the mapping less clear. Count the full module, new helpers
and unchanged wider dependencies, not selected functions alone. Tests and trial
apparatus are separate. Byte hashes establish identity, not compression benefit.

Verification plan: unchanged existing focused reporting tests where their
dependencies can run locally, plus paired complete normalized CSV outputs from
the same report fixtures, field-order/exception checks and a wrong-field mutation.
Pin local uv-managed Python and dependencies. No trading, real broker, operator
database or live network workloads. A focused pure-projection test run does not
establish the full publisher/deployment gate. Record exact exclusions.

SMT applicability: this is typed record projection and canonical serialization,
not the earlier arithmetic CEGIS domain. Do not label testing as SMT proof or
add a symbolic model that omits actual encoding/error semantics.

Budget: one normalized-export family and one candidate. Stop and retain any
counterexample; do not reduce checks to obtain a pass. Preserve snapshot source,
test, dependency and observation hashes, complete diff and empirical limitations.
Before editing: also test replacing local `values = base; values.update(fields);
records.append(values)` scaffolding with Python's ordered dictionary union at
the append site. Both operands are owned dictionaries. Retain evaluation order
and last-value-wins semantics; do not change any row mapping or loop boundary.
