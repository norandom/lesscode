<!--
Form of the requirements of a stage spec (skill protect-before-reduce, phase 4).
Replace everything in <angle brackets>; remove this comment and every line that begins with
"Guide:". Write in the language the project writes its specs in. Every fact at the top comes from
stage.json; do not write a count or a commit from memory.
-->

# Requirements: protect <file or subsystem> before it is reduced

Written <date> from a plan of lesscode (`step plan`, <n> planted faults in `<file>`, <n> noticed).
Plan: `<plan file of stage.json>`. Base: `<base commit>`. Tests: `<the test command of stage.json>`.
Not approved yet: the owner of the code reviews once, at the end.

A planted fault is one small mistake made on purpose in a copy of the code. Where every test still
passes with the mistake in place, nothing would tell us if a later change made the same mistake.
<Who> chose <n> function(s) to be reduced. Each requirement says what could change unseen there
today.

Guide: the three lines below come from `left_alone`, `left_out` and the plan. Leave a line out
where there is nothing to name.

Left alone: <unprotected functions of the plan that were not chosen>. Nothing has to be done about
them.

Left out: `<function>` (<n> faults, all in code that is not built on this system; nothing is known
about it here).

Protected already: `<function>` (<n> of <n> noticed).

Guide: where the tests live inside the file of the code, say here which line joins the stage's
test file and in which commit it came, for example: "The tests of the stage are in `<test file>`,
joined by `<line>` in `<code file>` (commit `<commit>`, before the plan)."

Guide: one requirement per chosen function, in the order of stage.json. The title names the
behaviour first and the function second. "Today nothing notices when ..." says what a caller would
see go wrong, in the words of the code's subject, and names every unnoticed fault of the function
by id. Do not describe the operator ("a negated condition"); describe the consequence ("the
platform is named wrongly").

## Requirement <N>: <the behaviour, in a few words> (`<function>`)

Today nothing notices when <what could go wrong, seen from a caller> (fault(s) `<id>`, `<id>`).

Guide: where the function reaches out (stage.json, `reaches_out`), say so in one line: "It calls
<names> and writes state at <places>."

#### Acceptance Criteria
1. <When/Where ...,> the <project's> tests shall fail when <the behaviour is not kept>.

Guide: one criterion per behaviour a test is to fix. A fault that is going to be proposed for
writing off gets a criterion that says so and why, in one of these two forms:

2. The fault at <line>:<column> changes nothing a caller can see: <why>. It is proposed for
   writing off.
3. The fault at <line>:<column> cannot be tested as the code stands: <what a test would need that
   it cannot have>. It is proposed for writing off; <what covers the behaviour meanwhile, or that
   nothing does>.
