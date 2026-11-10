# Composition blueprint: feasibility of the agnostic mutation driver as one Rust binary

COBESY composition back-end, written 2026-11-10 before the prose, from the skill's `diagnostic-intake` and
`composition-blueprint` templates. Recorded so the report can be audited against it (post-outline). The owner
asked for a report, so the humanizer pass applies.

## Phase 0, intake

- raw_knowledge: research entries R55 to R64, the `agnostic-annotation-driver` spec (brief, prototype gate with
  Amendments 1 to 12 and both results), the prototype and its retained receipts under
  `docs/experiments/annotation-driver/`, the native crate as it stands (`native/src/adapters/*`,
  `native/Cargo.toml`).
- cultural_assumptions: one owner who preregisters, wants failed predictions reported first and nothing claimed
  without a receipt; who said on 2026-11-10 that the apparatus must be ported to Rust, that two runtimes are too
  complex, that breaking codebases must be avoided, and that the runs were too slow.
- network_topology: the owner decides alone. Later readers: an implementer of the Rust port and an adopter who
  wants to know whether the tool can damage their code.
- Target behaviour: the owner decides whether to start requirements and design for a Rust port of the driver,
  and with which first step. One observable decision.
- Hidden assumption (Schein): "the gate passed, so the port is a formality". The report must separate what was
  shown (a mechanism, in a Python prototype, on five files) from what was not (a Rust implementation, its size,
  its speed on Rust scopes), or the decision rests on evidence that does not exist.
- A second assumption to correct, from the owner's question: that the Rust figures describe flawed
  deconvolution cuts. They describe deliberate small bugs that tests did or did not catch. No code was
  deconvoluted in this work.
- High-leverage anchor (Jackson): the owner; secondary, the implementer who will read the port section as a
  work list.

## Phase 1, systemic filter

- Artefacts: receipts, a gate with dated amendments, independent labels. Espoused value: falsifiability.
  Basic assumption: a result without a receipt is not a result.
- Edmondson quadrant: high safety, high accountability. No safety work needed. Scientist Mode: say what failed
  and what was not tested before what held.
- framing_protocol: frame the report as the evidence for one decision. Answer the owner's question in the
  opening, because a wrong reading of the Rust numbers would colour everything after it. Never say "proven";
  say on which files and with which tests. Keep "prototype" and "port" apart in every sentence that makes a
  claim. Put the safety property (copies, restore, error class) before the results, since "do not break
  codebases" is the owner's stated constraint.
- Disproof condition for the recommendation: "a single-binary Rust port is feasible within the existing
  dependency set plus two grammars" is wrong if the tree-sitter grammars pinned by the native crate reject the
  packs' node kinds for a language, if the port cannot be made to restore a scope byte for byte under an
  interrupted run, or if the sized port exceeds the apparatus cap with no removable counterpart.

## Phase 2, composition blueprint

### 0. Format
Technical report (long-form), Markdown, one file under `docs/reports/`.

### 1. Governing idea
One sentence: the agnostic driver is feasible as a single Rust binary because its mechanism was shown to work on
four languages with one engine and a small data pack per language, the port replaces line-scanning adapters with
that engine using parsers the crate already depends on, and the design keeps a codebase safe by working only on
copies and by reporting where tests are too weak to permit a cut; the port itself, its size and its speed on
Rust code are not yet shown.
Visible in: title, opening claim, every head, the conclusion's decision list.
Argue, don't report: the claim is "feasible, with three things still to show"; evidence is selected for that
claim; the catalogue lives in R55 to R64.

### 2. Four keys (Kolin)
- Audience: the owner; knows verification and the project; wants a feasibility judgement, not a catalogue.
- Purpose: enable the decision to start requirements and design for the Rust port.
- Message: mechanism shown, port plausible and bounded, safety designed in, three unknowns named.
- Style: plain, numbers in tables with their source entry, misses before hits, no hype.
- Action: approve or change the first step (a sizing spike inside the design phase) and the port's scope
  (mutation only, or mutation and analysis).

### 3. Opening (Cron)
- Protagonist: the owner, holding a tool whose mutation command works on four specimen files and nowhere else.
- What is happening: a prototype replaced the hard-coded operators and passed its gate on the second attempt.
- What is at stake: whether a Rust port is a bounded piece of work or an open-ended one, and whether running the
  tool can harm a codebase.
- Opening guidance: state the claim, then answer the "flawed cuts" question in three sentences, then the
  reading order.
- Words to avoid: proven, robust, seamless, any claim about compression.

### 4. Skeleton grid

| # | Head | Points to cover | Emphasis | Keywords | Serves the idea |
| --- | --- | --- | --- | --- | --- |
| 1 | What the Rust numbers mean | mutants are deliberate bugs, not cuts; 45 caught, 21 not; whose tests; no deconvolution run | correct the reading first | mutant, survivor, test gap | yes |
| 2 | The tool does not touch your code | copies, byte-exact restore, baselines, error class, what a stopped run leaves | owner's constraint | isolated copy, restore | yes |
| 3 | What was shown | arm D rows on five scopes; model-written packs identical; annotation optional | misses of attempt 1 first | data pack, held-out | yes |
| 4 | What the results say about cutting code safely | survivors per scope; kinds of gaps; tests first | link to deconvolution without claiming compression | oracle sensitivity | yes |
| 5 | Feasibility of the Rust port | what is replaced, what is added, dependencies, size estimate, speed, risks | prototype versus port | grammar, pack admission | yes |
| 6 | What is not shown | port, size, typed-language invalid rate, repair path, breadth, compression | plain list | not measured | yes |
| 7 | Decisions | port scope, first step, cap | one short list | sizing spike | yes |

- [x] MECE: safety, evidence, implication, port, gaps, decisions do not overlap
- [x] Post-outline to be checked after drafting

### 5. Paragraph moves (Graff and Birkenstein)
- They say: "the gate passed, port it"; and "the Rust cuts were flawed".
- Naysayer: a Python prototype on five files, three of them the owner's, with packs shaped by their author, proves
  little about a Rust implementation on unfamiliar code.
- Answer: agreed on breadth; the report claims feasibility of a bounded port, names the first measurements that
  would refute it, and proposes them as the first step.
- So what: the hard-coded operators can be deleted only when the port exists; until then the released command
  fails loudly, which is honest but not useful.
- Emphasis: the two tables an owner needs (results by scope; port work list).
- Style guidance: every number carries its entry; "prototype" for what ran, "port" for what would be built.
- Writing advice: short declaratives; a reminder of one sentence where a term first matters.

### 6. Linter pass (after drafting)

| Malady | Hit? | Fix applied |
| --- | --- | --- |
| First-paragraph drop-off | no | claim and the answer to the owner's question are in the first screen |
| Information overload | checked | per-arm tables of R64 are referenced, not repeated; one results table |
| Mismatched TOC | no | heads read as the argument |
| Scope creep | checked | compression and drift are named as out of scope, one paragraph |
| Flat exposition | no | each section ends on what it means for the decision |
| Expert fog | checked | actors named: the engine, the harness, the reviewer |
| Sentence sprawl | checked | long sentences split |
| Reader mismatch | no | written for the owner, who knows the project |
| AI slop | checked | no hype words, no decorative emphasis, no dashes as punctuation |

### 6a. Humanizer pass
- [x] No significance inflation, vague authority, forced trios or upbeat ending
- [x] No chatbot phrasing or signposting
- [x] No em dashes, curly quotes, emoji or decorative bold
- [x] Rhythm varied
- [x] Reader level kept
- [x] Asked what still sounds generated; removed two summarising closers
