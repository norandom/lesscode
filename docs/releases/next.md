# Notes for the next release

Collected on 2026-11-24 for the release after 2.1.0. Nothing here is tagged or published; the
version number and the tag are the owner's.

## What is new

- **A page that shows a run.** `step plan` and `step check` take `--open` and `--page`. With `--open`
  your default browser opens on a page that follows the run while it goes on; it is served from
  your machine only and ends with the run. With `--page` the report of the run is written as one
  HTML file at the end, and nothing is served. The page comes inside the binary and loads nothing
  from the network. Guide: [refactoring in checked steps](../steps.md#watch-the-run-in-a-browser---page-and---open).
- The page says first that no file of your project is changed and where the copies are. Under that
  the steps of the run stand in one row, with the one that is going on marked. It explains a
  planted fault with one of the run, lists the functions in which no fault was noticed, and says
  what to do with them.
- For `step check` the page ends as a report: the outcome in one box, the same for accepted,
  refused and not judged; a tile and a row per lens with the direction expected; the functions and
  calls before and after. `--measures <file>` adds numbers of the state and local-complexity lenses
  that were measured elsewhere.
- `step reports <report>... --out <directory>` writes one page that lists several reports, with a
  copy of each. `--without-names-of <project>` puts numbers where that project's files, functions
  and types are named.
- `--page-source` shows the line of code of the example fault. Without it the page holds names
  and no source text.
- A language pack can name the code that is not built on the current system. Faults planted there
  are counted apart and the page says that they tell nothing.

## Changed

- A run without `--page` and `--open` writes what it wrote before and prints what it printed before.
- The copies of a run take turns by the memory that is free. The first build of a copy and the first
  build with a planted fault run alone once and are measured; after that a run starts beside others
  only when the memory holds them. Three copies of a Rust project that needs about 8 GiB for a full
  build ran to the end on a machine with 13 GiB free, where they had run out of memory before. With
  little memory the copies mostly run one after another, which is slower and gets through.
- A copy whose first run fails says whether its commands ended before any test ran or a test did not
  pass (this needs `runner_evidence_patterns` in the configuration), keeps the whole output in
  `first-run-w<copy>-<attempt>.log` under the work directory, and tries once more when no other copy
  is in its first run. A run in which no test ever ran is "not judged" for `step check`, where it
  was "the project's tests do not pass" before.
- The receipt of a run names a copy's first attempt under `retried` where a second one was made.

## Known limits

- How much memory a run takes is read from the whole system while the run goes on alone. Another
  program that takes memory at that moment makes the number too large, and the copies then wait
  more than they need to. The number of copies is not lowered; they wait for their turn.
- On macOS the system is not asked how much memory is free, and nothing is held back.
- The page was checked in Chromium, and earlier in Edge. Firefox, Safari and a screen reader were not tried.
- `--open` on Linux starts `xdg-open`, on macOS `open`. macOS was not run.
