# R181: the run page on a real run, watched by the owner: what the owner said and what was changed

- Report date/time and timezone: 2026-11-24, 09:45 Europe/Berlin (from the clock)
- Execution interval: 2026-11-24, first attempt 07:55:48 to 08:02:05, second attempt 08:02:28 to 09:38:03 (from the start and end files of the runs)
- Status: task 8.1 completed on Windows; not run on Linux after the changes
- Spec: [run-dashboard](../../../.kiro/specs/run-dashboard/tasks.md), task 8.1
- Hypothesis ID and predecessor: [R180](01-run-page-short-first.md) (the page this run showed), [R164](../2026-11-23/01-renderer-baseline-faults.md) (the same subject and scope, without a page)
- Type: observation of one person using the page on a real run, and the implementation record of what followed

## Decision

The page of R180 did not pass its own test. R180 named what would show the design wrong: someone
who sees the page for the first time cannot say what the run found. On the first attempt the run
found nothing, because the machine ran out of memory, and the page told the owner that the tests do
not pass. The owner's words: "tests don't pass - why? they worked before. unclear what this means".

The page was changed in seven places because of what the owner said, and the owner asked for a new
stage of the whole path, the brownfield test stage, which now has a brief of its own. Task 8.1 is
done by its own measure: this record says what the owner saw, in the owner's words, and what was
changed. It does not show that the changed page is understood: the owner saw the changes in
pictures and in my descriptions, and the run in the owner's browser showed the old page to its end.

## What was run

- Subject: `D:\Source\drift-rdp` at `09b4ce8`, branch `master`, nothing dirty under `src-tauri`; only read.
- Scope and settings: those of R164, the seven Rust files of `src-tauri/src/renderer`, the crate's
  library tests. [config.json](../../experiments/run-dashboard/real-run/config.json),
  [run.ps1](../../experiments/run-dashboard/real-run/run.ps1).
- Tool: `lesscode-core` built from `850999a` (release profile), `step plan --page --open`, without
  `--page-source`.
- The owner chose the subject and asked to be told when it was ready.

| Attempt | Copies | Result |
| --- | --- | --- |
| 1 | 3 | void after 6 min 15 s, exit 3. In copies 1 and 2 the compiler ended while building `windows_numerics` (exit `0xe0000001`); no test ran. In copy 3, 535 of 536 tests passed and `renderer::yuv::tests::the_gpu_conversion_matches_the_reference` aborted (`0xc0000409`). |
| 2 | 2, same work directory | exit 0 after 1 h 35 min. 268 faults: 72 noticed, 193 unnoticed, 3 that did not build. Both copies ran 134 faults and closed with the unchanged tests passing. |

Before attempt 1 the machine had 8.4 of 29.6 GiB of memory free (a WSL virtual machine held 6.9).
I had measured that and still started three copies. The owner's reading, "Too much RAM used, get
oom'ed / killed", fits what is there; no message of the system says so, because the tool keeps only
the last 400 characters of a failed first run.

The numbers of attempt 2 are those of R164: 72 of 268 noticed, and 21 functions in which faults were
planted and none noticed. The page counts differently and says so: 66 of the unnoticed faults lie in
code that is not built on Windows, so it shows 130 unnoticed and 16 of 38 functions unprotected.

## What the owner said, and what was changed

The owner's sentences are given as written.

| # | The owner | What it was about | Changed |
| --- | --- | --- | --- |
| 1 | "boxes (how lesscode finds xyz not fit (2 (One mistake...))"; later: "2.) fits now" | the four steps under "How lesscode finds out what your tests protect" | nothing; in Chromium the boxes stood two by two and the text fitted. What did not fit on the owner's screen was not found out. |
| 2 | "Too much RAM used, get oom'ed / killed." | attempt 1 | the second attempt ran with two copies. The advice after a run that could not judge now names fewer copies (`--workers 1`). |
| 3 | "\"Copy dropped out\": it is unclear what that means even to me." | the notice of a copy that stops | "Working copy 2 of your project stopped ...: its build or its tests did not get through there. The run goes on with 1 of 2 copies, and takes longer. Your repository is not affected." What the tool recorded is behind a line that opens. |
| 4 | "tests don't pass - why? they worked before. unclear what this means" | "Your tests do not pass on the unchanged code", with 1,200 characters of compiler output, three times on the page | the notice no longer says the tests fail. It says the build and the tests did not get through in any copy, that this can be a failing test, a failing build or a machine short of memory, and keeps the output behind a line that opens. The same for the outcome box and for a failed stage. |
| 5 | "by default the Plant faults can be folded" | 43 rows of "0 noticed · 0 unnoticed" before any fault had run | the files of a stage are one row of the tree, shut while the stage has not begun or was left out. A row with nothing judged says "6 to run", after the end "6 not run". |
| 6 | "going on now -> not begun yet. that feels like a contradiction." | every function marked "going on" while the stage "Plant faults" said "not begun" | a function or file with faults waiting and none judged has not begun. Only what is going on is marked so. |
| 7 | "we should have a clear step by step visualizuation like a lane" | the stages as rows of the tree | the steps of the run stand in one row under the top line, numbered, each with how far it is, and the result as the last. The stage rows left the tree. At the end the lane stands at the result. |
| 8 | "what do we do with functions which are unprotected? igore them? that is not clear" | the tab Unprotected | "What to do with them": leave it if you do not plan to change it; protect it first if you do; where it talks to a device, a window or a network, cut what it decides from what it does. |
| 9 | "with lesscode the emphasis is that we work a codebase into being \"less code\". so we should have a test brownfield stage, where we improve this." then "kiro allows this. we can use this" and "there I'd simply refer to the browdfield and make a spec to produce the tests" | the path from a plan to a reduction | the tab Unprotected refers to the brownfield test stage: "make a spec whose tasks are the tests for these functions". A brief: [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/brief.md). The owner decided that the spec of the stage lives in the project under work and that the brief is worked after group 8. |

Two things I saw on the real data after the run and changed without the owner having named them:
on a void run the lane marked the last stage as the current one, and the files of the stages that
never ran stood open with zeros. Both are in rows 5 and 7 above.

## Evidence

- `D:\BuildArtifacts\lesscode\run-dashboard\real-run-8.1` (0.9 MiB): `plan.json`, `plan.log`,
  `plan.html` (the report of attempt 2, the page as the owner saw it), `work\plan-run.json`,
  `work\outcomes`, `start.json`, `end.json`; `attempt-1\` with the same of the void run.
  `plan-new-page.html` in both: the data of the same run in the changed page, put together by a
  script outside the tool, for looking at; the tool did not write them.
- The pages in Chromium through Playwright: 67 of 67, none skipped
  (`D:\BuildArtifacts\lesscode\run-dashboard\states-g8-1`, 11.6 MiB). One test is new: the lane, a
  step that has not begun, the end of a run, and a run that never planted a fault.
- Windows: core suite 471 of 471, 1 skipped; lints with warnings as errors, format, doctests, size
  budgets (`budgets-g8.json`) and `tools\bin\kiro-cli.exe code check --changed 850999a` pass.

## Changed files

| File | What |
| --- | --- |
| `native/page/run.html` | the lane; the tree without stage rows and with one row per fault stage; "not begun"; the notices; `Raw`; "What to do with them" |
| `native/page/states.spec.mjs` | the words and stops that changed; one new test |
| `native/src/step/plan_page.rs` | the advice after a run that could not judge |
| `docs/experiments/run-dashboard/real-run/` | the configuration and the start script of the run |
| `.kiro/specs/brownfield-test-stage/brief.md` | the brief of the new stage |

## Limits

- One person, the owner, who knows the tool. Nobody of the audience was asked.
- The question of R180, whether a first-time reader can say after ten seconds whether their code
  was changed and what the run found, was put to the owner and not answered as asked. What the
  owner said in the first minutes was about what was unclear.
- Whether `--open` started the browser by itself was not said. The owner had the page open; I had
  also printed its address.
- The owner did not see the changed page follow a run. Only pictures of fixed situations and the
  two reports rewritten with the changed page exist.
- Still not seen: Firefox, Safari, a screen reader; a project other than this one. Reviews were
  inline, by me.

## Not done, and why

The page still cannot say whether a build or a test failed in a copy, because the tool does not
tell it: a failed first run is one sentence with the last 400 characters of output. Four things
would need the tool to change, and two of its files are at their size limits (`runner.rs` 2,799 of
2,800 tokens, `page_log.rs` 2,790 of 2,800):

1. say whether the build or a test failed, from the signs of the test runner in the output;
2. keep the whole output of a failed first run in a file under the work directory;
3. try a copy again that stopped while the others were building;
4. choose the number of copies from the memory that is free, or refuse a number that cannot fit.

They are proposed to the owner and not agreed.

## Next gate

Task 8.2, the documentation, with the page's words; then 8.3. The brief of the brownfield test
stage waits for group 8 to close.

## COBESY check

The outcome stands first and is a failure of the earlier page. The owner's sentences are quoted,
with their typing. What was not found out is named. The changed page is not claimed to be
understood.
