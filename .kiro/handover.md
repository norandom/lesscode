# Handover: deconvolution research, state at 2026-11-04 evening

Written 2026-11-03 evening, updated 2026-11-04 after E3, Europe/Berlin.
Read this first, then the files it points to. Nothing is running.

## Where we are

E3 ran on 2026-11-04 (09:28 to 14:53) and is recorded as
[R25](../docs/research/2026-11-04/02-e3-evidence-regeneration.md). One
regeneration round met the oracle (T1), with 1,074 lines for 1,274, 546
statements for 660, 51 functions for 67, 60 intra-set edges for 84, 11 write
sites all in declared shell functions and 36 of 51 functions in a checked
pure core (T2, T3). T4 failed on its kill rate (18 of 28 = 0.643 against
0.70; M1 to M3 caught). T5 found one realizable oracle-blind difference
(`capture_payload` leaves no `AuditedAbsence` after an exception because
the candidate calls the route after deduplication); the reviewer and 300
random inputs had already exposed it, and the capture pair fails verifier
check 2 on a strict reading. Reviewer caveat: 26 statements of the reduction
are packing (about 572 in the original's style). No kill criterion fired;
H8(c) stands on one scope with a lowered protection claim. Research code
545 lines against 400, reported. Index, H8 lineage row and the spec
research protocol are updated. Owner decision pending: E3b (second round
against the known gaps, with the counterexample as a new oracle operation
under a new preregistration), E4 (second scope), or prevention.

Candidate root `.runs/e3-facdrone/cand` (only the five files differ from the
frozen copy); mutant copy `.runs/e3-facdrone/mut` verified identical to the
candidate at the end. E1c-2 (R24) stands as recorded.

Committed 2026-11-04 as `730e033`: R24, R25, the E1c-2 and E3 directories
(coverage data and files over 1 MB kept out, sha256 sidecars in place),
`adaptive.py` and its reference, the tooling-doc verifier section, this
file. The large golden trace `golden-e1c2.json` and `t5-soundness.json` are
on disk only; their hashes are committed beside them.

Release and archive (2026-11-04): spec, report, citation files and badge
committed (281a615, 9086551) and pushed; GitHub release `v0.1.0` published;
Zenodo archived it as DOI 10.5281/zenodo.23958364 under concept DOI
10.5281/zenodo.23958363 (the README badge and `CITATION.cff` carry the
concept DOI). Later releases archive automatically; bump `version` and
`date-released` in `CITATION.cff` and `.zenodo.json` first.

## Rules that are not negotiable

- Native Windows only, plain user account. WSL is CI-only; nothing executes
  there. Never call a bare `bash` from Python or PowerShell: Windows resolves
  it to the System32 WSL launcher. Use native Python, PowerShell, or the full
  Git Bash path. Tell every subagent this explicitly.
- No symlink privilege, Developer Mode or elevation assumed, by the research
  harnesses or by the desired implementation. Specimen tests that need them
  stay recorded as baseline failures.
- Never modify `D:\Source\facdrone`, `D:\Source\mmhmem`, or the frozen copies.
  Mutants only in separate copies under `.runs/`.
- One heavy job at a time; serial pytest for anything whose attribution must
  be deterministic; BLAS thread variables set to 1. Deadlines are adaptive:
  `docs/experiments/deconvolution/adaptive.py` (probe, scaled, wait_for_idle);
  never hard-cap wall clock; a timeout under load is rerun once when idle. The host ran out of
  memory once when cargo-mutants, a Facdrone pytest and an unrelated rustc
  build overlapped.
- Preregister before executing; amendments are dated and never change
  predictions after results; failed predictions are reported as failed.
- Research entries use `docs/research/ENTRY-TEMPLATE.md`, conclusion first,
  COBESY check at the end; update the index in the same change.
- mmhmem: call `session_resume_or_handoff` for project `lesscode` at start,
  checkpoint after each unit of work. Last checkpoint 2026-11-03 19:24 UTC.

## Where things stand, by hypothesis

H8 (recorded trace oracle + edge measures + regeneration), registered in
[R19](../docs/research/2026-11-03/01-dynamic-deconvolution-direction.md):

| Part | Status | Entry |
| --- | --- | --- |
| H8(a) oracle, Rust (mmhmem session bus) | mechanics positive; gain prediction hit a 96% seed-test ceiling | [R20](../docs/research/2026-11-03/02-e1a-session-bus-oracle.md) |
| H8(b) edge-based selector (Facdrone census) | rejected as formulated; C2 and C5 failed; recorder validity proven | [R21](../docs/research/2026-11-03/03-e2-facdrone-edge-census.md) |
| H8(a) oracle, Python (Facdrone evidence publication) | byte-deterministic, catches stored-representation change; kill rate 0.712 to 0.746; gap-halving failed | [R22](../docs/research/2026-11-03/04-e1b-evidence-publication-oracle.md) |
| Guided-generation rung (CrossHair on lifted predicates) | failed as run because drivers concretised strings and the oracle never calls the readers; one target killed by solver input | [R23](../docs/research/2026-11-03/05-e1c-concolic-rung.md) |
| Guided-generation rung, corrected and verified (E1c-2) | S1-S4 failed, S5 held; 7/10 killed, 3 unresolved, 0 proved; solver narrowed to a verified counterexample finder, never equivalence authority | [R24](../docs/research/2026-11-04/01-e1c2-corrected-solver-rung.md) |
| H8(c) regeneration (E3, five evidence modules) | T1-T3 held in 1 round; T4 kill rate 0.643 failed (M1-M3 caught); T5 one real oracle-blind difference on the exception path; stands on one scope with a lowered protection claim | [R25](../docs/research/2026-11-04/02-e3-evidence-regeneration.md) |

Tool assessment for symbolic execution per language:
[docs/symbolic-execution-tooling.md](../docs/symbolic-execution-tooling.md).

## Environment facts

- Facdrone frozen copy `.runs/e2-facdrone/source` at `d49f48bd…`, venv
  `.runs/e2-facdrone/venv` (CPython 3.13.16, from the copy's uv.lock plus
  coverage 7.13.0). Hypothesis extras `.runs/e1b-facdrone/extras`; CrossHair
  0.0.112 and z3 in `.runs/e1c-facdrone/extras`. Mutant copy
  `.runs/e1c-facdrone/mut`.
- Facdrone embedded PostgreSQL runs natively only with the two shim fixes in
  `docs/experiments/deconvolution/e2-facdrone-edge-census/e2_winshim.py`
  (unquoted `listen_addresses`, detached pipes on `pg_ctl start`); load with
  `-p e2_winshim` and the e2 directory on `PYTHONPATH`. Gate marker expression
  is `not port_baseline and not live_broker`; `not db` would deselect the
  database seeds.
- 236 Facdrone tests fail on this host at baseline (symlink privilege, file
  locks, POSIX-only modules); recorded, not repaired; identical with and
  without the recorder.
- mmhmem isolated copy `.runs/e1-mmhmem/original` at `8a9ce348`; cargo-mutants,
  cargo-llvm-cov, nextest installed natively; no nightly, so no Miri or
  cargo-fuzz.
- `python` is not on PATH; use `uv.exe run --no-project --python 3.13` or the
  venv interpreter.

## Specimen findings not yet reported upstream

- mmhmem: `checkpoint` panics when the 6,000-byte working-state cap falls
  inside a multi-byte character (`truncate_to_budget`, `session_bus.rs:355`).
- Facdrone test support on Windows: quoted empty `listen_addresses` and
  pipe inheritance on `pg_ctl start`; `os.geteuid` call in the fixture plugin.

## Spec written after E3 (uncommitted)

`.kiro/specs/deconvolution-loop/` (2026-11-04, owner request): brief,
algorithm-agnostic requirements with a per-requirement feasibility table,
a provisional design documenting the prototype as built and what a native
implementation would copy or rebuild, a retrospective prototype-gate
assessment (`proceed` on one Python scope, `revise` for size-under-
formatting and sensitivity, `not-run` for scope selection and other
languages), and research links. Owner approved requirements, design and
tasks on 2026-11-04 ("approved"). `tasks.md` (reviewed twice by an
independent agent, 18 findings applied) holds bounded apparatus revisions,
the regenerator-program prototype and E4; tasks marked "owner go" (3.2, 4,
5.2, 6.1 for 6.2 to 6.7) still need an explicit answer before they start.
Executed 2026-11-05 (owner: "continue the process as planned"), each by
an implementer subagent, an independent kiro-review and fresh
verification, then a selective commit: 1.2 (3a14745), 1.1 (3094ec5), 2.2
(a461ee8), 2.1 (e60c3a0), 3.1 (33180c5), 1.3 (606e37d), and on 2026-11-06
3.2 (eb80fc4). Findings worth knowing: formatted,
the E3 candidate has 1,188 lines, above the T2 bar of 1,083, and a
formatter cannot expose statement packing (spec gap in requirement 4.2);
the missing-test report was actionable (a fresh writer's tests killed 3 of
3 gaps and exposed the exception path); test-infra detection reproduces
the 19 importing files, the 236/24 deselections and the marker expression
but not the 191 coverage seeds. Details in `tasks.md` Implementation
Notes. Task 3.2 (2026-11-06): the same sensitivity arm on the original
scores 17/26 = 0.654 against the candidate's 18/28 = 0.643; neither meets
0.70; T4's failure measures the arm on this module, not a loss from
regeneration (R25 Amendment 2). The 2026-11-05 series was cut off after
mutant 24 when the session ended; a terminated arm reads as "killed" in
`classify`, so the rerun voided it by receipt note. Owner on 2026-11-06:
"proceed" (pause lifted); order 4 (waits for
`C:\Users\mariu\.lesscode\model.env`), then 5.2 scope choice (survey done:
lifecycle monitor group ranked first), then 6.1; 5.1 after 4.
Done 2026-11-06, each reviewed by an independent sonnet agent and
pushed: 4 (576e072: regenerator program `regenerator/regen_turn.py`,
three recorded turns against claude-opus-5-5, the third at 128k output /
medium effort produced the E3-scope candidate `.runs/e4-regen/cand-t4`;
the first turn's silent stream was cut at 310 s, the second spent 64k
tokens thinking), 5.1 (d5cbb2f: that candidate 1,214 lines / 549
statements / 53 functions / 63 edges / pure 0.679, edit-scope and
structure pass, behaviour untested), 5.2 (be6226d: lifecycle scope
apparatus `e4-lifecycle/`, 24-operation recorder, corpus 50 sequences /
317 operations recorded twice byte-identically, round 0 exit 0 in 75.7
s; goldens on disk under `.runs/e4-lifecycle`), 6.1 (1d9c59c: E4 plan
with T1 to T7 fixed, original-side sample and four hand-written
mutants, reviewer brief). Owner go for 6.2 to 6.7 asked 2026-11-06.
Lessons: receipt the probe that sets a deadline; sidecar hashes must
be over the bytes as stored (CRLF on Windows bit three times); a
terminated arm reads as killed in `classify`.
Owner stance recorded: tests belong to the TDD workflow; a cleaner smaller
AI version is preferred when it holds.

Applied recap paper (uncommitted): `docs/reports/deconvolution-loop-applied.md`,
the loop in eight steps, results R20 to R25, a preregistered prediction for
the next three scopes (lines 10 to 20 percent, functions 20 to 30 percent,
edges 25 to 35 percent fewer; kill rate 0.60 to 0.75; zero held-out
divergence as the trust row), and the DSPy-style regenerator program the
owner's model endpoint would run. Linked from the research index and the
spec's research.md.

## Open items after E3

1. Owner decision: E3b (second round on the same candidate against m17,
   m21, m29 and the `capture_payload` exception path, with the realised
   counterexample added as a new oracle operation under a new
   preregistration), E4 (same protocol on a second Facdrone scope), or stop
   and move to prevention at generation time. Do not start any of them
   without the owner's choice.
2. Two protocol rules proposed in R25, not yet adopted in the spec:
   statement counts in the original's formatting as well as the
   candidate's; sensitivity samples drawn from original and candidate alike.
3. Known E3 apparatus facts for any follow-up: the T5 drivers must use the
   E1c-2 CrossHair launcher (audit wall blocks an import-time hostname call);
   Hypothesis refuses `@given` on functions with defaults; `run_e3` writes
   bytecode into the mutant copy unless `PYTHONDONTWRITEBYTECODE=1`; the
   memory guard (`wait_for_idle(1800, 0.35, 8 GB)`, run if free RAM above
   6 GB) is a scratch script, not apparatus.
4. A corrected edge census (E2b) if a measured selector is still wanted:
   distinct codec sites, size-normalised measures, minimum executed size,
   worker-independent attribution.
