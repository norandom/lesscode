# R85: Three annotation runs select more stably than one, but not stably enough to call the selection fixed

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 10:27 to 10:32 local (times from the run's output)
- Status: completed
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: runs [R84](10-slice-b-annotation-hardening-preregistration.md)
- Type: confirmatory experiment on one model

## Decision

Both rules help and neither closes the gap. Keeping the sites that at least two of three runs name gives
selections that agree with each other more than single runs do in four of five files, and the selection is
no larger than a single run's. But the level reached is 0.74 to 0.96, and the target of 0.80 in four files is
missed. Taking the union of three runs is also more stable in four of five files and keeps 10% to 60% more
sites. Either rule costs three calls per file instead of one.

Recommendation, for the owner: if annotations are used at all, use the majority of three. It keeps the
selection as small as one run's and is the more stable rule where files have many candidate sites. Do not
describe a selection made this way as reproducible; describe it as "what two of three runs agreed on". The
default without annotations, every eligible site, remains the only selection that does not vary.

Separate outcomes: stability of a selection. Whether the selected sites are the important ones was not
measured.

## Hypothesis and falsification

| Claim of R84 | Result | Verdict |
| --- | --- | --- |
| H1: majority selections agree more than single runs, in at least four of five scopes | four (equal in the fifth) | holds |
| H2: majority agreement at least 0.80 in at least four of five scopes | three | refuted |
| H3: union selections agree more than single runs, in at least four of five scopes | four (lower in the fifth) | holds |

## Frozen inputs and method

As R84: six fresh runs per scope on `claude-opus-5-5`, ten splits into two groups of three, fifteen pairs of
single runs. Results: `docs/experiments/annotation-driver/results/slice-b/annot-opus-ensemble/`
(`ensemble.json`). All thirty calls returned usable annotations; none was refused.

## Evidence and results

Median agreement between two selections (lowest to highest over the splits or pairs):

| File | Single runs | Majority of three | Union of three |
| --- | --- | --- | --- |
| Rust `update.rs` | 0.75 (0.70 to 0.92) | 0.75 (0.71 to 0.79) | 1.00 (0.88 to 1.00) |
| Python `ratelimit.py` | 0.82 (0.71 to 0.90) | 0.86 (0.78 to 0.90) | 0.89 (0.81 to 0.91) |
| TypeScript `state.ts` | 0.84 (0.71 to 0.94) | 0.91 (0.84 to 0.97) | 0.92 (0.87 to 0.95) |
| TypeScript `verification.ts` | 0.62 (0.46 to 0.87) | 0.74 (0.68 to 0.81) | 0.60 (0.53 to 0.67) |
| Java `Lexer.java` | 0.86 (0.78 to 0.94) | 0.96 (0.88 to 1.00) | 0.87 (0.85 to 0.87) |

Median number of sites kept, against the eligible sites:

| File | Single run | Majority of three | Union of three | Eligible |
| --- | --- | --- | --- | --- |
| Rust `update.rs` | 21 | 21 | 24 | 71 |
| Python `ratelimit.py` | 57 | 56.5 | 66.5 | 211 |
| TypeScript `state.ts` | 34 | 34 | 37.5 | 70 |
| TypeScript `verification.ts` | 15 | 15 | 24 | 64 |
| Java `Lexer.java` | 32.5 | 33 | 36.5 | 184 |

- The majority rule narrows the spread everywhere: its worst split is better than the worst pair of single
  runs in every file.
- The union is perfect in the Rust file, where the runs draw from a small pool of 24 sites, and worst in
  `verification.ts`, where each run adds sites the others do not name.
- Cost: 30 calls, 3.25 USD at list prices. Recorded spend after this experiment: about 32.9 of 40 USD.

## Limits and interpretation

- One model, six runs per file, five files. The ten splits of six runs are not independent of each other.
- Single-run agreement in these six runs (0.62 to 0.86) is a little below the five runs of R83 (0.69 to
  0.92) for two files; that is the size of variation between batches of runs.
- Three runs is one choice; five runs with a threshold of three was not tried.
- The rules do nothing for a model that often returns no annotations (R83).

## Tool/spec consequence and next gate

- No change to the tool. `--annotations` takes any list; a majority list is made outside the driver, as here.
- If the owner wants the rule in the product, it is a small command that merges annotation files, to be
  specified separately.
- Slice B after R66 to R85: measured for pack authoring and annotation, between runs and models, with two
  optimisers and two selection rules. Not measured: one model over time.

## COBESY check

Outcome first: two claims hold and the one with an absolute target is refuted. Replay: inputs and result
directory named. Disproof: thresholds applied as written. The recommendation is worded so that it does not
claim more stability than was measured.
