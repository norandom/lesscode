# R152: The bounded solver check is a command of lesscode-core and repeats R139 on E5

- Report date/time and timezone: 2026-11-18, 14:20, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 14:00 to 14:17 (from the clock and the file times); first complete run 14:06, final binary 14:16
- Status: completed
- Spec: [state-consolidation](../../../.kiro/specs/state-consolidation/brief.md), prototype runtime; [corrective task 2.3](../../../.kiro/corrective-tasks.md) and the owner decisions of 18 November
- Hypothesis ID and predecessor: ports the prototype of [R139](05-e5-rust-z3-check.md); follows [R151](17-pack-driven-call-counts.md)
- Type: preregistered against R139's kept result; no model call

## Decision

`lesscode-core deconvolute bound-check --target T --out D` compares versions of one behaviour inside
stated ranges. The core holds the general part: the target file, the queries, the pinned solver, the
run of every case through each version's command, the witness replay and the verdict. The E5 row
filter stays in the experiment, which lowers the three guards from source and writes the target file.

On E5 the command gives what R139 gave:

| Pair | Solver | Cases where expressions differ | Cases where reports differ | Witness | Verdict |
| --- | --- | ---: | ---: | --- | --- |
| Baseline, E5 candidate | unsat | 0 of 81 | 0 of 81 | none | `same` |
| Baseline, candidate with the planted revision error | sat | 18 of 81 | 18 of 81 | case 9, reproduced | `different` |

The 18 case numbers are the 18 of R139's `result.json`, and the witness is the same case. Corrective
task 2.3 is ported. `pure_verifier.rs` can go with task 3.1.

## Hypothesis and falsification

The prototype held the target's knowledge and the general procedure in one program. The hypothesis: the
procedure works from a description alone, and then repeats R139's result on the same inputs. Rejected
by another verdict, another set of differing cases, a witness that does not reproduce, or a window
list that disagrees with a lowered guard.

A second claim is new. The command reports a pair as `inconsistent` when the expressions and the real
runs disagree about where the versions differ. Rejected if a model that misses the planted fault still
gets `same`.

## Frozen inputs and method

- Source `c1c76e8`, Rust 1.98.1. Binary SHA-256 `d8d51d80…fe34ab2`, kept in the run directory.
- Inputs as in R139: the E2 and E5 trees, CPython 3.13.16 of the specimen, Z3 5.3.3. The target file
  pins the two subject files, the kernel module in both trees, the interpreter and the solver by
  SHA-256; the command checks them before anything runs. The adapter `invoke.py` is unchanged.
- Target file: four symbols from 0 to 2, three arms with the lowered expression and the adapter's
  command line, two pairs, and a case template in which `d{old_digest}` and the like take the values.
- Per run: one solver call evaluates all arms on all 81 cases; each arm's command runs once on the 81
  cases; per pair one solver call gives the verdict; on `sat` one more gives values, and the two arms
  run again on that one case in new processes.
- A pair is `same` when the solver says unsat and no report differs. It is `different` when the solver
  says sat, the cases where reports differ are the cases where expressions differ, and the witness
  differs again when run alone. Anything else is `inconsistent`.

```powershell
$ev = 'D:\BuildArtifacts\lesscode\corrective-audit\task75\2.3'
$m = 'docs/experiments/state-consolidation/e5-bound/Cargo.toml'
cargo +1.98.1 run --offline --locked --manifest-path $m --bin e5-bound -- --target . "$ev\target-new"
& "$ev\lesscode-core.exe" deconvolute bound-check --target "$ev\target-new\target.json" --out "$ev\run-new"
cargo +1.98.1 run --offline --locked --manifest-path $m --bin e5-bound -- --windows . "$ev\target-new" "$ev\run-new"
```

The second command exits 1, because the planted pair differs.

## Evidence and results

- E5 run: exit 1, table above. `run/result.json`; every query, answer and command line is beside it.
  Repeated with the final binary (`run-final/`): the receipt is identical in bytes.
- Window lists: all 243 lists that the three arms reported are the ones their lowered guards give
  (`windows.txt`). This check is the experiment's, since it reads E5's report format.
- Control, the model misses the fault: the planted arm was given the candidate's expression. The solver
  said unsat, 18 reports differed, all 18 cases were listed as unexplained, verdict `inconsistent`,
  exit 2 (`control-model-misses-fault/`).
- Control, a wrong pin: exit 3 before the solver or any command ran (`control-wrong-pin.txt`).
- Tests: a target with a repeated name, an unknown arm, an empty or too large range, or an unknown key
  is refused; solver answers bind each asked name exactly once. Windows: fmt and clippy clean, nextest
  409 of 409, doctests pass, budgets 76 files with none exceeded.
- Linux: Dagger `evidence` at `c1c76e8`: every step exit 0, overall 0; nextest 408 of 408 passed, the refusal test among them (`dagger-evidence/`). No solver ran there.
- Along the way: the first layout put the target module in a subdirectory. The test
  `the_engine_names_no_language_or_project` failed on it, because it reads each entry of the driver
  directory as a file. The module is now a file beside the others, so that test scans it. The first
  complete E5 run, before that change, gave the same receipt (`first-run-before-split/`).

[Run directory](D:/BuildArtifacts/lesscode/corrective-audit/task75/2.3)

## Limits and interpretation

- `same` means no difference inside the ranges. It is not a proof.
- The expressions are not derived by the core. For E5 the experiment lowers two pinned filter shapes
  and refuses others, as in R139. A wrong expression is caught only where the real runs show it, which
  is what the first control exercises.
- No test in the suite calls a solver, since none is installed in CI. The solver path ran on E5 only.
- Symbols are integers with ranges and the logic is `QF_LIA`. Histories of events, as in R140 and
  R146, need symbols per step; the solver module is written to be shared with those ports.
- The command runs the target file's commands as written and copies no source tree. The E5 adapter
  only reads.
- Domain limits of R139 hold unchanged: two readings, within boundary, standing unrecorded.

## Tool/spec consequence and next gate

The bounded comparison that R148 found missing in the driver exists as a command, for any language
whose behaviour someone can state as an expression and run by a command. Next in the owner's order:
state-machine discovery (R140), then the Java lowering (R146, R147). Both produce expressions from
source, which is the part this port leaves to the target.

## COBESY check

The table and the two controls come first with their exit codes. The disproof conditions are named
and one of them is run on purpose. What the core does not do, deriving the expressions, is stated in
the decision and again in the limits.
