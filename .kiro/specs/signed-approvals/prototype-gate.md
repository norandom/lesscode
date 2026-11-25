# Prototype gate: signed approvals

Written 2026-11-25, about 14:10 (Europe/Berlin), after the runs. Record:
[R192](../../../docs/research/2026-11-25/03-signed-approvals-prototype.md).

**Decision: proceed.** All five cases on the real stage and all adverse controls behaved as
expected. One thing is left for the owner's setup: the passphrase prompt in a real terminal.

## Why a gate here

This is not a compression mechanism, so steering's usefulness targets (net reduction) do not
apply. The gate checks the design's riskiest assumptions instead: that the check needs no new
dependency, gives the same verdict on Windows and Linux, ignores the host's Git settings, and
works on a real stage's history.

## Frozen inputs

- lesscode `380eb5f`; scitools-hook stage branch `protect/protect-cycles` at `b885391` (copied,
  never changed in place); the forge answer for `norandom` of 2026-11-25.
- OpenSSH 10.6p1 and 9.5p2 on Windows, 9.2p1 in `buildpack-deps:bookworm-scm` (the CI image's
  base). Git 2.56.0.windows.3 and 2.39.5.

## Behaviours and limits

| # | Behaviour | Limit for proceed | Result |
| --- | --- | --- | --- |
| G1 | A good signature verifies and names its key's fingerprint | on both platforms, fingerprint equals `ssh-keygen -l` | met |
| G2 | A changed payload, or the namespace `file`, does not verify | on both platforms | met |
| G3 | An unsigned commit is seen as unsigned | on both platforms | met |
| G4 | No host Git setting is needed | no `gpg.*` config in the runs | met |
| G5 | The forge lists signing keys without a token; unreachable and unknown account are errors | 200 / non-zero exit | met |
| G6 | On a real stage, the commit that last changed one write-off is found | R4 names `421c1ef`, R5 names the later commit | met |
| G7 | The four refusals of requirement 1.3 and one count, each in its own case | R1 to R5 | met |
| G8 | A passphrase key cannot sign without a terminal | commit refused | met |
| G9 | The owner can sign with the passphrase in a terminal | owner's first signing | **not run** (owner's setup) |

## Stop conditions

Stop and revise if G1 to G7 differ between platforms, or if any adverse control verifies. Neither
happened. If G9 fails on the owner's machine (no prompt through Git on Windows), the design keeps
its check and the guide changes how the owner signs, for example with `ssh-keygen -Y sign` run
directly. The check itself would not change.

## Budget

Ten minutes of runs, 13 MB for the copy of the stage, one image of 515 MB
(`buildpack-deps:bookworm-scm`), to be removed when the tasks no longer need it.
