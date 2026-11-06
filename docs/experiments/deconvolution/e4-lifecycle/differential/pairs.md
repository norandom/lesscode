# E4 task 6.5: the differential pairs (preregistered, written before any gate ran)

- Written: 2026-11-06, Europe/Berlin, deconvolution-loop task 6.5 phase 1, after 6.3 (review-e4.md) and while the
  6.4 acceptance-arm series runs on this host. Fixed before execution: the pairs, their domains, the planted mutants,
  the renamings, the tiny domains and the predicted outcomes below. Anything decided later is a dated note at the end.
- Governing text: [plan.md §6.5](../plan.md) (at most **3 pairs** of pure functions, same signature on both trees, each
  tied to a named reviewer hypothesis, chosen from the original's pure core, the four verifier gates of E3 Amendment 2.8,
  then `diffbehavior` at 300 s base scaled; a corpus-check disagreement is a difference found by the corpus and the solver
  verdict is not gated; a failing control voids 6.5; domains as E3 Amendment 2.7) and the kill criteria (criterion 4).
- Hypotheses and findings by name from [review-e4.md](../review-e4.md) sections 4, 6 and 8.
- Trees: original = the frozen copy `.runs/e2-facdrone/source` (read only); candidate = the accepted turn-3 root
  `.runs/e4-lifecycle/cand-turn3` (read only). Scratch copies (planted mutant, renaming) exist only as text in the
  driver process; nothing is written to either tree.

## Difference from the task brief, reported as the plan requires

The brief asks for at least one pair per H1, H2, H3 and H9 plus control pairs for unchanged functions. The frozen plan
allows **at most 3 pairs**, each "of pure functions (same signature on both trees)" from the listed candidates
(`_rules_revision`, `_flag_reason`, `_reading_id`, `baseline_view`, `_daily`, `_lookback`, `_observed_day`,
`LiveEvidence.for_window`, `monitor_label`, `labeled_reason`, `_latest_basis`, `_baseline`, `_reading`,
`_latest_and_not_run`, `_flag`, `_state`, `signal_standing`, `_event_id`, `_window`, `_compare`, `MonitorRunner._judge`).
The plan wins:

- H1 is covered by the `compare` pair (F3, the only H1 finding that sits in a listed pure function; F1 lives in
  `MonitorRunner._cycle`, an effect-shell method over the engine, not a pure pair).
- H2 (`bind_writers`, F2) is not in the plan's list and is declared **shell** in the candidate's structure
  (`results/structure-e4-t3.json`); H3 (`LifecycleStore.transition`, F4) needs the in-memory store subclass and is
  shell; H9 (F5, patch timing around `bind_lifecycle_monitor`) is not a function differential at all. These three stand
  for T6 as **probe-found** differences of 6.3 (F2, F4, F5), not as 6.5 pairs; 6.7 reports them as such.
- No slot is spent on an unchanged-function control pair: with three slots, the per-pair controls the plan requires
  (3a planted mutant, 3b equivalent renaming) are the apparatus controls, and the two restructured pairs below
  (`standing`, `window`) carry a predicted "agree" that serves the same purpose on changed code. This is a choice
  the owner may overrule before phase 2 (a dated note here, no line of the drivers changes).

## The three pairs

| # | Name | Original (file:line) | Candidate (file:line) | Hypothesis (review-e4.md) | Predicted |
| --- | --- | --- | --- | --- | --- |
| 1 | `compare` | `cli/lifecycle_group.py:203` `_compare(stored, fresh) -> int`, echoes each line through `typer.echo` as it goes | `cli/lifecycle_group.py:195` `_compare(stored, fresh) -> tuple[list[str], int]`, returns the lines; the command echoes after the loop | **H1** error and malformed-input paths (**F3**: partial output lost on a stored-row fault), **H11** CLI output | **counterexample, F3 shape**: a well-formed non-`window` row followed by a row missing `disposition`, `value` or `monitor`: both raise `KeyError`, the original has already emitted the earlier line(s), the candidate has emitted nothing. Expected to be found already by gate 2 (the 300-input corpus), so the solver verdict is **not gated** (plan 9.3). Well-formed rows and a malformed first row agree. |
| 2 | `standing` | `lifecycle_projection.py:139` `signal_standing(standing, baseline, readings, flag, moment)` over the helpers `_latest_basis` (63), `_latest_and_not_run` (103), `_state` (129), `_baseline` (73), `_reading` (87), `_flag` (116) | `lifecycle_projection.py:61` `signal_standing(...)`, one body, the six helpers inlined | **H8** projection helpers inlined, **H7** time handling (flag age at and past the overdue age) | **agree** on every well-formed and singly-malformed input (the review's 16 H8 and 9 H7 probes agreed). One **predicted error-ordering difference** from reading both bodies: with `standing = None` (malformed) **and** a malformed baseline (`states` holding a non-mapping, or `traded_ic`/`survival` shapes that raise), the original evaluates `standing.signal` first (`AttributeError: 'NoneType' object has no attribute 'signal'`), the candidate computes the baseline view first (`AttributeError: 'str' object has no attribute 'get'`). Same type, different message, so the canonical result differs. If the corpus or the solver finds exactly this, it is reported as an error-path difference of the H8 class (double malformation), realised only through inputs the type `Standing` forbids; otherwise `inconclusive` (never read as equivalent). |
| 3 | `window` | `lifecycle_evidence.py:166` `LiveEvidence.for_window(signal, window)` calling `_lookback` (124), `_daily` (103), `_observed_day` (95) | `lifecycle_evidence.py:148` `for_window`, `_lookback` inlined; `_daily` (100) and `_observed_day` (92) unchanged | **H13** inlined lookback (conflicts, naive instants, non-tuple windows, ambiguous observed dates) | **agree**: the inlined block is the `_lookback` body verbatim in the same position (both compute `series` before the lookback and raise the conflict before the "no IC" return). Solver verdict expected `inconclusive` (the domain is not exhaustible); a counterexample would be a real H13 finding. |

### Pair 1, `compare` (adapted comparison)

- Adaptation: the original prints through `typer.echo`; the driver rebinds the module global `typer` of the loaded
  module to a namespace whose `echo` appends to a list, so both sides yield `(lines emitted, code)`; an exception yields
  `(lines emitted before it, ("raise", type, message))`. This is what the CLI user sees (the text before a failure and the
  exit code), so the pair compares the observable of `lifecycle_readings` without the command's own plumbing.
- Domain (`o_compare(rows: list[Rowv], fresh: list[Fresh])`): `rows` at most 3 stored rows, each
  `(monitor, disposition, value, boundary, shape)` with `monitor % 3` indexing `("survival", "ic_gap", "window")` (index 2 is
  the not-run marker row the function skips), `disposition % 3` indexing `("within", "breach", "not_run")`, `value` and
  `boundary` plain ints, `shape % 4` dropping the key `disposition`, `value`, `monitor` or nothing; `fresh` at most 3
  `(monitor, value, boundary, disposition)` building a dict monitor name to a frozen dataclass `R(value, boundary,
  disposition)` (the three attributes `_compare` reads through `getattr`).
- Pure enough: no engine, no clock, no I/O after the `typer` rebinding; dict and tuple comparisons and one f-string.
- Planted mutant (3a), candidate line 204: `        differs += not same` becomes `        differs += same` (the exit code
  inverts for every non-marker row). Expected: found by the corpus (and by the solver in phase 2).
- Renaming (3b), candidate lines 197 to 207, locals `lines, differs, again, now, same, verdict, row` prefixed `r_`
  (word-bounded; none occurs inside a string literal in that range). Expected: no spurious disagreement on the corpus;
  exhaustion on the tiny domain in phase 2.
- Tiny domain (`ct_compare / et_compare(a, b, c)`): row 1 `survival within 1 2` well formed when `a` else missing
  `disposition`; row 2 `ic_gap breach 1 2` well formed when `b` else missing `value`; `fresh` holds `survival` when `c`.

### Pair 2, `standing`

- Domain (`o_standing(standing: tuple[int, int, int], baseline: Optional[tuple[int, int]], readings: list[Rd],
  flag: Optional[tuple[int, int]], now: int)`): `standing = (kind, state, days)` with `kind % 3` choosing
  `UnrecordedStanding("carry", 3)`, `StandingRevision("carry", 3, LifecycleState[state % 4], 2, AS_OF - days % 30 d,
  "census:x")` or `None` (malformed); `baseline` `None` or `(x, y)` choosing `route` (`"scored"` or `None`),
  `traded_ic` (`{"mean": 0.05}`, `0.1` non-mapping, `None`), `survival` (`{"ratio": 0.8}`, `None`), `states`
  (two mappings, `["calm"]` non-mapping, `None`) and whether `digest` (`"a" * 40`) is present; `readings` at most 3 rows
  `(window, monitor, disposition, basis, shape)`: `window_id` `w0..w2`, monitor and disposition as in pair 1, basis
  choosing `baseline_digest` `d0/d1` and `rules_revision` `r0/r1`, `shape` choosing the payload (`{"limitation": "no IC"}`,
  `None`, `{}`) and the key dropped (`monitor`, `disposition`, `window_id`, none); `flag` `None` or `(age, reason)`:
  `OpenFlag("carry", 3, "FEV", reason, AS_OF - age % 30 d - (age // 30 % 2) s, "dec")` with the reason
  `"traded_horizon_ic breached on 3 consecutive windows"` or `"plain"`; `moment = (AS_OF + now % 3 d, 14 d)`.
  `AS_OF` is `facdrone_testing.core.cppi.AS_OF` (2026-09-07 12:00 UTC), outside the scope.
- Pure enough: dict and dataclass reads, `datetime` arithmetic on fixed instants, `OpenFlag.overdue` (kernel, pure).
- Planted mutant (3a), candidate line 79: `        if r["disposition"] == "not_run":` becomes `!=` (not-run rows and
  readings swap). Expected: found by the corpus.
- Renaming (3b), candidate lines 71 to 139, locals `now, after, newest, basis, latest, shown, shown_readings, shown_flag,
  no_reading, r` prefixed `r_` (`not_run`, `reason`, `state`, `survival`, `traded` are excluded because the same words
  occur as dict keys or attribute names in that range). Expected: no spurious disagreement; exhaustion on the tiny domain.
- Tiny domain: `standing` revision when `a` else unrecorded; `baseline` `None` when `b` else the well-formed shape;
  `readings` one not-run row when `c` else none; no flag; `now = AS_OF`.

### Pair 3, `window`

- Domain (`o_window(consumed: list[IC], states: list[tuple[int, int]], ic_days: int)`): `consumed` at most 3
  `SimpleNamespace` consumed ICs `(observed, value, known, kind, horizon)` as the review's probes build them:
  `observed_at = 2026-09-01 + observed % 7 d`, `knowledge_at = 2026-09-01 12:00 UTC + known % 7 d`, `value = value % 1000 / 10`,
  `horizon_days = 1 + horizon % 2`, `signal = ("carry", 3)`; `kind % 4`: 0 aware, 1 naive `knowledge_at`, 2 `observed_at`
  as a datetime at 05:00 UTC (ambiguous session date), 3 `window = "absent"` (non-tuple); `states` at most 3
  `(day, state)` giving the scripted readings callable `cutoff -> {2026-09-01 + day % 7 d: entry.state.value in
  ("calm", "stress")}`; `ic_days % 12`. The call is `LiveEvidence(readings, consumed, ic_days).for_window(("carry", 3, 1),
  Window("2026-09-05", 2026-09-03, 2026-09-05))`. The result is the `MonitorEvidence` repr or the not-run string.
- Pure enough: no engine (`LiveEvidence` holds the already-consumed list), no clock, the readings callable is a closure
  over a dict built from the input.
- Planted mutant (3a), candidate line 162: `        traded = [c for c in mine if c.horizon_days == horizon]` becomes `!=`
  (the traded series takes the other horizon). Expected: found by the corpus.
- Renaming (3b), candidate lines 150 to 206, locals `ic_window, since, through, mine, series, recent, latest, lookback,
  state_of, by_state, day, known, horizons, by_horizon` prefixed `r_` (`name`, `version`, `horizon`, `traded`,
  `readings`, `value`, `state`, `state_intervals` are excluded: attribute names, a keyword argument, or words inside the
  not-run string). Expected: no spurious disagreement; exhaustion on the tiny domain.
- Tiny domain: one consumed IC (observed 09-02, value 0.2, known 09-04, horizon 1) when `c` else none, aware when `a`
  else naive; one `calm` reading on 09-03 when `b`; `ic_days = 10`.

## Gate protocol (plan 6.5, E3 Amendment 2.8), what runs in which phase

| Gate | Form | Phase | Pass |
| --- | --- | --- | --- |
| 1 symbolic-ness | `crosshair check e5_drivers.smoke` (E1c-2 launcher), `E5_SMOKE=<pair>`, 15 s per condition (60 s where a path takes longer, as E3) | 1 | more than one entry marker per side |
| 2 corpus | 300 Hypothesis inputs from the driver signature, seed 20261106, `o_` against `c_` | 1 | 300 of 300 agree for a pair predicted to agree; for pair 1 the first disagreement is the candidate counterexample and the solver verdict is marked not gated |
| 3a planted mutant | corpus `c_` against `k_` (phase 1), then `diffbehavior k_ c_` at the scaled deadline (phase 2) | 1 + 2 | corpus finds it; solver yields a counterexample |
| 3b renaming | corpus `c_` against `e_` (phase 1), then `diffbehavior et_ ct_` on the tiny domain (phase 2) | 1 + 2 | no spurious disagreement; "All paths exhausted" |
| solver | `diffbehavior o_ c_` at 300 s base scaled | 2 | verdict `counterexample`, `inconclusive` or `exhausted`, never `inconclusive` read as equivalent |
| 4 replay | `--replay` of every `Given:` on both sides concretely | 2 | the sides differ concretely; then the oracle question (expressible in the alphabet or not, plan 9.6) |

A control that fails (3a without a counterexample, 3b spurious or not exhausted) is read by the plan's criterion 4
literally: "A verifier control of 6.5 fails: 6.5 is void, reported as void; 6.2 to 6.4 and 6.6 stand." The brief's
per-pair reading (only that pair void) is reported beside it, labelled. The driver may be fixed once with a dated note;
the specimen is never touched. (Wording corrected 2026-11-06 after the independent review of 6.5; it said "voids the
pair" before.)

## Oracle expressibility, stated in advance (plan 9.6)

- Pair 1's predicted difference needs a stored reading row lacking a key. `cli_readings` reads rows the store wrote
  through `_append`, which always writes every key; the alphabet has no operation that corrupts a stored row. Expected
  verdict: **inexpressible**, reported as such.
- Pair 2's predicted error-ordering difference needs `standing = None` and a malformed baseline; `standing` and
  `standing_view` derive both from the store and the ledger. Expected: **inexpressible**.
- Pair 3: no difference predicted; if one appears, expressibility is judged on the realised input.

## Dated notes

- **2026-11-06, driver fix 1 (receipt note `e5-driver-fix-1`, fact `task-6.5-apparatus-2`).** The first `--smoke compare`
  run (`e5-compare-smoke`) exited with code 3758096385 (0xE0000001) and an empty log, while the same CrossHair launcher
  run directly through the receipt (`e5-compare-smoke-diag`) passed with 54 entries per side. The scratch diagnostics
  `e5-compare-smoke-diag3/4/5` isolated it: a parent process that has imported `facdrone_testing.core.cppi` exits with
  0xE0000001 as soon as a child process imports the same module (child variants `testing`, `drivers` kill the parent;
  `shells`, `sleep` and a parent without that import do not). The driver imported it only for the constant `AS_OF`
  (2026-09-07 12:00 UTC); the constant is now written in the driver and the import is gone. No pair, domain, mutant,
  renaming or prediction changed; the specimen is untouched; the compare gates are rerun as `e5-compare-*-2` and the
  first entries stay in the receipt. The mechanism inside `facdrone_testing` is not investigated here (outside the scope).
- **2026-11-06, driver fix 2 (note `e5-driver-fix-2`, fact `task-6.5-apparatus-3`).** After fix 1 the `--smoke` wrapper still
  failed: the launcher child returned exit 1 with zero entries and `MemoryError` at interpreter teardown
  (`e5-compare-smoke-2`), or the parent died 0xE0000001 again (`e5-compare-smoke-diag7-file`), while the launcher run
  directly under `receipt_atomic.py` passed (`e5-compare-smoke-diag6`, 71 entries per side). The common factor is a
  parent that has loaded the two trees (not `facdrone_testing` alone, as fix 1 read it). The driver now skips the tree
  load in the modes that only spawn CrossHair (`--smoke`, `--solve`, `--gates`), so the launcher's parent is as clean as
  `receipt_atomic.py`; the child still loads everything. Pairs, domains, mutants, renamings and predictions unchanged;
  the compare gates are rerun as `e5-compare-*-3`; every earlier entry stays in the receipt.
- **2026-11-06, driver fix 3, the one control fix the brief allows (note `e5-driver-fix-3`, fact `task-6.5-apparatus-4`).**
  Three driver defects, none in the specimen: (a) gate 3b for `window` was **spurious** (`e5-window-rename`): the locals
  `latest` and `lookback` also occur inside the message string of candidate line 173 ("conflicting latest consumed
  lookback"), so the renaming changed an exception message; both names are removed from the renaming set (12 locals
  remain). (b) Gate 1 for `standing` ran no path (`e5-standing-smoke`: `TypeError: unsupported operand type(s) for -:
  'datetime.datetime' and 'timedelta'`): CrossHair's symbolic `timedelta` built from a symbolic int cannot be subtracted
  from the concrete `AS_OF`; the driver now indexes concrete tables (`DAYS[days % 30]`, `NOWS[now % 3]`,
  `AGE[age % 30][age // 30 % 2]`) and the row ids `w0..w2`, `d0/d1`, `r0/r1` are tuple lookups instead of f-strings.
  (c) Gate 1 for `window` stopped at the same class of fault (`datetime.combine() argument 1 must be datetime.date, not
  date`): dates, instants and the ambiguous 05:00 datetimes come from concrete tables (`D7`, `K7`, `A7`), the IC value is
  `(0.1, 0.2, 0.3)[value % 3]` (was `value % 1000 / 10`) and `ic_days` is `tuple(range(12))[ic_days % 12]` so the
  specimen's own `timedelta(days=self.ic_days)` sees a concrete int. The domains keep their shape (enumerations as
  indexes, lists of at most 3, no rejection guard); the pairs, the planted mutants, the tiny domains and every
  prediction are unchanged. All twelve light gates are rerun from the re-registered file (`e5-compare-*-4`,
  `e5-standing-*-2`, `e5-window-*-2`); the earlier entries stay in the receipt. The gate-2 verdicts of the first runs
  (compare: differ with the F3 shape; standing: differ with the predicted error ordering; window: agree) are evidence
  already, since gate 2 does not use the parts that changed except the `window` value and date tables.
- **2026-11-06, driver fix 4, summary mode only (note `e5-driver-fix-4`, fact `task-6.5-apparatus-5`).** After the nine
  solver runs and the four concrete replays, `--gates` (`e5-gates-3`) crashed on the replay result files, which are
  lists, not dicts (`isinstance(v, str)` became `isinstance(v, (str, list))` in the one summary line). No gate, driver
  function, domain, mutant, renaming, deadline or verdict is touched; every solver and replay result file predates the
  fix and is unchanged. The summary is rebuilt as `e5-gates-4`.
- **2026-11-06, phase 2 record.** The `--replay` mode reads the text that holds `Given:`; the receipt log of a `--solve`
  run holds only the filtered summary, the full CrossHair output is the `output` field of `results/e5-solve-P-K.json`.
  The receipted step `e5-given-extract` wrote each `output` to `results/logs/e5-P-solve-K.crosshair.txt` (sha256 in its
  log and in fact `task-6.5-phase2`) and the replays ran on those (`e5-compare-replay-2`, `e5-standing-replay-2`); the
  first replay attempts on the receipt logs (`e5-compare-replay`, `e5-standing-replay`, exit 1, no `Given:` found) stay
  in the receipt. The serial phase-2 script was reaped by the harness after `e5-compare-solve-plant` (note
  `e5-phase2-stop-1`); the remaining eight runs ran one per job (note `e5-phase2-resume-1`), nothing rerun.
