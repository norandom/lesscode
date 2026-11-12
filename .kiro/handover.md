# Handover: release 1.2.0 is prepared and waits for the tag

Reset 2026-11-11, 20:46 Europe/Berlin (from the clock), at the end of the day's session. This file is the
start for the next session. The file as it stood until today, with its dated history back to the first
experiments, is [handover-archive-2026-11-11.md](handover-archive-2026-11-11.md); nothing in it was changed.

## Start here

1. Read [task 43 handover](task_43_handover.md): what the release contains, what the docs audit corrected,
   and the mistake with a wrongly built binary that R93 records.
2. Then the latest research entry,
   [R93](../docs/research/2026-11-11/19-refused-pinned-accepted.md), and
   [docs/releases/1.2.0.md](../docs/releases/1.2.0.md).

## State

- `main` holds release 1.2.0, prepared and not tagged. Version 1.2.0 is in `native/Cargo.toml`, both lock
  files, `CITATION.cff`, `.zenodo.json` and the README badge.
- CI on five platforms: success for `db175e5` (run 43087717445) and for `3710db7` (run 43088660633). The
  commits after `3710db7` change documents only; look at their run before tagging.
- `gradual-refactoring-steps`: every task is ticked. Four real steps are recorded (R92, R93): one in this
  repository (Rust, merged as `63d456f`), one in commons-csv (Java), two in OpenUltraCode (TypeScript), one
  of them refused, pinned and then accepted.
- Driver fix `aa9d7d7`: a working copy that is used again follows its source. Releases 1.1.0 and 1.1.1
  have the fault; the release notes say so.
- README and `docs/steps.md` present Python, TypeScript, Java and Rust alike (owner request). No step has
  been taken in a Python project yet, and both documents say so.
- openidea-kiro is the owner's co-project (`../openidea-kiro`). The evidence file of a step is provisional;
  its target is the `RunRecord` of that project's spec `verification-evidence`
  ([alignment note](specs/gradual-refactoring-steps/evidence-alignment.md)).
- Slice B (drift hardening) is closed. No model was called today; model spend stands at about 32.9 of
  40 USD. No container of this project is running.

## For the owner

1. Tag `v1.2.0` on a commit with green CI; the steps are at the end of `docs/releases/1.2.0.md`. If the tag
   is not pushed on 2026-11-11, set `date-released` in `CITATION.cff` to the day of the tag first.
2. After publication: signatures, checksums, attestation, Zenodo, as for 1.1.1; then record the result in
   `docs/releases/1.2.0.md`.

## Open work, in the order it seems worth doing

1. A real step in a Python project, so that all four languages have one.
2. `tools/adopter_check.sh` is written for 1.0.x and probes a file that is gone. Bring it up to date for
   1.2.0 (it should run `deconvolute run` and a step) or retire it.
3. Not shown on a real step: an override, a release entry on the ledger, a frame specification, a function
   marked out of reach, and a push of a step branch through the installed hook.
4. Per-test results in the evidence file, so that openidea-kiro's `breadth accept` can take it. This waits
   for `verification-evidence` to be implemented there and needs a spec here.
5. `step guard` as a required job in this repository's own pipeline (`.dagger/`, `native-core.yml`).

None of these has a spec or the owner's approval yet. Each needs a decision before work starts.

## Owner decisions, 2026-11-12

Objective: use the tool on real code ourselves and record what happens, then write an onboarding skill
that speaks to a developer at their level. The open work above is closed first, as dogfooding, each item
with a dated research entry. Item 4 stays deferred. The new spec starts only after that.

Order for the open work: item 1 (Python step), item 3 (the five things not shown on a real step), item 5
(`step guard` in our own pipeline), then item 2 (`tools/adopter_check.sh`), which needs the published
1.2.0 release because the script installs from a release. CI for `eec7262` is green (run 43090409481).

For the onboarding spec (not started; run `$kiro-discovery` after the open work):

- Entry is `/lesscode start`, shipped as a plugin. It asks how familiar the developer is, 1 (vibe coding)
  to 5 (professional or academic software engineer), and sets what is offered and the wording.
- Level 1: point to other tools, for example `/plugin install code-simplifier@claude-plugins-official`.
  Verify the install line when the skill is written.
- Level 2: show commands only; the code is never changed. The agent may run the commands that produce
  output; the developer stays in the agent tool. Nothing about the model-driven part (DSPy).
- Level 3 and up: the flow is menus and the developer chooses. The model-driven part starts here.
  Level 3 uses a pre-commit hook. That hook does not exist; today there is a pre-push hook only.
- Level 4: large refactoring; the check runs in CI or before a push. Developers do not have minutes to
  wait locally, so CI is the likely place. To be designed.
- Level 5: every choice is offered.
- Wording: levels 1 and 2 hear nothing about instrumenting code or solvers; levels 4 and 5 do.
- "Pex and Moles" is dropped as a name. It meant mainly the check step, where the code is instrumented.
  The owner's working name is the "nobreak gate". No current guide uses a name for it yet.

## Rules that caught someone out today

- `cargo build --examples` does not rebuild the executable. Build with `--bins`, and copy the executable
  before a long run: a running `step check` locks it, and a later build silently leaves the old one.
- Do not reuse the work directories `work-shape` and `work-replay2` under
  `D:\BuildArtifacts\lesscode\steps-foreign`; they hold outcomes stored by runs without the fix. The other
  work directories there, and `D:\BuildArtifacts\lesscode\step-one-exit`, can be deleted.
- A project's own install script can set `core.hooksPath` in a clone (OpenUltraCode's does).
- Times in documents come from the clock, not from an estimate.
- Before pushing README or guides, run them through the humanizer skill (owner, 2026-11-11).
