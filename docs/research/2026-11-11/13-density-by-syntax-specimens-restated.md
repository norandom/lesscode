# R87: Measured by syntax, the four specimen candidates are 6% to 19% smaller, not 24% to 44%

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 11:20 to 11:45 local
- Status: completed
- Spec: [agnostic-annotation-driver](../../../.kiro/specs/agnostic-annotation-driver/design.md), design
  Amendment 6
- Hypothesis ID and predecessor: follows [R86](12-rust-candidate-built-and-measured.md); restates the
  compression figures of [R50](../2026-11-09/08-typescript-adapter-prototype-gate.md),
  [R51](../2026-11-09/09-java-adapter-prototype-gate.md),
  [R52](../2026-11-09/10-rust-adapter-prototype-gate.md) and
  [R53](../2026-11-09/11-multi-language-deconvolution-synthesis.md)
- Type: retrospective re-measurement with a corrected instrument

## Decision

The compression results this project published for its four specimens do not hold as substantial
reductions. Measured by syntax, which layout cannot change, the candidates are smaller by 5.6% (Java), 7.0%
(Rust), 8.7% (Python) and 18.8% (TypeScript) in tokens. The published figures were 38.5%, 40.3%, 23.8% and
36.8% in lines (R53). None of the four reaches the threshold for a substantial reduction that was fixed
before this measurement. Most of what was reported as compression was denser layout: more on each line.

`deconvolute density` now measures syntax. Its earlier form counted lines and, under the name of statements,
keywords and semicolons per line.

Separate outcomes: this restates compression only. That the Java and TypeScript candidates pass their
projects' tests (R50, R51) and that the Rust candidate passes this crate's tests (R86) is unchanged. The
adoption decisions of the earlier experiments are the owner's to revisit.

## Hypothesis and falsification

After R86 the expectation, stated to the owner before the change, was that the other specimens' figures
"may shrink the same way". The thresholds of the new verdict were fixed in design Amendment 6 (commit
`70e9e24`) before any specimen was measured with the new command: at least 20% fewer tokens, at least 400
tokens removed, at least 10% fewer nodes. A specimen that met them would have kept its "pass". None does.

## Frozen inputs and method

- Command: `deconvolute density` at the working tree after `70e9e24` (receipt `deconvolution-density-v2`).
  Tokens are the leaves of the syntax tree outside comments; nodes are the named syntax nodes outside
  comments; lines are the lines on which a token starts.
- Pairs: Rust `native/adapters/rust/baseline_lifecycle.rs` and `candidate_lifecycle.rs`; Java
  `CSVPrinter.java` of commons-csv at the gate revision and `native/adapters/java/candidate_CSVPrinter.java`;
  TypeScript `src/degradation.ts` of OpenUltraCode at the gate revision (hash as in R50) and
  `native/adapters/typescript/candidate_degradation.ts`; Python the seven files of the lifecycle experiment
  and the six of the report-envelope experiment, baseline `.runs/e2-facdrone/source`, candidates
  `.runs/e5-state-boundary/cand-e5` and `.runs/e6-report-envelope/cand-e6` (the files that differ between
  the trees are exactly those thirteen).
- Receipts: `docs/experiments/annotation-driver/results/density-v2/`. The Python trees are local and not in
  the repository; their per-file sizes are in the receipts.

## Evidence and results

| Specimen | Tokens | Nodes | Lines with code | Verdict now | Reported before |
| --- | --- | --- | --- | --- | --- |
| Rust lifecycle | 1,846 to 1,716, 7.0% | 1,460 to 1,350, 7.5% | 249 to 149, 40.2% | marginal | 40.3% lines, pass |
| Java `CSVPrinter` | 1,420 to 1,341, 5.6% | 1,212 to 1,159, 4.4% | 221 to 136, 38.5% | marginal | 38.5% lines, pass |
| TypeScript `degradation` | 1,283 to 1,042, 18.8% | 1,064 to 842, 20.9% | 201 to 113, 43.8% | marginal | 36.8% lines, pass |
| Python lifecycle, 7 files | 11,241 to 10,400, 7.5% | 11,076 to 10,161, 8.3% | 1,220 to 948, 22.3% | marginal | 23.8% lines for both sets |
| Python report envelope, 6 files | 11,132 to 10,036, 9.8% | 10,885 to 9,809, 9.9% | 1,612 to 1,315, 18.4% | marginal | |

- The line counts of this measurement reproduce the earlier ones closely (Java 221 to 136 and Rust 249 to
  149 against 221 to 136 and 248 to 148), so the candidates and originals are the ones measured then; what
  changed is the unit.
- TypeScript is the one specimen with a reduction near the threshold in relative terms (18.8% of tokens,
  20.9% of nodes); it removes 241 tokens, below the floor of 400.
- The Python sets remove the most in absolute terms, 841 and 1,096 tokens, at 7.5% and 9.8%.

## Limits and interpretation

- Tokens and nodes are one way to measure syntax. They agree with each other within two points on every
  specimen. A measure that weighted tokens differently would move the figures somewhat, not from 7% to 40%.
- The floor of 400 tokens was derived from the earlier floor of 50 lines at eight tokens a line. It decides
  only the TypeScript verdict, which fails the relative threshold as well.
- A smaller reduction is not a worthless one: fewer tokens for the same behaviour is still less code. What
  is withdrawn is the size of the claim.
- The earlier experiments before R49 (E3 to E6 on Facdrone) reported line-based savings with the Python
  prototype's own counters; only the two Python sets above were re-measured.

## Tool/spec consequence and next gate

- `deconvolute density` counts syntax for Rust, Python, TypeScript and Java (design Amendment 6). The README
  example shows the Rust pair as `marginal`.
- R50 to R53 get a dated amendment each; the README and the architecture page name the new figures.
- Still in the library, unused by the command, and the owner's to decide: the line and keyword counters in
  `adapters/<lang>/metrics.rs` and `density.rs`, and the tests that pin 248 to 148 lines as "QUALIFIED".
- For the owner: the project's headline claim was "substantial simplification while preserving features".
  The specimens show small reductions with behaviour preserved under the tests there are. Whether the
  regeneration method can produce reductions that are substantial by syntax is open again, and is the
  question the next experiment should ask with this instrument from the start.

## COBESY check

Outcome first: the published compression figures are restated downwards for all four languages, with the
thresholds fixed beforehand. Replay: command, pairs and receipts named; the Python trees are local. Disproof:
any specimen could have kept its verdict and none did. What is not withdrawn (behaviour under tests) and
what was not re-measured (earlier Python experiments) is stated.
