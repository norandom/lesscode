# Implementation Plan: the run page

Generated 2026-11-23, 15:35 (Europe/Berlin). **Approved by the owner on 2026-11-23** ("approved, push main and start with group 1").
Basis: [requirements](requirements.md) (approved 2026-11-23), [design](design.md) (approved
2026-11-23, with the section "Decided by the owner"), [prototype gate](prototype-gate.md) (run five
times; assessed as proceed with the event stream; row P2 failed and decided the way the data
travels).
Task-graph sanity review: inline (no subagent, by the owner's rule), verdict PASS, with one note:
group 6 depends on a fact about the driver that task 2.1 has to establish first.

Rules for every task group (owner rules of 2026-11-07, 2026-11-10 and 2026-11-23):

- Foreground execution; one commit per completed task on main; Rust tests with cargo-nextest; format
  check and lints before every push. Rust only: no Python and no JavaScript tooling. The page and
  its library are files that the Rust build takes in as text.
- A run without `--page` must behave exactly as before. Every group ends with the existing suites
  passing unchanged, on Windows and through Dagger on Linux.
- Each group ends with the budgets check passing for every new file at the default ceilings (2,800
  tokens, 300 units, 560 tokens per function), a dated research entry where a result or a deviation
  exists, and `.kiro/task_N_handover.md`. A file over budget is split, not exempted.
- The page explains and shows. No task may let it change an exit code, a receipt or a verdict.
- Tasks marked "can fail" carry something the gate did not show. A failure is recorded and goes
  back to the design before the group is closed.
- What is reported about a task is what ran and was seen. Times come from the clock.

## Tasks

- [x] 1. The snapshot
- [x] 1.1 The value the page shows, and how it is built from what a run tells
  - Done 2026-11-23: `native/src/adapters/driver/page_snapshot.rs` and `page_log.rs`, tests in
    `native/tests/run_page_snapshot.rs`. Not a module of its own at the top of the crate, as the task
    says: `lib.rs` is at its frozen ceiling, and a directory under the driver breaks a guard test
    that reads the driver's directory as flat. So the two files sit in the driver beside the runner.
  - A module of its own in the core: the snapshot with `schema: 1` as in the design, and a log that
    takes events (a stage begins or ends, the unchanged tests ended in a copy, a fault was classed,
    a copy dropped out, the run ended) and keeps the snapshot. No input or output in it.
  - Counted stages carry done, total and the time for the rest, which is given only after five
    items; uncounted stages carry the time gone.
  - Done when tests build the snapshots of a short run, a run with a dropped copy, a void run and a
    run without a test command, and each matches a written expectation.
  - _Boundary: Snapshot, RunLog_
  - _Requirements: 3.1, 3.2, 3.5, 3.6, 3.8, 6.2, 6.3_
- [x] 1.2 What the run says about the person's code
  - Done 2026-11-23: `native/src/adapters/driver/page_subject.rs` asks Git with three read-only
    commands; tests in `native/tests/run_page_subject.rs` on temporary repositories. The copies, the
    commands and the files written are set on the log by the run; filling them from a real run's
    configuration is group 2.
  - The snapshot's `subject`, `copies`, `commands` and `writes_in_repository`, filled from the run's
    configuration and from Git: repository, branch, commit, uncommitted changes, whether the branch
    is the main branch.
  - Done when tests on a temporary repository show each of these, including a dirty tree and the
    main branch.
  - _Boundary: Snapshot_
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [ ] 2. `step plan` tells the log
- [ ] 2.1 The driver reports its stages and each classed fault (can fail)
  - Where the batch runner prints progress or adds to its receipt, it also tells the log, behind
    one trait, so that a run without a page has an empty implementation.
  - Establish whether the function of a fault is known when it is classed. If it is, the snapshot
    carries the tally per function while the run goes on. If it is not, say so in a research entry;
    the tree then shows files during the run and functions at its end, as in the prototype.
  - Done when a driver suite runs with a recording log and the recorded events match the receipt:
    same faults, same classes, same dropped copies.
  - _Boundary: Driver, RunLog_
  - _Requirements: 3.1, 3.3, 3.5_
- [ ] 2.2 A sign of life during stages that count nothing
  - A timer that marks the snapshot at least every five seconds while the tool lives, also while a
    copy builds.
  - Done when a test with a command that sleeps shows marks no more than five seconds apart, and
    none after the run has ended.
  - _Boundary: RunLog_
  - _Requirements: 3.4, 3.7_
- [ ] 2.3 Unprotected functions, and what they reach out to
  - At the end of `step plan` the snapshot carries, per function, whether no fault was noticed and
    what the plan already lists as reached outside it.
  - Done when the snapshot of a planned fixture names the same unprotected functions as the plan.
  - _Boundary: Snapshot, Plan_
  - _Requirements: 6.1, 6.5, 6.6_

- [ ] 3. Serving the run
- [ ] 3.1 A listener on the machine's own address, with the standard library
  - Bound to `127.0.0.1` on a free port for the length of the run. The address carries a secret.
    It answers three things: the page, the stream of snapshots, and nothing for anything else.
  - Each new snapshot goes down every open stream. A stream that the page closes is dropped
    without disturbing the run. The listener ends with the run.
  - Done when tests with a plain client show: the page and the stream with the secret; no content
    without it or with a wrong one; two streams at once; a client that goes away mid-run; and that
    a failing listener (port taken, send fails) never changes the run's exit code.
  - _Boundary: Server_
  - _Requirements: 1.2, 3.4, 3.7_
- [ ] 3.2 `--page` and `--open`
  - `--page <place>` writes the report there at the end and prints the place; without a place it
    goes beside the run's outputs. `--open` starts the default browser on the served page once, at
    the start, without a shell: the platform's own opener on Windows, macOS and Linux.
  - Refused: a place inside the repository's tracked files unless it is named explicitly. Never
    opened unless asked. If the browser cannot be started, the tool says so, prints the address
    and goes on.
  - Done when tests show each of these with a stand-in for the opener, and a run without either
    option produces byte-identical receipts to a run before this group.
  - _Boundary: Options, Opener_
  - _Requirements: 1.5, 2.1, 2.2, 2.3, 2.4, 2.5_

- [ ] 4. The page
- [ ] 4.1 The page and its library inside the binary
  - The page from the prototype as one file in the crate, taken in with `include_str!`; the view
    library as one file with its licence, its source address and its hash, named in the
    third-party notices. A test fails when the library's hash changes without the notice.
  - Done when a test shows the written page contains its style, its code and the library, names no
    address on another machine, and is under 150 KiB without data.
  - _Boundary: Page_
  - _Requirements: 1.1, 1.2, 1.3_
- [ ] 4.2 The six states, and drawing that cannot leave a blank page
  - As in the design's table. The page draws from the snapshot alone; every field may be missing.
  - Fix what the gate's pictures showed: the status line after the end; folding one file in the
    tree.
  - Done when the gate's fixed situations give their states with no failure, checked in a headless
    browser by a script kept beside the page, on every machine that has a browser. The script is
    not part of the unit tests, and the record says where it ran.
  - _Boundary: Page_
  - _Requirements: 3.1, 3.2, 3.6, 3.7, 1.4_
- [ ] 4.3 A width of 400, and a second browser (can fail)
  - The gate's browser would not go below 492. Find a way to see the page at 400, and run the fixed
    situations in one browser other than Edge.
  - Done when both were seen, or the record says what could not be seen and why.
  - _Boundary: Page_
  - _Requirements: 1.3, 1.4_
- [ ] 4.4 The toolbar, the keyboard and a screen reader
  - Every control reachable and usable by keyboard, with a visible focus and a name; no meaning
    carried by colour alone; reduced motion respected.
  - Done when a written walk through the page by keyboard is in the research entry, with what
    worked and what did not.
  - _Boundary: Page_
  - _Requirements: 1.4_

- [ ] 5. What the page explains
- [ ] 5.1 Your code
  - The tab and the top of the page say what the tool touches, from the snapshot of task 1.2, with
    the warning on the main branch and, at the end, the space the copies take and how to remove them.
  - Done when the fixed situations show each sentence only where its fact is in the snapshot.
  - _Boundary: Page_
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_
- [ ] 5.2 What a planted fault is, with one from this run
  - The explanation, the first classed fault as the example, what a noticed fault shows and does
    not, and the three readings of an unnoticed one.
  - Done when the text is in the page, the example appears with the first classed fault, and the
    owner has read the wording.
  - _Boundary: Page, Snapshot_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.6_
- [ ] 5.3 Faults in code the platform does not compile (can fail)
  - The tool does not know this today. Find out whether the language packs can mark a site that
    lies under a condition of compilation, for Rust first. If they can, such faults are counted
    apart and shown apart. If they cannot, the page says nothing about it and the research entry
    says why; no guess is shown.
  - Done when the renderer of the recorded run shows its 64 such faults apart, or the entry records
    the failure.
  - _Boundary: Pack, Snapshot, Page_
  - _Requirements: 5.5_
- [ ] 5.4 Without tests, nothing is protected
  - The notices for no test command, for tests that do not pass on the unchanged code, and for a
    copy that drops out; the share of unprotected functions before any other result; the sentence
    on code that needs a device.
  - Done when each fixed situation shows its notice and no other.
  - _Boundary: Page_
  - _Requirements: 6.1, 6.2, 6.3, 6.5, 6.6_

- [ ] 6. `step check` and the report
- [ ] 6.1 `step check` tells the log
  - Its stages: the checks that need no run, the unchanged tests on the base and on the step, the
    faults on both sides, the verdict. A refusal before any run ends the page at once, with the
    reasons.
  - Done when a refused and an accepted fixture step each give the snapshot expected for them.
  - _Boundary: Step, RunLog_
  - _Requirements: 3.1, 6.4, 7.7_
- [ ] 6.2 The report: the page with the data inside
  - Written once when the run has ended, readable without the tool. For a judged step it shows the
    lenses the tool has: size, protection per function, behaviour kept. Structure is measured with
    `deconvolute measure-scope` on both sides. State and local complexity are marked as not
    measured unless a file with those numbers is handed in.
  - Each lens with its expected direction and whether it moved that way; the tool's version, the
    commands and the commits; names and no source text unless asked.
  - Done when the report of the fixture steps shows each lens or "not measured", and a refusal as
    prominently as an acceptance.
  - _Boundary: Report writer, Page_
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.7, 7.8, 7.9_
- [ ] 6.3 The drawing of functions and calls, and the grid of shapes
  - The two drawings of the page "Two Bowls", from the snapshot: functions and calls before and
    after to one scale; per type the shapes that can be written, are reached and are seen. Each
    only where its numbers are in the snapshot.
  - Done when the report built from the numbers of the first run on `SharedFrame` shows both, and a
    report without those numbers shows neither and says "not measured".
  - _Boundary: Page_
  - _Requirements: 7.5, 7.6_

- [ ] 7. Several reports
- [ ] 7.1 A list of reports
  - A subcommand that reads the data inside several reports and writes one page that lists them,
    links to each, and shows refused and worsened ones like the others; with an option that leaves
    the names of files and functions of a named project out.
  - Done when a list of three fixture reports, one of them refused, shows all three alike, and the
    option removes every name of the named project from the list and from its report.
  - _Boundary: Gallery writer_
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [ ] 8. Closing
- [ ] 8.1 A real run, watched
  - `step plan --page --open` on a subsystem of a real project, with the owner looking at the page
    while it runs.
  - Done when the research entry says what the owner saw, in the owner's words, and what was
    changed because of it.
  - _Requirements: 3, 4, 5, 6_
- [ ] 8.2 The documentation
  - The guide gets a section on the page; `docs/steps.md` the two options; the skill
    `untangle-state` a line telling an agent to open the page for the person. The owner's wish of
    2026-11-23: recommend `https://github.com/jamesaphoenix/diff-core` in the documentation, as
    something that should suit developers. Read that project first and write what it is for in one
    sentence that is true of it; the wording is the owner's to approve. A humanizer pass before the
    push.
  - Done when the owner has read the section.
  - _Requirements: 2.1_
- [ ] 8.3 Validation and release notes
  - The suites on Windows and through Dagger on Linux; budgets; lints; the opener and the listener
    on Linux in a container (the opener with a stand-in); a validation report; an entry for the
    next release notes.
  - Done when the validation report says GO or names what stands against it.
  - _Requirements: all_
