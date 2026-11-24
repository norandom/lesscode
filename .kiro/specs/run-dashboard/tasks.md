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

- [x] 2. `step plan` tells the log
- [x] 2.1 The driver reports its stages and each classed fault (can fail)
  - Done 2026-11-23. It did not fail: the function of a fault is known when it is judged, from the
    file's functions as they are before any fault (`FileFunctions::owner`), so the tree can show
    functions while the run goes on. `page_watch.rs` holds the events and one listener for the
    process; `step/plan_page.rs` turns them into the snapshot; tests in
    `native/tests/run_page_events.rs` compare what is told with the receipt, fault by fault.
  - Deviations. The runner tells three things, not every stage: the faults planned, the unchanged
    tests of a copy, each judged fault. Copying and the first run of the tests are one stage on the
    page, because nothing tells them apart while they happen. A copy that fails before its tests
    run is not told while the run goes on; it is in the receipt at the end. The listener is one
    for the process and not a parameter: `runner.rs` had 63 tokens left under its ceiling and
    now has 1, so the next change to that file needs it split first.
  - A class was added that task 1.1 lacked: a fault whose run failed twice and has no class.
  - Where the batch runner prints progress or adds to its receipt, it also tells the log, behind
    one trait, so that a run without a page has an empty implementation.
  - Establish whether the function of a fault is known when it is classed. If it is, the snapshot
    carries the tally per function while the run goes on. If it is not, say so in a research entry;
    the tree then shows files during the run and functions at its end, as in the prototype.
  - Done when a driver suite runs with a recording log and the recorded events match the receipt:
    same faults, same classes, same dropped copies.
  - _Boundary: Driver, RunLog_
  - _Requirements: 3.1, 3.3, 3.5_
- [x] 2.2 A sign of life during stages that count nothing
  - Done 2026-11-23: `Heart` in `page_watch.rs` and `PlanWatch::heart`; tests in
    `native/tests/run_page_heart.rs`. Deviation: the tests use an interval of 100 milliseconds on the
    log itself and measure against it, not a command that sleeps for more than five seconds. No
    command starts a heart yet; that comes with the options of group 3.
  - A timer that marks the snapshot at least every five seconds while the tool lives, also while a
    copy builds.
  - Done when a test with a command that sleeps shows marks no more than five seconds apart, and
    none after the run has ended.
  - _Boundary: RunLog_
  - _Requirements: 3.4, 3.7_
- [x] 2.3 Unprotected functions, and what they reach out to
  - Done 2026-11-23: `step plan` ends the snapshot through `PlanWatch::close` with the plan's own
    list of what each unprotected function calls, and through `PlanWatch::failed` when nothing could
    be judged. Tests in `native/tests/run_page_events.rs` compare with the written plan. The plan's
    arguments carry who listens; no option sets it yet (group 3). The plan's state writes per
    function are not carried into the snapshot, only its calls.
  - At the end of `step plan` the snapshot carries, per function, whether no fault was noticed and
    what the plan already lists as reached outside it.
  - Done when the snapshot of a planned fixture names the same unprotected functions as the plan.
  - _Boundary: Snapshot, Plan_
  - _Requirements: 6.1, 6.5, 6.6_

- [x] 3. Serving the run
- [x] 3.1 A listener on the machine's own address, with the standard library
  - Done 2026-11-23: `page_serve.rs`, tests in `native/tests/run_page_serve.rs`. The secret comes
    from the standard library's randomly keyed hasher, 32 characters. A stream looks for a new
    snapshot every 50 milliseconds. That a failing listener leaves the exit code alone is shown in
    the tests of task 3.2, where a run exists.
  - Bound to `127.0.0.1` on a free port for the length of the run. The address carries a secret.
    It answers three things: the page, the stream of snapshots, and nothing for anything else.
  - Each new snapshot goes down every open stream. A stream that the page closes is dropped
    without disturbing the run. The listener ends with the run.
  - Done when tests with a plain client show: the page and the stream with the secret; no content
    without it or with a wrong one; two streams at once; a client that goes away mid-run; and that
    a failing listener (port taken, send fails) never changes the run's exit code.
  - _Boundary: Server_
  - _Requirements: 1.2, 3.4, 3.7_
- [x] 3.2 `--page` and `--open`
  - Done 2026-11-23: `step/plan_show.rs`, `page_open.rs`, `page_text.rs`; tests in
    `native/tests/run_page_options.rs`. Only `step plan` has the options so far.
  - Deviations. (1) The page served and written is a plain stand-in that prints the snapshot; the
    page of the design comes with task 4.1. (2) The listener starts only with `--open`; `--page`
    alone writes the report and serves nothing. `--open` alone writes the report beside the plan.
    (3) Among a repository's files means: inside a repository and not ignored by Git. (4) A place
    whose directory does not exist is refused before anything runs. (5) Byte-identical is shown
    for the plan with and without the options in the same run directory, not against a binary of
    before this group. (6) The platform's own opener was not started by any test or by me.
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

- [x] 4. The page
- [x] 4.1 The page and its library inside the binary
  - The page from the prototype as one file in the crate, taken in with `include_str!`; the view
    library as one file with its licence, its source address and its hash, named in the
    third-party notices. A test fails when the library's hash changes without the notice.
  - Done when a test shows the written page contains its style, its code and the library, names no
    address on another machine, and is under 150 KiB without data.
  - _Boundary: Page_
  - _Requirements: 1.1, 1.2, 1.3_
- [x] 4.2 The six states, and drawing that cannot leave a blank page
  - As in the design's table. The page draws from the snapshot alone; every field may be missing.
  - Fix what the gate's pictures showed: the status line after the end; folding one file in the
    tree.
  - Done when the gate's fixed situations give their states with no failure, checked in a headless
    browser by a script kept beside the page, on every machine that has a browser. The script is
    not part of the unit tests, and the record says where it ran.
  - _Boundary: Page_
  - _Requirements: 3.1, 3.2, 3.6, 3.7, 1.4_
- [x] 4.3 A width of 400, and a second browser (can fail)
  - The gate's browser would not go below 492. Find a way to see the page at 400, and run the fixed
    situations in one browser other than Edge.
  - Done when both were seen, or the record says what could not be seen and why.
  - _Boundary: Page_
  - _Requirements: 1.3, 1.4_
- [x] 4.4 The toolbar, the keyboard and a screen reader
  - Every control reachable and usable by keyboard, with a visible focus and a name; no meaning
    carried by colour alone; reduced motion respected.
  - Done when a written walk through the page by keyboard is in the research entry, with what
    worked and what did not.
  - _Boundary: Page_
  - _Requirements: 1.4_

- [x] 5. What the page explains
  - Record: [R177](../../../docs/research/2026-11-23/14-run-page-group-5.md). The owner read the
    wording of 5.2 on 2026-11-23 and let it stand ("ok").
- [x] 5.1 Your code
  - Done 2026-11-23: a line at the top of the page for the whole run and the tab, each sentence
    drawn from its fact. Beyond the page: the run now measures the copies at its end and names its
    own output that lies in the repository (`page_subject.rs`, `plan_show.rs`); before, neither
    fact was ever set. The warning on the main branch is shown whenever the run reads it; that a
    change is judged is not in the snapshot before task 6.1.
  - The tab and the top of the page say what the tool touches, from the snapshot of task 1.2, with
    the warning on the main branch and, at the end, the space the copies take and how to remove them.
  - Done when the fixed situations show each sentence only where its fact is in the snapshot.
  - _Boundary: Page_
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_
- [x] 5.2 What a planted fault is, with one from this run
  - Read by the owner 2026-11-23, 20:10, in the conversation of group 6, with the line "The code
    there" as it is since task 6.2 (left out unless `--page-source` is given); answer: "ok".
  - The explanation, the first classed fault as the example, what a noticed fault shows and does
    not, and the three readings of an unnoticed one.
  - Done when the text is in the page, the example appears with the first classed fault, and the
    owner has read the wording.
  - 2026-11-23: the text is in the page and the example names its function as well. Not ticked:
    the owner has not read the wording yet.
  - _Boundary: Page, Snapshot_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.6_
- [x] 5.3 Faults in code the platform does not compile (can fail)
  - Done 2026-11-23: the packs can. The Rust pack names, per operating system, the attributes under
    which code is not built there; the page's log counts a fault under one apart, whatever its run
    gave. The recorded run of R164, told to the page again, shows 66 of its 268 apart and not 64:
    two are single statements under a condition, which R164's count by the lines of two modules took
    for built code that did not build. The pack limit went from 40 lines and 2,500 characters to 50
    and 3,200 by the owner's decision. The run itself and the plan still plant and count these
    faults as before; conditions the pack does not name, and a module made conditional in another
    file, are not found.
  - The tool does not know this today. Find out whether the language packs can mark a site that
    lies under a condition of compilation, for Rust first. If they can, such faults are counted
    apart and shown apart. If they cannot, the page says nothing about it and the research entry
    says why; no guess is shown.
  - Done when the renderer of the recorded run shows its 64 such faults apart, or the entry records
    the failure.
  - _Boundary: Pack, Snapshot, Page_
  - _Requirements: 5.5_
- [x] 5.4 Without tests, nothing is protected
  - Done 2026-11-23: four notices, each shown alone in its fixed situation; the share of
    unprotected functions is the first line of the result; the sentence on devices only where a
    function's reaching out is listed. The notice for a test command that runs no test is in the
    page, but `step plan` does not count the tests that ran, so a real run never shows it.
  - The notices for no test command, for tests that do not pass on the unchanged code, and for a
    copy that drops out; the share of unprotected functions before any other result; the sentence
    on code that needs a device.
  - Done when each fixed situation shows its notice and no other.
  - _Boundary: Page_
  - _Requirements: 6.1, 6.2, 6.3, 6.5, 6.6_

- [x] 6. `step check` and the report
  - Record: [R178](../../../docs/research/2026-11-23/15-run-page-group-6.md).
- [x] 6.1 `step check` tells the log
  - Done 2026-11-23: `step check` takes `--page` and `--open` (`step/check_page.rs`; the showing in
    `plan_show.rs` now serves any command). Six stages, in the order things happen: the checks
    without a run, tests and faults before the step, tests and faults after it, the comparison.
    The faults of a file and of a function carry their side (`before`, `after`); the subject is
    the step's branch and commit and names the commit it is compared with; the verdict and the
    evidence are named as written into the repository. A refusal carries the verdict's reasons
    (`report.refusal`), and an acceptance of functions in which no test noticed a fault before
    says that nothing was protected before. Tests in `native/tests/run_page_check.rs`: an
    accepted step, a step refused before any run, steps refused or not judged by their runs; the
    verdict is the same bytes with and without the page. The page itself does not read the new
    fields yet (task 6.2). When the tests do not pass before the step, the check refuses and the
    page's notice says the run cannot judge: both are shown, the check was not changed.
  - Its stages: the checks that need no run, the unchanged tests on the base and on the step, the
    faults on both sides, the verdict. A refusal before any run ends the page at once, with the
    reasons.
  - Done when a refused and an accepted fixture step each give the snapshot expected for them.
  - _Boundary: Step, RunLog_
  - _Requirements: 3.1, 6.4, 7.7_
- [x] 6.2 The report: the page with the data inside
  - Done 2026-11-23: the report of a check has one row per measure of a lens
    (`adapters/driver/page_report.rs`, built in `step/check_report.rs`): size and structure from
    the two commits, structure counted as `deconvolute measure-scope` counts it; behaviour kept
    from how the two stages of the tests ended; protection from the verdict, as a row and per
    function. State and local complexity say "not measured" unless `--measures <file>` hands in
    rows for them, which are then marked as handed in, with the command they are from. Each row
    names its command; the result names the two commits and the tool's version. The outcome stands
    in one box at the top of the result, the same for accepted, refused and not judged, with the
    reasons of a refusal inside it. The line of code of the example fault is left out unless
    `--page-source` is given, also for `step plan`. The faults of each side are under the stage
    of that side, and the unprotected functions are those after the change. In a browser: four
    fixed situations of a check, the box of a refusal against that of an acceptance, and the six
    reports the tool wrote for the fixture steps (`LESSCODE_CHECK_REPORTS`). The options of the
    check moved to `step/check_show.rs`. A check that ends with an error has no report rows.
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
- [x] 6.3 The drawing of functions and calls, and the grid of shapes
  - Done 2026-11-23: `report.graph` has the functions of each file and the calls among them on
    both sides, from the two commits, as `measure-scope` finds them; `report.shapes` has per type
    the shapes that can be written and, for each, whether the code reaches it and the tests see
    it, only from the file named with `--measures` (`step/check_handed.rs`), each of the two
    marked "not measured" where it is absent. The page draws the two sides on one rim each, a
    function taking the same part of the rim on both, a call across files dashed, and the shapes
    as cells that say in words what was reached and seen. In a browser: the numbers of
    `SharedFrame` in drift-rdp (`native/page/shared-frame.json`: the file at `e4d3b2c` and at
    `e02154e`, 15 functions and 8 calls against 17 and 12; the 8 shapes of the first run, 6
    reached, 6 seen) show both drawings; a report without them shows neither and says "not
    measured" twice. A scope of more than 400 functions or 2,500 calls is not drawn. Not drawn
    from the page "Two Bowls": the ring for local complexity and the mark for state writes; the
    snapshot has no numbers for them.
  - The two drawings of the page "Two Bowls", from the snapshot: functions and calls before and
    after to one scale; per type the shapes that can be written, are reached and are seen. Each
    only where its numbers are in the snapshot.
  - Done when the report built from the numbers of the first run on `SharedFrame` shows both, and a
    report without those numbers shows neither and says "not measured".
  - _Boundary: Page_
  - _Requirements: 7.5, 7.6_

- [x] 7. Several reports
  - Record: [R179](../../../docs/research/2026-11-23/16-run-page-group-7.md).
- [x] 7.1 A list of reports
  - Done 2026-11-23: `step reports <report>... --out <directory>` reads the run inside each report
    and writes, into a new or empty directory, a copy of each report (`report-01.html`, ...) and
    `index.html`, the list (`native/page/list.html`, drawn from `page_list.rs`). Each report is one
    block built the same way: the project (the name of the repository's directory, the link), the
    outcome with a mark and a word, the date, the scope, the two commits, the reasons or the last
    sentence of the run, and the rows of its lenses. `--without-names-of <project>` puts a number
    where a file, a function or a type is named (`page_names.rs`), in the copy of the report and
    so in the list; the project's commands, the files the run wrote, the shapes of a type and the
    line of source are left out as a whole; a project no report is of is an error. In a browser:
    the list of an accepted step and two refused ones at 1,280 and at 400, each link, and the
    list and its three reports without names, tab by tab. The fixture has two refusals, not one.
    A name is not looked for in the tool's own sentences; in a reason of the run a function named
    like a common word takes that word out too.
  - A subcommand that reads the data inside several reports and writes one page that lists them,
    links to each, and shows refused and worsened ones like the others; with an option that leaves
    the names of files and functions of a named project out.
  - Done when a list of three fixture reports, one of them refused, shows all three alike, and the
    option removes every name of the named project from the list and from its report.
  - _Boundary: Gallery writer_
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [x] 9. Short first (owner's request, 2026-11-24, done before group 8)
  - Record: [R180](../../../docs/research/2026-11-24/01-run-page-short-first.md).
- [x] 9.1 The flow of the page and its descriptions
  - The owner, 2026-11-24: developers need the page short and simple at first, with a way to go
    deeper; they are heavily loaded, and a new tool that changes their code needs explainers and
    pictures. Worked out with the skill `cobesy`; the page only, the snapshot is unchanged.
  - Done 2026-11-24: the head says what the run asks, in the person's words, and names an ended
    run by its outcome. "No file of your project is changed" stands first in the line on what
    the run touches. The bar of faults has its numbers in words under it, and the tree the
    meaning of the marks it shows. A stage says one sentence first, what is needed from the
    person (nothing), the functions so far as one square each, and, closed, the four steps of a
    planted fault. The result says the outcome, then what to do next, then one tile per lens in
    plain words, then the functions as squares, then the numbers as before. The tab "Planted
    faults" is one sentence, the four steps, the example, and three parts that open. The list of
    reports shows the tiles and opens the rows of a report's measures on request.
  - Done when the browser check shows the outcome, what to do next and the tiles before the rows
    of the measures, and the four steps closed while a run goes on and open by keyboard.
  - _Boundary: Page_
  - _Requirements: 3.1, 5.1, 5.2, 7.2, 7.3, 7.7, 8.1_

- [ ] 8. Closing
- [x] 8.1 A real run, watched
  - `step plan --page --open` on a subsystem of a real project, with the owner looking at the page
    while it runs.
  - Done when the research entry says what the owner saw, in the owner's words, and what was
    changed because of it.
  - Done 2026-11-24 on the renderer of drift-rdp. The first attempt was void (three copies, out of
    memory); the second ran to its end on two. Nine remarks of the owner, seven changes of the page, and
    a brief for a brownfield test stage. The owner saw the changed page in pictures, not following a run.
    Record: [R181](../../../docs/research/2026-11-24/02-run-page-a-real-run-watched.md).
  - _Requirements: 3, 4, 5, 6_
- [x] 8.2 The documentation
  - The guide gets a section on the page; `docs/steps.md` the two options; the skill
    `untangle-state` a line telling an agent to open the page for the person. The owner's wish of
    2026-11-23: recommend `https://github.com/jamesaphoenix/diff-core` in the documentation, as
    something that should suit developers. Read that project first and write what it is for in one
    sentence that is true of it; the wording is the owner's to approve. A humanizer pass before the
    push.
  - Done when the owner has read the section.
  - Done 2026-11-24: `docs/steps.md` (the options, `step reports`, the recommendation of diffcore, whose
    sentence the owner approved), `docs/state-refactoring.md` ("The same on a page"), the skill
    `untangle-state`. The owner read them: "docs are fine". Humanizer pass done; one list lost its bold
    headers.
  - _Requirements: 2.1_
- [ ] 8.3 Validation and release notes
  - The suites on Windows and through Dagger on Linux; budgets; lints; the opener and the listener
    on Linux in a container (the opener with a stand-in); a validation report; an entry for the
    next release notes.
  - Done when the validation report says GO or names what stands against it.
  - _Requirements: all_
