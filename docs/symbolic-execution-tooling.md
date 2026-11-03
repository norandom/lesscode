# Symbolic execution and guided input generation: tool assessment

Reference note, written 2026-11-03 for the H8 research direction
([R19](research/2026-11-03/01-dynamic-deconvolution-direction.md)). It answers
the owner's question whether KLEE is an adequate way to obtain SAGE-style
symbolic-execution features in the deconvolution loop, and records which tool
supplies that feature per target language under the project's constraints:
native Windows execution on a plain user account, WSL reserved for CI, target
languages Python, TypeScript, JavaScript and Rust.

## The feature wanted

SAGE (Godefroid et al., Microsoft Research) established that instrumentation
plus fuzzing produces results only when input generation is constraint-guided:
symbolically execute a real seed trace, negate one branch constraint at a
time, solve for an input that flips it, rank by coverage gained. Blind
mutation did not reach deep paths. In this project the same mechanism serves
two purposes:

1. Raise the trace oracle's coverage of a scope by generating inputs and
   initial states that reach branches the seed tests do not.
2. Check an original against a regenerated candidate differentially, with a
   concrete counterexample when they disagree.

## KLEE: right reference, wrong tool here

KLEE is path-sensitive symbolic execution over LLVM bitcode with STP or Z3,
emitting one test per explored path. It is the whole-program form of the SAGE
mechanism and the correct conceptual reference.

| Concern | Assessment |
| --- | --- |
| Python, TypeScript, JavaScript | Not reachable. Interpreted code has no bitcode; executing CPython or V8 symbolically explores the interpreter, not the program. Translating a Python model to C and running KLEE tests the translation, not the source (already recorded in the spec's tool shortlist). |
| Rust | Reachable in principle: rustc emits LLVM bitcode, but the whole program including std must be built as bitcode and the environment stubbed. Panics, allocation and trait objects make this fragile; prior efforts are unmaintained. |
| Effects | KLEE's environment model covers POSIX files and stdin. The scopes under study talk to SQLite through C and to PostgreSQL over a socket. Symbolic execution into SQLite explodes; stubs reintroduce an unverifiable model, the problem H8 abandoned. |
| Host | Linux only. Under the project's runtime rule this is a CI rung at most. |

Decision: do not adopt KLEE. Obtain the feature per language from tools that
operate at the language or IR level the code actually has.

## Per-language instruments

| Need | Rust | Python | TypeScript / JavaScript |
| --- | --- | --- | --- |
| Constraint-driven inputs, branch flipping | Kani (bounded model checking on MIR, `kani::any()`; Linux/macOS, so CI only); cargo-fuzz natively on Windows as the coverage-guided rung; proptest as the random rung | CrossHair (concolic, Z3-backed symbolic Python values); Atheris or HypoFuzz as coverage-guided rungs; Hypothesis as the random rung | No mature concolic engine; Jazzer.js (libFuzzer for Node) coverage-guided; fast-check random. Recorded gap. |
| Differential check with counterexample | Kani harness feeding the same symbolic inputs to original and candidate; Miri for undefined behaviour in the candidate | CrossHair `diffbehavior`; Hypothesis ghostwriter `equivalent` as the mechanical baseline | Property tests only |
| Research-grade symbolic execution on IR | haybale, crux-mir | PyExZ3 (dated) | ExpoSE, Jalangi (dated) |
| Oracle strength | cargo-mutants | cosmic-ray (mutmut refuses Windows) | Stryker |
| Coverage | cargo-llvm-cov | coverage.py | c8 |

Availability was checked on 2026-11-03: cargo-mutants, cargo-llvm-cov,
cosmic-ray 8.7.0 and CrossHair run natively on Windows; mutmut, Kani, strace
and LD_PRELOAD do not.

## Where symbolic execution belongs in the loop

Not on the whole scope. The record-and-replay oracle
([E1a](experiments/deconvolution/e1-mmhmem-session-bus/plan.md),
[E1b](experiments/deconvolution/e1b-facdrone-evidence-publication/plan.md))
handles the effectful part: database state, files, effect ordering. Symbolic
execution pays off on the pure core of a scope, parsers, codecs and
state-transition functions, where it can establish that original and
candidate agree on all inputs within declared bounds rather than on the
recorded corpus only.

A regeneration that separates the pure core from the effect shell produces
exactly that split. E3 should therefore require it, and then CrossHair on
Python and Kani in CI on Rust supply the KLEE feature without KLEE. State is
part of the input: generators must mutate recorded initial states, not only
call arguments, or exploration stays on the corridor the seeds reach.

Practical consequence for E1b: its generated-sequence rung is Hypothesis, the
random tier. If the generated inputs find little beyond the seeds, the next
step is CrossHair over the publication module's pure functions, not a larger
random budget.

## Related records

- [R19, dynamic deconvolution direction](research/2026-11-03/01-dynamic-deconvolution-direction.md), amendments on guided generation, Rust mapping and host constraints.
- [Spec research protocol, tool shortlist](../.kiro/specs/constraint-guided-compression/research.md) including the earlier KLEE entry.
- [Research log](research/README.md).
