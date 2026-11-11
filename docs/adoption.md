# Try Lesscode with one change and one reviewer

Use the first trial to test whether the receipts help a review you already do.
Pick one module with checks you understand and a proposed simplification small
enough to review by hand. You do not need to make a team-wide commitment.

## Agree on what the result will mean

Treat measurements as evidence about a change, not a rating of the author.
A `marginal` density verdict says the reduction is small; it does not say the
change is wrong. A reported write site or gap is a place to look, not a defect.
Neither should be relabeled to make adoption look successful.

Before running, the module owner and a regular reviewer should agree on the
source scope, the tests that count as behavioral evidence, and any size
ceiling. Record what remains unchecked. The person proposing the change should
not relax those rules after seeing the outcome.

If your team cannot discuss a failed trial without blaming its author, agree
on that review practice before adding a gate. This guide assumes a voluntary
trial; it does not establish your team's culture or incentives.

## Run a trial you can inspect

1. [Install the binary](../README.md#install) and run `lesscode-core --version`.
2. Read [what is not checked](../README.md#what-is-not-checked). In particular,
   no command establishes that a change behaves the same, and a share of the
   planted faults in typed languages does not compile.
3. Take the file before and after one proposed change. Run
   `deconvolute density` on the pair and `deconvolute analyze` on the new file
   (Rust, Python, TypeScript or Java). Write the receipts outside the source
   tree.
4. Run your own test suite on both versions yourself and keep its output with
   the receipts. That is the behavioral evidence.
5. Run `deconvolute run` on the file ([the agnostic driver](driver.md)). It
   plants each fault in a copy of your source tree, runs your own build and
   test commands there, and reports which faults your tests catch. It never
   writes to the tree you point it at. A fault your tests do not catch is a
   place to look at the tests before cutting code there; it is not a verdict
   on the proposed change, and a run in which every fault is caught is not
   evidence that the change behaves the same.
6. Let the reviewer read the receipts next to the diff. Record whether they
   affected the review, what setup they cost, and which important behavior
   nothing checked.

Releases before 1.1.0 had no `run` command, no Python analysis, and a `mutate`
that rarely produced mutants outside this project's specimens.

Do not rely on a result as a blocking gate if identical inputs produce
different receipts, if a receipt reports `"passed": true` for something the
command did not examine, or if a command changes your sources. Reproduce and
report those cases. A shorter candidate that fails your tests must be rejected
whatever the density receipt says.

## From a receipt to a checked step

From release 1.2.0 the same trial can end in a verdict instead of receipts to
read side by side. Put the proposed change on a branch, declare it as a step,
and run `step check` ([refactoring in checked steps](steps.md)). Exit code 0
says the change stayed in its declared files and size, your tests pass before
and after, and no function lost fault sensitivity. Agree on two things before
you use it with others:

- Who may record an override, and that its reason is read by someone.
- That an accepted step is still reviewed. The check does not show the change
  behaves the same, and it does not judge whether a new test asserts the right
  behaviour ([what no command checks](steps.md#what-no-command-checks)).

Start with a function your tests already cover well; `step plan` shows which
those are.

## Before you install the pre-push hook

The hook is the one part that can stop other people's work. Settle these first:

- Everyone who pushes has `lesscode-core` on `PATH`. Without it every push
  fails, and `LESSCODE_SKIP_HOOK=1` does not help because the binary never
  starts. The fix is to correct `PATH` or delete `.git/hooks/pre-push`.
- The repository has a budgets file that lists every Rust file under the
  governed root, with ceilings the owners accept. A new file fails the check
  until someone lists it.
- The team agrees who may raise a ceiling and how that is recorded.
- The hook is local to each clone. It is not installed by cloning, and it is
  not a substitute for running the same check in CI.
- `deconvolute install-hook` checks the budgets only. `step install-hook`
  (from release 1.2.0) also runs the step guard for every pushed branch.

## Expand only after the first review is useful

Have the module owner and reviewer show the actual receipts to a nearby team
working on similar code. Involve a CI maintainer when moving from an advisory
review to enforcement. Those roles are suggested starting points, not a claim
that we know your team's trust network.

Keep a way to report false alarms, unsuitable thresholds and missing language
support. Agree on exceptions before enforcement and keep the original
receipts. Reconsider adoption if setup and review cost outweigh the decisions
the receipts improve.

The binary measures supplied files and judges steps you declare. It does not
generate a simplification or distribute coding-agent rules. The only baseline
it keeps is the step ledger: per function, the fewest unnoticed faults an
accepted step left behind. The
[roadmap](../.kiro/steering/roadmap.md) separates what is delivered from what
is proposed. Start by using today's receipts in an existing review, then
choose whether to invest in a gate.
