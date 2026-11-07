# Requirements amendment review, 2026-11-07
Draft review passed; not phase approval or feature GO.
Numeric requirements 1-6 use EARS criteria and explicit inclusion/exclusion.
Independent reviewer challenged active-document contradictions, bootstrap cycles,
baseline parity, cumulative attempt accounting, normalization and S9 ratification.
Controller incorporated those corrections before writing requirements.
Targets remain proposed; the frozen plan needs approval before execution.
No prototype or production code has been run. Document validation must check local
links, structured files and locked approval state before claiming this edit done.

Final independent review identified missing explicit candidate/turn demonstrations
and an ambiguous S9 ratification prerequisite. Both were repaired in U2/U6 and
the preregistration section before completion. Structured/link/locked-state checks
passed; repeat after these repairs. This remains documentation verification only.

## Amendment 2 review (2026-11-07, library and complexity audit)
Empirical audit of the E3/E4 baseline under pinned `ruff==0.17.3 format --line-length 100`
and inspection of `native/src/` and `native/Cargo.lock` identified three plan blockers:
1. Canonical formatting expands the 14 baseline apparatus files from 2,222 as-written
   lines to 3,963 canonical lines (`e1b_oracle.py`: 406 -> 659; `e4_oracle.py`: 534 -> 1,258),
   where 1,498 canonical lines in `e1b_oracle.py` and `e4_oracle.py` are irreducible
   scope-specific `op_*` handlers, fixtures, `SEEDS` and generator strategies.
2. Charging existing `native/src/lib.rs` (618 lines) or verbatim-migrated `op_*`/`SEEDS`
   blocks (1,498 canonical lines) in full against `U7` (600 lines) would deadlock
   prototype execution before writing a single new line of logic.
3. In `research.md`, `serde_jcs` breaks Python float/ASCII digest parity (`S2`),
   `quick-junit` cannot parse XML, and `fs4` is redundant on Rust 1.98 (`std::fs::File::lock`).

Repairs applied across `requirements.md` (Reqs 2.2, 2.4, 4.2, 4.5, 6.3, 6.5),
`prototype-gate.md`, `slice-proposal.md`, `spikes.md`, `research.md`, `complexity.md`
and `complexity-budgets.json`. EARS syntax and technology-neutral requirement wording
verified. Approval state remains locked (`approved = false`, `solution_unlock = not-run`).
