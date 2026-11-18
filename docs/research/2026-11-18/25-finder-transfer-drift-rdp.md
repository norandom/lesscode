# R159: The frozen finder transfers to a third repository: eight of nine machines, and none of eleven reports wrong

- Report date/time and timezone: 2026-11-18, 16:30, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, census sealed and registered at 16:29, scoring written at 16:30, from the clock
- Status: completed; passed as registered
- Spec: [state-consolidation](../../../.kiro/specs/state-consolidation/brief.md), stage 1 (find); brief only, no requirements yet
- Hypothesis ID and predecessor: the transfer test asked for by [R158](24-finder-follows-names.md)
- Type: preregistered in the [prototype gate](../../../.kiro/specs/state-consolidation/prototype-gate.md), commit `b62ac40`, with the census sealed before the finder ran

## Decision

Revision 3 of the finder, frozen, ran on `drift-rdp` at `7eb659e`, a repository it had never read. The
answer was fixed beforehand in a hand census of 24 cells of the application code.

| Measure | Registered bar | Result |
| --- | --- | --- |
| Cells with exactly the census's access lines | at least 16 of 24 | 21 of 24 |
| Census machines reported as machines | at least 6 of 9 | 8 of 9 |
| Machines reported in application code that are wrong | at most 1 in 3 | 0 of 11 |
| Verdict right, of the cells with a verdict | no bar | 20 of 22 |

The find step transfers on this repository. Three cells are missed, for two reasons that are one word
and one rule each: `try_lock` is not among the methods the rules know (`operation` and `pending` in
the clipboard and the updater), and a `static` is not a cell (`REGISTRY`). The missed machine is
`pending`: installing an update fails unless a check stored one.

The eleven machines reported are real ones. Among them the clipboard lease (`owner`,
`claim_sequence`, `valid`), the credential check's revisions and publication, the frame's `dirty`
flag, the identity drafts, the codec policy's terminal error, the session manager's deleted
connections, and the library store's ready or write-blocked state.

What the finder cannot see is larger than what it missed: the session's own state is a plain field,
`state: SessionState`, assigned in about twenty places and tested in four. It is the main state
machine of that application and is not in a cell.

## Hypothesis and falsification

Registered bars as in the table. Predictions written before the run:

| Prediction | Result |
| --- | --- |
| `REGISTRY` is not listed | held |
| `pending` is not reported as a machine | held, for another reason than given: `try_lock`, not `ok_or` |
| `drafts` is not reported as a machine | **wrong**: reported, and rightly; the conditions at 168 and 173 test the draft that was taken out |
| `owner`, `claim_sequence`, `valid`, `revision`, `publishing`, `dirty` are found | held |
| The plain `state` field is not seen | held |
| Lines complete for at least 18 cells | held: 21 |

## Frozen inputs and method

- `D:\Source\drift-rdp` at `7eb659e`, 59 Rust files, clean, only read (`git archive`).
- Finder binary `6af3a9d1…ec85bc8` and rules `3f156fdb…fd6b30ae`, as committed in `32fc798`.
- Census by text search and reading, sealed with SHA-256 `3bdb3ac3…e75eba` in commit `b62ac40`.
  Scope: application code under `src-tauri/src` outside test code. Left out beforehand: the probe
  program, the tests, `store/library.rs` (test hooks between the store's methods) and cells that exist
  only under `cfg(test)`. Two cells have accesses but no verdict in the census.
- A census machine is one where a unit changes the value and another unit's behaviour depends on it by
  any means, which is wider than the finder's conditions.
- Lines in test code are not compared.

## Evidence and results

[Run directory](D:/BuildArtifacts/lesscode/state-consolidation/r159): `census.md`, `finder.json`,
`scoring.md`, `files.txt`, the source copy.

- 59 of 59 files parsed. 60 listings: 14 machines, 13 shared without a condition, 33 in one unit.
- Three machines reported outside the judged census were read and are right (`codec_policy.rs:340`,
  `session.rs:2280`, `library.rs:1173`).
- Wrong detail lines, no verdict changed: `library.rs:1337` and `1341` (a `match` on a load result
  that mentions the bound name, the over-count already seen in R158) and `credential_check.rs:185`
  (a `let … else` that tests a timeout around a gate without a value).
- Two machines reported in test code (`connect_calls`) were not read.

## Limits and interpretation

- The census maker wrote the finder. The census was sealed first, but both share one reader's idea of
  what a cell is. A census by someone else would be the stronger test.
- One more repository, one language. Three repositories in all, each of the owner's.
- The census left `library.rs` out for cost, and the finder's report there was read afterwards, not
  predicted.
- State that is not in a cell type is outside the finder: on this repository that is the largest
  machine. The same holds for `judged` in this repository's guide adapter.
- The bound-name rule over-counts condition lines.

## Tool/spec consequence and next gate

1. Stage 1 has what the prototype gate asks for on real code: a registered pass on a repository not
   seen before. Requirements and a design for the find step can be written; that is the owner's phase
   decision.
2. Two small rule changes are indicated and not made: `try_lock` among the methods, statics as cells.
   Making them needs a rerun of all three censuses.
3. The open research question is state outside cells: plain fields that hold an enum and are assigned
   in many places. A second kind of rule, starting from a field whose type is an enum and whose value
   is tested, would be a new experiment.
4. Queued as before: `fatal` and `queue` in the driver's runner as the second case for stage 2; the
   `match`-arm prototype for the Rust pack.

## COBESY check

The bars were set before the run and the table shows them beside the results. The wrong prediction is
marked. The largest thing the tool cannot see is in the decision, not in the limits alone. No count of
lines or tokens is used as a measure.
