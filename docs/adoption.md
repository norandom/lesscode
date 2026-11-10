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
   the commands do not run your tests, Python analysis is not implemented, and
   `mutate` rarely produces mutants on code outside this project's specimens.
3. Take the file before and after one proposed change. Run
   `deconvolute density` on the pair and `deconvolute analyze` on the new file
   (Rust, Java or TypeScript). Write the receipts outside the source tree.
4. Run your own test suite on both versions yourself and keep its output with
   the receipts. That is the behavioral evidence; the tool does not supply it.
5. Let the reviewer read the receipts next to the diff. Record whether they
   affected the review, what setup they cost, and which important behavior
   nothing checked.

Do not rely on a result as a blocking gate if identical inputs produce
different receipts, if a receipt reports `"passed": true` for something the
command did not examine, or if a command changes your sources. Reproduce and
report those cases. A shorter candidate that fails your tests must be rejected
whatever the density receipt says.

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

## Expand only after the first review is useful

Have the module owner and reviewer show the actual receipts to a nearby team
working on similar code. Involve a CI maintainer when moving from an advisory
review to enforcement. Those roles are suggested starting points, not a claim
that we know your team's trust network.

Keep a way to report false alarms, unsuitable thresholds and missing language
support. Agree on exceptions before enforcement and keep the original
receipts. Reconsider adoption if setup and review cost outweigh the decisions
the receipts improve.

The binary measures supplied files. It does not generate a simplification,
distribute coding-agent rules or ratchet a regression baseline. The
[roadmap](../.kiro/steering/roadmap.md) separates what is delivered from what
is proposed. Start by using today's receipts in an existing review, then
choose whether to invest in a gate.
