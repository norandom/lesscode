# Composition blueprint: the deconvolution-loop run report and prototype résumé

COBESY composition back-end, written 2026-11-06 before the prose. Recorded so the report's structure can be audited
against it (post-outline). The owner asked for prose, so the humanizer pass applies.

## Phase 0, intake

- raw_knowledge: research entries R19 to R26, the deconvolution-loop spec (requirements, design, tasks with
  Implementation Notes), the E3 and E4 experiment directories with receipts, the regenerator program, the applied
  report of 2026-11-04 (`deconvolution-loop-applied.md`).
- cultural_assumptions: one owner, a researcher who preregisters and reports failed predictions; "a cleaner, smaller AI
  version is preferred when it holds"; tests belong to the TDD workflow; WSL is CI-only; nothing claimed without a
  receipt.
- network_topology: the owner decides alone; third-party readers (a future implementer, a reviewer) read later.
- Target behaviour: the owner decides, with the evidence in front of them, whether to start a Rust implementation of
  the reusable parts, run a third scope first, or stop. One observable decision.
- Hidden assumption (Schein): "a prototype that accepted twice is an implementation-ready design". The report must
  separate what is proven on two Python scopes from what has no evidence (other languages, scope selection, multi-scope
  cost), or the decision rests on an assumption the data does not support.
- High-leverage anchor (Jackson): the owner themself; a secondary anchor is a future Rust implementer who will read the
  prototype résumé as a contract.

## Phase 1, systemic filter

- Artefacts: receipts, preregistered plans, dated amendments, independent reviews. Espoused value: falsifiability.
  Basic assumption: a result without a receipt is not a result.
- Edmondson quadrant: high safety, high accountability (learning zone). No safety work needed; the framing is
  Scientist Mode: state predictions, report misses first.
- framing_protocol: frame the report as the evidence record for one decision; lead with what failed (T2 on E4, T4 on
  E3, the void differential, the rejected hypotheses) before what held; never say "proven" about anything outside the
  two scopes; keep the cost and host-time figures beside every claim.
- Disproof condition for the report's recommendation: "the prototype is mature enough for a Rust implementation of the
  agnostic parts" is wrong if a third scope under the same protocol fails T7 (held-out divergence) or exhausts four
  turns, or if the Rust recorder of R20 cannot be made byte-deterministic for a store-backed scope.

## Phase 2, composition blueprint

### 0. Format
Technical report (long-form), markdown, two deliverables: (a) a new run report with the prototype résumé;
(b) the applied paper updated in place.

### 1. Governing idea
One sentence: after two Python scopes the loop's agnostic parts (acceptance rule, recorded-trace oracle contract,
turn rule, measures, verifier gates, residue, receipts) behaved as specified and can be written down as a contract for a
native implementation, while the bound parts (surface derivation, recorder, measures, mutation, solver) must be rebuilt
per language and have evidence only for Python plus one Rust recorder.
Visible in: title, executive summary, every head, the résumé's "copy / rebuild / do not build" table, the conclusion.
Argue, don't report: the claim is "mature enough to specify, not yet mature enough to ship as tools"; the evidence is
selected for that claim; the full catalogue lives in the receipts.

### 2. Four keys (Kolin)
- Audience: the owner (knows SMT and verification; wants a decision on implementation effort); secondary a Rust
  implementer and an outside reviewer.
- Purpose: enable one decision about Rust tooling; record the run so it can be audited.
- Message: the loop works as a measured instrument on Python; the contract is stable; the tools are not.
- Style: plain, numbers in tables, failed predictions first, no hype, reminders of one or two sentences for the
  verification concepts.
- Action: pick one of three options at the end (third scope, Rust build of the agnostic core now, stop).

### 3. Opening (Cron)
- Protagonist: the owner who must decide whether to spend implementation effort.
- Situation: six experiments and one full task batch later, two regenerated candidates passed a recorded oracle.
- Stakes: building Rust tools on an unproven contract wastes the effort; waiting forever loses the method to drift.
- Opening guidance: start with the decision and the two failed predictions, then the run, then the résumé.

### 4. Skeleton grid

| # | Head | Bullets | Emphasis / cue | Keywords | Serves idea |
|---|---|---|---|---|---|
| 1 | Executive summary: what held, what failed, what it costs | E3 and E4 results in one table; cost per scope; the recommendation and its disproof condition | lead with T2 and T4 misses | preregistered, held, failed, receipt | yes |
| 2 | The process, as run | Kiro flow (requirements, design, tasks, implement, review, verify, commit); the task batch of 2026-11-05/06 in order with commit ids; what "independent review" meant here | the reader can replay the process | spec, task, reviewer, verify | yes |
| 3 | The run in detail: E4 on the lifecycle scope | scope and rationale; apparatus (5.2); plan (6.1); turns (6.2); review (6.3); sensitivity (6.4); differential (6.5); held-out (6.6); residue (6.7); each with numbers and the kill-criterion status | one subsection per task, same order as the process | turn, arm, corpus, kill rate, residue | yes |
| 4 | The regenerator program: six turns, what the model did | E3-scope smoke (three turns) and E4 (three turns); token usage, elapsed, failures and their causes; the turn rule exercised once | the model needs 128k output at medium effort; thinking-only turns | turn rule, feedback, max_tokens | yes |
| 5 | Prediction table after one of three scopes | report section 8 table with the E4 row filled; what the second and third scope must show | lines outside the band, the rest inside | band, falsifier | yes |
| 6 | Apparatus findings worth carrying | surface = tree ∪ recorder imports; CRLF hashing; label collisions; harness reaping; probe receipts; stdout-only arm; patching check | each as a spec amendment | amendment | yes |
| 7 | Prototype résumé: what exists, what is proven, what is not | table of components: name, prototype file(s), lines, evidence (R-entry), status (proven on 2 scopes / proven on 1 / built not exercised / unbuilt); the copy / rebuild / do-not-build split from the design; what a Rust implementation would need first | the contract for an implementer | agnostic, bound, copy, rebuild | yes |
| 8 | Maturity assessment against the owner's question | criteria for "reusable mature tools": stable contract, two independent runs, reproducible receipts, known failure modes, per-language rebuild cost; score each honestly | say where it is not mature | maturity, reuse | yes |
| 9 | Options and the recommended order | third scope first vs Rust core now vs stop; what each costs and what each would disprove | one recommendation, with its disproof condition | decision | yes |
| 10 | References | R19 to R26, spec files, experiment directories, the applied paper | | | yes |

- MECE: process (2), evidence (3, 4, 5), findings (6), résumé (7), judgement (8, 9). No overlap.
- Post-outline after drafting: topic sentences must echo the heads.

### 5. Paragraph moves (Graff & Birkenstein)
- They say: "two accepted candidates mean the method is ready to be productised".
- Naysayer: one language, two scopes, hand-picked; the differential's solver failed its own control once; the line
  prediction failed; the original's own sensitivity sample was composition-limited; cost per scope is a working day.
- Answer: the agnostic parts are specified and reproducible (receipts, reviews, held-out corpora); the bound parts are
  Python-only; so the mature artefact today is the contract, not the tools.
- So what: a Rust build of the agnostic core is bounded and low-risk; a Rust recorder for store-backed code is the
  unknown and should be the first thing a third scope or a Rust pilot tests.
- Emphasis: numbers in tables, failed predictions before held ones, one recommendation.
- Style: short sentences, subjects are actors, no nominalisation, reminders for SMT/verification terms in one sentence.
- Writing advice: cite the receipt or entry for each number; never round away a failure; keep the owner's stance on
  tests visible.
- Marketing-language ban in force.

### 6. Linter and humanizer
Run after drafting: first-paragraph drop-off, data dump, mismatched heads, scope creep, flat exposition, expert fog,
sprawl, reader mismatch, AI voice. Humanizer: no em dashes, no curly quotes, no decorative bold, no forced trios, no
upbeat endings, varied rhythm, the owner's voice.
