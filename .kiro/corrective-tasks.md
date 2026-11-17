# Corrective tasks after the audit of 2026-11-17

Written 2026-11-17, 22:10 Europe/Berlin (from the clock). Source: [R134](../docs/research/2026-11-17/21-audit-of-port-and-reduction-claims.md)
and [R133](../docs/research/2026-11-17/20-model-simplification-closed.md). Generated at the owner's
request; **not approved**. Nothing here is started.

Rule for every task: it is done when the named command ran on the real code and its output is kept.
A check that ran on functions written inside a script does not count. `(P)` marks tasks that can run
side by side. **Owner** marks a decision that only the owner can take.

## 1. Finish the audit, so the extent is known

- [ ] 1.1 (P) Rerun E6's tests on its candidate
  - `docs/experiments/deconvolution/e6-report-envelope/run_e6.py` on `.runs/e6-report-envelope/cand-e6`.
  - Done when: the pass count is recorded beside R45's "46 of 46", or the reason it cannot run.
- [ ] 1.2 (P) Audit E3 and E4 the way E5 was audited
  - Read `e3-evidence-regeneration` and `e4-lifecycle`: for each reported figure, find the code that
    produced it and say whether it ran on the candidate. Rerun what still runs.
  - Done when: one table with a row per figure (reproduces, not supported, not rerun).
- [ ] 1.3 (P) Audit the three adapter specs against their requirements
  - `typescript-deconvolution-adapter`, `java-deconvolution-unification`, `rust-deconvolution-unification`
    say `completed`, `GO`. R75 showed their gates' controls were not real; their operators were deleted in
    `964921c`. Check what of each spec's requirements exists today (`native/src/adapters/{java,rust,typescript}/mod.rs`,
    one file each; `native/adapters/` holds candidates and probes).
  - Done when: a finding per requirement, as in `native-deconvolution-closure/spec.json`.
- [ ] 1.4 Sweep the other specs that claim completion
  - `deconvolution-loop`, `deconvolution-native-core`, `capability-compression-core`,
    `efficiency-baseline`, `dogfood-self-application`, `refinement-evidence-integrity`.
  - For each validation claim: is there a command, did it run on real inputs, can it run today?
  - Done when: one table; specs with unsupported claims get the same note and status as the two of R134.
- [ ] 1.5 Search for literal results in gate and evaluator scripts
  - Every `run_gate.py` and `*_evaluator.py` under `.kiro/specs/` and `docs/experiments/`: values such
    as `"killed": True`, `"cases_evaluated": 21 + 8`, metrics typed in by hand.
  - Done when: a list of file, line and what the literal stands in for.

## 2. Put real checks where the unsupported ones stood

- [ ] 2.1 Planted faults against E5 and E6 with the released driver
  - `lesscode-core deconvolute run` on the baseline and on each candidate, same pack, same tests.
  - Done when: noticed and unnoticed faults per function, baseline beside candidate, for both trials.
    This replaces "100% adverse mutant kill rate".
- [ ] 2.2 (P) Measure functions, call edges and write sites again
  - The module that produced "69 to 50 functions, 89 to 61 edges, 12 to 11 write sites" is deleted. Use
    `deconvolute analyze` for functions and write sites; say plainly if call edges cannot be measured
    with what exists.
  - Done when: the figures stand beside R42's and R45's, or are marked not reproducible.
- [ ] 2.3 A bounded equivalence check that runs on candidate code
  - **Owner:** this is the first prototype of the spec `state-consolidation`, not a repair of the old
    port. It needs a decision on how Rust tooling runs Python functions of a specimen (the project
    allows no Python tooling of its own), and Z3 as an external program.
  - Done when: one changed pure function of E5 is checked, baseline against candidate, over stated
    bounds, with a planted difference found as a control.

## 3. Remove what pretends

- [ ] 3.1 **Owner:** delete or wire the modules of the port
  - `state_frame.rs`, `pure_verifier.rs`, `stream_monitor.rs`, `mutant_generator.rs`, `token_guard.rs`,
    `state_contract.rs`: no command reaches them. Recommended: delete them with their tests, and keep
    the helpers of `specimen_runner.rs` that the driver uses. Anything kept gets a caller.
  - Done when: the suite passes, the budgets file has no entry for a deleted file, and
    `lesscode-core --help` describes nothing that is gone.
- [ ] 3.2 (P) Remove candidates and probes from the source tree
  - `native/adapters/{java,rust,typescript}/` holds candidate files and a compiled `ProbeMain.class`
    that are part of no build.
  - Done when: they are under `docs/experiments/` or deleted, and nothing references them.
- [ ] 3.3 Correct the steering files
  - `.kiro/steering/roadmap.md` lines 161 to 163 still say the three adapters are "completed with GO"
    with "100% adverse sensitivity". `product.md`, `tech.md` and `structure.md` are to be read for the
    same claims.
  - Done when: steering states what R75 and R134 found, in one place each.

## 4. Correct the record without erasing it

- [ ] 4.1 Dated amendments in the entries that carry the claims
  - R42, R43, R45, R46 (E5, E6) and R49 to R53 (gates): a short amendment at the end of each that
    names what does not stand and links R134 or R75. The original text stays.
  - Done when: each entry and its line in the research index point to the correction.
- [ ] 4.2 (P) Check what the published releases say
  - `README.md`, `docs/*.md` and the release notes were searched on 2026-11-17 for the E5 and E6 claims:
    none found; the "4 of 4" figures were withdrawn in release 1.1.0. Still to read: `.zenodo.json`'s
    description ("accepts only on zero divergence", "searches the pure core for differences") and the
    reports under `docs/reports/` other than the one already marked.
  - Done when: a list of sentences to change in the next release, or the statement that there is none.

## 5. Keep it from happening again

- [ ] 5.1 A rule for validation claims, in `AGENTS.md`
  - A claim of "verified", "covered" or "GO" names the command, the real input paths and the kept
    output. Gate scripts hold no literal results. A spec is `completed` only with a command that can be
    run again.
  - Done when: the owner has approved the wording and it is in `AGENTS.md`.
- [ ] 5.2 A test that fails on code no command reaches
  - In the core's suite: every module under `native/src/` is reachable from a command, or is listed
    with a reason. The port's modules would have failed it.
  - Done when: the test passes on main after 3.1 and fails when an unused module is added.
- [ ] 5.3 A check of spec status against evidence
  - Extend the pipeline's spec audit: a `spec.json` that says `completed` or `GO` must name an evidence
    command; one without is a warning that lists the spec.
  - Done when: it lists today's cases and nothing else.

## 6. Decisions that are only the owner's

- [ ] 6.1 **Owner:** push or hold the 30 local commits of 2026-11-17 (all documents, specs and
  experiment files; `origin/main` is at `a5df503`).
- [ ] 6.2 **Owner:** whether the corrections of group 4 go into a release 2.0.1 with a note, since
  Zenodo archives each release.
- [ ] 6.3 **Owner:** which agent may work on this repository, and whether validation of one agent's
  work is always done by another. The validation report of `state-boundary-deconvolution` names
  "Antigravity" as its auditor; the port's commits name no agent.

## Order proposed

1.5 and 1.1 first (an hour, they bound the damage), then 2.1 (the one real check that is missing and
that the released tool can do today), then 3.1 and 5.2 together, then the rest of group 1. Group 4
follows group 1. Task 2.3 starts with the requirements of `state-consolidation`.
