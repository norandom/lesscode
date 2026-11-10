# R65: A Rust port of the driver's model-free core reproduces the prototype exactly on five scopes; it is 898 canonical lines against 753 it makes removable, and the apparatus cap as defined does not count either

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 19:10 to 19:45 local
- Status: completed
- Spec: [.kiro/specs/agnostic-annotation-driver/requirements.md](../../../.kiro/specs/agnostic-annotation-driver/requirements.md) (8.2, 8.4, 9.1, 9.3); first step of the design phase
- Hypothesis ID and predecessor: sizing spike after [R64](10-annotation-driver-gate-attempt2.md) and the [feasibility report](../../reports/agnostic-driver-feasibility.md)
- Type: exploratory (measurement; acceptance by parity fixed before the port)

## Decision

The port is feasible and bounded for the part measured: discovery of sites, all operators, annotation
validation, mutant classification and pack admission. A Rust version on the crate's pinned parser reproduces
the Python prototype site for site and mutant for mutant. This permits the design to rest on measured sizes.
It does not cover the batch runner, which was not ported.

Two statements made earlier today are corrected by this entry:

- The feasibility report estimated the engine and validator at 750 to 1,000 lines. Measured: 1,035 physical
  lines, 898 canonical. The estimate also left out pack admission, which a faithful port puts at 413
  canonical lines.
- The gate result (R64) and the report said the apparatus is "at its cap of 7,173 lines" and that the port
  therefore needs the cap raised or an equal removal. That was not checked against how the cap is counted.
  The cap's counted set is a fixed list of files (R41) and contains neither the adapters the driver replaces
  nor the deconvolution command code. Removing the adapters does not lower the counted figure, and whether the
  driver raises it depends on whether the owner places the driver in the counted set.

Separate outcomes: setup and model fidelity (parity) are shown for the model-free core. Behavioural evidence,
compression and adoption are not part of this entry.

## Hypothesis and falsification

Claim tested, from the feasibility report: a single-binary port is feasible within the existing dependency set
plus two grammars. Stated beforehand as refuting it: the crate's grammars reject node kinds the packs name;
the measured port exceeds the size the project can accept with nothing removable. The first did not happen.
The second is a matter of accounting that this entry lays out and the owner decides.

Acceptance fixed before the port: identical sites and identical mutants (compared by SHA-256 of the full
mutated text) on the five gate scopes, identical outcomes on all validator controls, identical pack admission
results, identical mutant classification.

## Frozen inputs and method

An isolated Cargo package, [native-spike](../../experiments/annotation-driver/native-spike/), outside the
native crate, with the native crate's pins: `ast-grep-core` and `ast-grep-language` 0.51.0 (grammars for Rust,
Java, Python and TypeScript switched on), `serde`, `serde_json`, `sha2`, Rust 1.98.1. Written by a delegated
Opus agent at the highest effort setting from a written specification; parity, formatting and lint were rerun
by the session agent. Nothing under `native/` was changed.

Ported: pack loading and the model-free admission checks, the site rules, eligible sites, all operators,
annotation validation, mutant classification, and the derivation of the control cases. Not ported: selection
(Python's random sequence is not reproducible), the batch runner and worker pool, model calls, and the two
admission checks that need a scope run.

Reference: the Python prototype at the hashes of gate Amendment 12, run on the pristine scope copies.
Command: `uv run python native-spike\parity.py --bin <spike binary>`; exit code 0.

## Evidence and results

### Parity

| Check | Prototype | Spike | Identical |
| --- | --- | --- | --- |
| Rust scope, sites / mutants | 71 / 66 | 71 / 66 | yes |
| Python scope | 211 / 199 | 211 / 199 | yes |
| TypeScript `state.ts` | 70 / 74 | 70 / 74 | yes |
| TypeScript `verification.ts` | 64 / 66 | 64 / 66 | yes |
| Java scope | 184 / 189 | 184 / 189 | yes |
| Validator controls | 98 of 98 as expected | 98 of 98 | every outcome string |
| Admission of four hand packs and two model-written packs | admitted | admitted | yes |
| Admission of four deliberately wrong packs | rejected by `kinds_valid`, `shape`, `size`, `templates` | same | yes |
| Mutant classification, three inputs on five scopes | `no_op`, `no_op`, `parse_error` | same | 15 of 15 |
| Full syntax tree of each scope file (every node's kind and range) | 3,141, 4,121, 2,346, 1,891 and 3,094 nodes | same | yes |

No grammar difference appeared between the parser versions (0.39.5 in the prototype's Python binding, 0.51.0
in the crate) on these files. No pack needed a change.

### Size

Canonical lines are non-blank lines that are not line comments, tests excluded.

| Part | Spike file | Physical | Canonical |
| --- | --- | --- | --- |
| Operators and eligible sites | `engine.rs` | 326 | 295 |
| Site rules and annotation validation | `validator.rs` | 319 | 289 |
| Tree helpers | `tree.rs` | 104 | 80 |
| Typed pack | `pack.rs` | 133 | 109 |
| Scope and file reading | `scope.rs` | 153 | 125 |
| **Core, the ground of the Python engine and validator** | | **1,035** | **898** |
| Pack admission, faithful to the prototype's untyped checks | `pack_check.rs` | 440 | 413 |
| Control-case runner (test tooling) | `controls.rs` | 338 | 311 |
| Command-line glue | `main.rs` | 179 | 157 |

The Python it corresponds to is 569 physical lines for the core. The Rust is about twice that in canonical
lines, mostly from typing the pack, writing out behaviours Python has built in, and lifetimes for syntax
nodes.

What the port makes removable in the native crate:

| Files | Physical | Canonical |
| --- | --- | --- |
| `adapters/{rust,java,typescript}/mutation.rs` | 449 | 401 |
| `adapters/{rust,java,typescript}/analysis.rs` | 420 | 352 |
| Total | 869 | 753 |

On a whole-crate view the core adds 898 and removes 753 canonical lines, a net of 145 more, and gives Python
both an analyzer and a generator. Pack admission adds up to 413 on top if ported as it stands; a design that
relies on the typed pack for the shape check would need much less, but that was not measured.

Against the project's per-file ceilings (400 physical lines, 80 per function): every core file fits. The
admission file is 440 lines and one function in the control runner is 100 lines; both would be split.

### Dependencies and build

- Added to the lock file relative to the native crate: `tree-sitter-java` 0.23.5 and `tree-sitter-rust` 0.24.2,
  both MIT. Nothing else.
- Clean release build of the spike: about 20 seconds with all four grammars and about the same with two; the
  difference is within noise on a shared machine.
- Binary: 5.7 MB with four grammars against 4.2 MB with two, about 1.5 MB more.

### How the apparatus cap is counted

R41 fixed the cap at 7,173 canonical lines as "exactly the measured total" of a named set: `regenerator.rs`,
`measures.rs`, `surface.rs`, `recording.rs`, `jsonval.rs`, `admission.rs`, `receipt.rs`, the recording shim,
the registration hooks and `state_contract.rs`
([R41](../2026-11-08/08-state-boundary-ceiling-amendment.md), counted-set table). The language adapters added
afterwards and `cli_deconvolute.rs` are not in that table, and R53 reported the cap as held with them present.
All Rust sources of the crate together are 15,212 canonical lines today.

So under the cap as it has been applied, the adapters were outside it, and their replacement would be outside
it too; the per-file budgets are what govern them. Under a stricter reading, in which every added path
counts, the driver adds its whole size to a figure that is already at the cap and nothing it removes was ever
in the count.

## Limits and interpretation

- Parity was shown on five files. It says the port is faithful there, not that the rules are right on other
  code; that limit is the prototype's.
- The spike is a faithful port, not a design. It reproduces Python behaviours a native design would not need
  (for example the exact wording of admission details), so its size is an upper figure for those parts.
- The batch runner, the worker pool, recovery after an interrupted run and outcome reuse (Requirements 4 and
  5) were not ported or sized. In the prototype they are about 600 lines of Python. The native crate already
  has process supervision code that may carry part of it; that is unexamined.
- The spike accepts only the four compiled-in grammars. A pack for another language cannot be admitted by the
  binary without adding that grammar to the build.
- One grammar convention is still a literal in the library (the word "attribute" in the test-region rule); it
  belongs in the pack.
- The statement about the cap rests on reading R41's table and R53's claim. Whether the adapters were meant to
  be counted was not confirmed with the owner before this entry.

## Tool/spec consequence and next gate

- Requirement 8.2 has its measurement for the core and for admission. Requirement 9.1 and 9.3 are met by the
  spike for sites, mutants and controls; the native driver must keep meeting them.
- The grammar-version risk named in the feasibility report did not materialise on these scopes.
- For the owner: decide how the driver is accounted. Either it stays outside the counted set, as the adapters
  it replaces were, and is governed by the per-file budgets; or it enters the set and the cap is amended by
  its measured size.
- Next in the design: the batch runner, with reuse of existing supervision code examined and sized, and
  recovery after interruption. Slice B keeps its place in the queue (roadmap, "Work queue").

## COBESY check

Outcome first: exact parity, measured sizes, and a correction of two earlier statements. Replay: package,
command, pins and reference hashes given. Disproof: the two refuting conditions are named; one did not occur
and one is laid out for decision. The agent's own unchecked claim about the cap is corrected in the decision
section. Facts are in the tables; the two readings of the cap are presented as readings.
