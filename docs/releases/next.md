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
- After a run that could not judge, the advice names running with fewer copies (`--workers 1`) for
  a machine that ran short of memory.

## Known limits

- When a working copy stops at its first build or test run, the tool keeps the last 400 characters
  of the output and does not say whether the build or a test failed. The page shows that text
  behind a line that opens. Three copies of a large Rust project building at once ran out of
  memory on a machine with 8 GiB free; the tool does not choose the number of copies from the
  memory that is free.
- The page was checked in Chromium, and earlier in Edge. Firefox, Safari and a screen reader were not tried.
- `--open` on Linux starts `xdg-open`, on macOS `open`. macOS was not run.
