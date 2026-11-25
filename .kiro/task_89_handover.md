# Handover 89: brownfield test stage, group 7 done (keyword-argument swaps, time figures), validation GO

Written 2026-11-25, about 13:55 Europe/Berlin. Predecessor: [task_88_handover.md](task_88_handover.md).
Spec: [brownfield-test-stage](specs/brownfield-test-stage/tasks.md), every task ticked again.
Validation: [validation.md](specs/brownfield-test-stage/validation.md), addition for group 7,
**GO**. Record: [R191](../docs/research/2026-11-25/02-begin-time-figure.md) (retrospective,
completed), with a dated amendment to [R190](../docs/research/2026-11-25/01-protect-stage-python.md).

## What was done

- **The owner's decisions on handover 88**: "accept 2 and 3, keep 5, start the task group".
  `.dagger/main.go` `38d337e` is kept as it is. Amendment of the design and group 7 in `ada2bb6`;
  requirements unchanged (5.9 holds).
- **7.1 (`ad0b3f1`), no swap of keyword arguments.** A pack's `call_argument` operators gain
  `named_kinds`; `ArgumentOps::swappable` in `pack.rs` decides; admission checks the kinds. The
  Python pack names `keyword_argument`. R190's `cycles.py` at `73a736e`: 21 faults where the plan
  had 22, without `arg_swap@138:8`.
- **7.2 (`bc5770d`, `719ee6b`), `begin`'s time figure.** A run takes the tests without a fault
  three times (first copy alone, the others after it, all again at the end) and the first fault
  alone. Base run: `3T + S/n + (S - S/n)/c` (`2T + S` with one copy); the check: `3T` more, since
  on the base it takes the base run's stored results. Both printed, each with the room of 1.4. On
  the frozen plans of R188 to R190 the base runs took 0.91 to 1.00 of the new figure (1.32 to 1.73
  of the old), the checks 0.45 to 1.05 (up to 2.7 of the old). `719ee6b` moved the figure check
  into its own test after `kiro-cli`'s ratchet refused a test of 81 lines.
- **7.3 (`add72d0`), the guides.** `docs/driver.md`, `docs/steps.md`, the skill's phases 5 and 8,
  the 2.2.0 notes (also: the stage was walked on Python too). Humanizer pass. The owner: "Accept as
  written".
- **7.5 (`8b41287`), owner's "Extend the rule now".** The swap is not planted when *either* of the
  first two arguments is of a named kind. A keyword argument put before a positional one parses
  but does not compile in Python, so the fault had counted as noticed for nothing. `cycles.py`
  unchanged at 21.
- **7.4, validation GO**: Windows 511 of 511, doctests, fmt, Clippy, budgets; Linux Dagger at
  `8b41287`, 23 steps 0, core 510 of 510, model 85 of 85; `kiro-cli code check` pass (3
  pre-existing errors).
- **`protect-cycles` approved** by the owner ("Approve both phases"): clone commit `b885391`;
  `step protect result` still exits 0.

## Decisions of the owner in this group, in their words

- "accept 2 and 3, keep 5, start the task group".
- Guides of 7.3: "Accept as written".
- Swap with a keyword argument second: "Extend the rule now".
- Approvals an agent cannot make: "Signed commits".
- `protect-cycles`: "Approve both phases".

## Open, for the owner to decide or to start

1. **Signed approvals (chosen, not specified).** `step protect result` is to accept an approval
   only in a commit signed with the owner's key, so an agent with shell access cannot write one.
   This needs a spec of its own: which signature (SSH or GPG, Git's `gpg.format`), where the
   allowed key lives (an allowed-signers file in the project, or the forge), what happens to the
   approvals already written by script, and how CI checks it. The release-signing key is also
   still waiting for the owner's setup ([release signing](../docs/release-signing.md)); one key
   setup could serve both. Start with `$kiro-spec-init`.
2. **Rust labels passed through a helper** are still planted (4 in `yuv.rs`). Accepted for now.
3. **Merging the protection tests** into drift-rdp and scitools-hook, and the **tag of 2.2.0**
   (push of the tag starts the release workflow): both wait for the owner's word.
4. **`D:\Source\lesscode\native\target`** holds 7.47 GiB, last written 2026-11-07, inside the
   source repository rather than under `D:\BuildArtifacts`. Lesscode's and inactive; not removed.
   `cargo clean --manifest-path native/Cargo.toml --target-dir native/target` would remove it
   with the tool's own cleanup, on the owner's word.

## Repositories and storage

- lesscode `main`: this handover's commit; `origin/main` at `9f6aab7`. Push only on the owner's
  word. Release 2.2.0 is written up (now with group 7) and not tagged.
- drift-rdp clone `D:\BuildArtifacts\drift-rdp\lesscode-brownfield-stage`, clean, on
  `protect/protect-shared-frame` (`f3469cc`). For 7.2 it was switched to `stage-base/yuv` and
  `stage-base/shared-frame` to run `begin`, and switched back; what `begin` wrote was removed.
- scitools-hook clone `D:\BuildArtifacts\scitools-hook\lesscode-stage`, clean, on
  `protect/protect-cycles` (`b885391`). Not merged.
- Evidence: `D:\BuildArtifacts\lesscode\brownfield-test-stage\` 0.13 GiB in all, new: `group-7\`
  and `validation-7.4\` (small). `D:\BuildArtifacts\lesscode\native-evidence\linux-8b41287` (and
  the two earlier runs of the group, `linux-add72d0`, `linux-719ee6b`, small).
- Docker image `lesscode-stage/scitools-hook-tests:2026-11-25` (664 MB) still kept for a repeat of
  R190. The Dagger engine container is the shared one and was left running. No other container.
- D: 190 GiB free; lesscode's disposable caches under the 40 GiB limit
  (`D:\BuildArtifacts\lesscode-cargo-test` 8.19 GiB, plus item 4 above).

## Next

The spec is finished again. The next step is the owner's choice: the spec for signed approvals
(open point 1), merging the protection tests, the tag of 2.2.0, or another spec.
