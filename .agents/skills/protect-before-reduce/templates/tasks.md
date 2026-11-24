<!--
Form of the tasks of a stage spec (skill protect-before-reduce, phases 4 to 10).
Replace everything in <angle brackets>; remove this comment and every line that begins with
"Guide:". Write in the language the project writes its specs in.
-->

# Tasks: protect <file or subsystem> before it is reduced

One task per test. Each test is first shown failing on the planted fault and passing on the
unchanged code (`lesscode-core step pin`; the receipts are in `pins/`). All tests go into
`<test file(s)>`; `<code file(s)>` is not changed.

Guide: a task is one test. A test may be for several faults where one behaviour explains them; do
not plan more tests than faults. "Fixes" is the one sentence the owner of the code checks against
what they want the code to do: say what holds, so that it could be wrong. Faults are named by id;
the number in brackets is the `#N` of `step pin`, filled in after the base has run (phase 5).
Tick a task only when `step pin` admitted the test for every fault it names.

- [ ] <N>. <the behaviour, in a few words>
  - Fixes: <one sentence: what holds after this test, in the words of the code's subject>
  - Faults: `<id>` (#<n>), `<id>` (#<n>)
  - Pins: `pins/pin-<n>.json`, `pins/pin-<n>.json`
  - _Boundary: <test file>; test files only_
  - _Requirements: <N.M>_

Guide: a task that was stopped because the test would have needed a change of the code keeps its
entry without that fault, and gets one line: "Given up for `<id>`: <why>; proposed for writing
off."

Guide: the last task holds every fault no admitted test notices. It is done when each has its
entry in write-offs.json; approving them is not part of the task.

- [ ] <N>. Proposals to write off, for the owner of the code
  - `<id>`, `<id>`, in `write-offs.json`, each with its kind and reason.
  - _Requirements: <N.M>, <N.M>_

## Result, <date>

Guide: fill this section from the output of `step protect result` and from nothing else, after
the one review. Until then leave the line "Not judged yet." Give the exit code as it was.

`lesscode-core step check`: <accepted | refused | not judged> (`verdict.json`).
`lesscode-core step protect result`: exit <code> (`protection.json`).

| Function | Noticed before | Noticed now | Written off | Waiting |
| --- | --- | --- | --- | --- |
| `<function>` | <n> of <n> | <n> of <n> | <n> | <n> |

Written off, with the owner's approval (`write-offs.json`): `<id>` (<kind>): <reason>. Approved by
<name> on <date>.

Waiting: `<id>`: <why it waits, as the result says>.

<"All <n> functions are protected enough." | "Not protected enough: <n> fault(s) wait.">
`<code file(s)>` is unchanged since the base. <What the owner said of the tests, in their words.>

Known gap: <what no test checks after the stage, in plain words, and what a reduction of that
function has to keep by other means>.
