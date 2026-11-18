# R151: Call counts are in lesscode-core for all four languages and equal the Python reference on E5 and E6

- Report date/time and timezone: 2026-11-18, 13:55, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 13:30 to 13:55 (from the clock and the file times)
- Status: completed
- Spec: [native-deconvolution-closure](../../../.kiro/specs/native-deconvolution-closure/requirements.md), the measurement part of requirement 2; [corrective task 2.2](../../../.kiro/corrective-tasks.md) and the owner decisions of 18 November
- Hypothesis ID and predecessor: ports the prototype of [R135](01-rust-scope-measurements.md); follows [R134](../2026-11-17/21-audit-of-port-and-reduction-claims.md)
- Type: preregistered against R135's frozen reference; no model call

## Decision

Call counting is part of the driver and is driven by the language packs. `deconvolute analyze` reports
`call_occurrences` for a file. The new `deconvolute measure-scope` reports, for a set of files, the
distinct function names, the call occurrences and the call edges. Both work for Python, TypeScript,
Java and Rust. No language is named in the code.

For Python the counts equal the frozen reference on every arm and control of R135, per file and as
whole edge lists:

| Arm | Functions | Edges | File pairs with an edge | Call occurrences |
| --- | ---: | ---: | ---: | ---: |
| E5 baseline | 69 | 89 | 12 | 62 |
| E5 candidate | 50 | 61 | 9 | 36 |
| E5 candidate, one call repeated | 50 | 61 | 9 | 37 |
| E5 candidate, one new call | 50 | 62 | 9 | 37 |
| E6 baseline | 102 | 145 | 10 | 90 |
| E6 candidate | 63 | 101 | 10 | 66 |

Every figure is the same in the reference. Corrective task 2.2 is ported.

## Hypothesis and falsification

R135's prototype counted call occurrences for Python only, through the Python module `measures.rs`, and
chose that path with a language test in the command handler. The hypothesis here: a rule that takes its
node kinds from the pack gives the same Python numbers. One differing file count, edge or total on the
six arms would have rejected it. The reference files, their input hashes and the two controls were
frozen by R135 before this work began.

## Frozen inputs and method

- Source: `fec7d8d`, built with Rust 1.98.1. Binary SHA-256 `01dc2dd8…7ad8c2`, kept in the run directory.
- Reference: `reference.json`, `*-python-scope.json` and `*-python-files.json` of R135's run directory.
  The comparison checks each input file's hash against `reference.json` before it counts.
- A pack gets a `calls` entry: `nodes` lists the call node kinds, `via` gives per node kind the field
  that leads on to the name. The text of the first node without a `via` entry is the name. Wrappers
  named in `unwrap_kinds` are skipped. A call of what another call returns has no name.
- An occurrence is a call whose name is the name of a function the same file declares; a call written
  twice counts twice. An edge is caller file, caller, callee file, callee, for a call inside a named
  function to a name declared in the scope, the caller's own name excepted. Edges are distinct.
- Not taken from the prototype patch: its changes to `measures.rs` and the command that printed the
  frozen `scope-measures-v1` report. That report carries the SQLAlchemy write vocabulary, which is a
  rule for one target. `measures.rs` is unchanged and still serves the regenerator.

```powershell
$run = 'D:\BuildArtifacts\lesscode\corrective-audit\task75\2.2'
pwsh -NoProfile -File "$run\compare.ps1" "$run\lesscode-core.exe" D:\BuildArtifacts\lesscode\corrective-audit\foreground-20261118 $run
```

The script runs `deconvolute measure-scope --lang python --root ROOT --file F...` once per arm and
`deconvolute analyze --lang python --source F` once per file. It exits 1 on any difference.

## Evidence and results

- Comparison: exit 0, no difference on six arms and 40 file measurements. `comparison.json` has the per-file rows,
  `cli.jsonl` every command, `*-scope.json` and `*-analyze.json` the receipts.
- Other languages: one test with a caller file and a second file per language checks the occurrence
  count, the edge list, a path outside the root and a file that does not parse
  (`calls_are_counted_per_file_and_as_edges_across_a_scope_for_four_languages`). These inputs are
  written for the test. There is no reference measurement for TypeScript, Java or Rust.
- Rust on real code: the 26 files of `native/src/adapters/driver` give 175 function names, 193
  occurrences and 521 edges (`self-rust-driver-scope.json`). Of the 24 edges out of `calls.rs`, 14 are
  calls that the source makes. Ten are matches by name only: `BTreeMap::get` against `store.rs get`,
  `Path::new` against `annotations.rs new`, `read` against three other files' `read`.
- Windows: fmt and clippy clean; nextest 0.9.143 with four threads, 402 of 402 passed; doctests pass;
  budgets 73 files, 66 within budget, 7 with a kept ceiling, none exceeded. A first full run with eight
  threads had one failure, `the_installed_hook_stops_a_push_without_a_matching_verdict` (a `git push`
  in a temporary repository failed); its binary passed when rerun alone and the test passed in the
  four-thread run. Both logs are kept.
- Linux: Dagger `evidence` at `fec7d8d` in `rust:1.98.1-bookworm`: every step exit 0, overall 0; nextest 401 of 401 passed, the new four-language test among them (`dagger-evidence/`).

[Run directory](D:/BuildArtifacts/lesscode/corrective-audit/task75/2.2)

## Limits and interpretation

- Matching is by name. A method call matches every function of that name in the scope, and library
  methods match local functions of the same name. The Rust self-measurement shows the size of this:
  ten of 24 edges. For Python on E5 and E6 the same rule is what the historical figures used, so the
  figures repeat; they are not a call graph.
- Positions decide whether a call lies inside a function. The reference used lines. The two agree on
  all six arms. They can differ where two functions share a line.
- Java counts `new T(...)` as a call of `T`, because constructors are function boundaries in that pack.
  TypeScript `new` is not counted.
- Receipts of `analyze` gain the field `call_occurrences`. Receipts written before today do not have it.
- `measures.rs` keeps its own Python edge rule for the regenerator, without the fix for a callee in
  parentheses that the prototype patch carried. Two rules for Python edges exist until that module is
  retired or moved onto the driver.

## Tool/spec consequence and next gate

The measurement part of requirement 2 has a command for every shipped language. Runtime calls, state
observation and the solver check are not touched by this. Next in the owner's order: the bounded Z3
check of task 2.3, then state-machine discovery (R140) and the Java lowering (R146, R147). The
Python-only `check_syntax` in `surface.rs` is still open; `tree::parse_clean`, added here, is the
grammar-based check it can use.

## COBESY check

The table comes first and each figure has a command. The disproof was one differing count. The
name-matching limit is measured on real Rust code and stated with its numbers. The other three
languages have tests on written inputs only, and the entry says so.
