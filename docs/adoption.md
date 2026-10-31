# Try Lesscode with one change and one reviewer

Use the first trial to test whether the evaluator helps your review. Pick a
Python module with checks you understand and a proposed simplification small
enough to review manually. You do not need to make a team-wide commitment.

## Agree on what the result will mean

Treat measurements as evidence about a change, not a rating of the author.
A rejected candidate can expose a useful regression; an incomplete result can
expose a missing tool or test contract. Neither should be relabeled as a pass to
make adoption look successful.

Before running, the module owner and a regular reviewer should agree on the
source scope, independent checks, mandatory evidence and any scalar budget.
Record what remains unchecked. The person proposing the change should not
silently relax those rules after seeing the outcome.

If your team cannot discuss a failed trial without blaming its author, agree
on that review practice before adding a gate. The documentation assumes a
voluntary trial; it does not establish your team's culture or incentives.

## Run a trial you can inspect

1. Run the [unchanged fixture](../README.md#try-an-unchanged-pair-first) to verify setup.
2. Configure a [project-specific profile](profiles.md) outside both source trees. Compare an unchanged project snapshot first; resolve unavailable evidence before interpreting a proposed improvement.
3. Evaluate one supplied change. Let the reviewer inspect `summary.md`, the reasons and the indexed raw evidence alongside the source diff.
4. Record whether the evidence affected the review, what setup cost it added and which important behavior it did not check.

The local trial is unsuitable as a blocking gate if identical inputs produce
unexplained structural differences, required evidence appears successful when
missing, or normal check execution alters the original source. Reproduce and
investigate those failures before relying on the result. A shorter candidate
that breaks a declared check must be rejected.

## Expand only after the first review is useful

Have the module owner and reviewer show the actual report to a nearby team
working on similar code. Involve a CI maintainer when moving from an advisory
review to externally controlled enforcement. Those roles are suggested starting
points, not a claim that we know your team's trust network.

Keep a way to report false alarms, unsuitable thresholds and unavailable tools.
Agree on exceptions before enforcement and preserve the original evidence.
Adoption should be reconsidered if setup and review cost outweigh the decisions
the reports improve.

The current CLI evaluates supplied snapshots under fixed profiles. It does not
provide an automatic coding-agent rule distribution or regression-ratchet
workflow. Those are the [next proposed features](../.kiro/steering/roadmap.md).
Start by using today's report in an existing review process, then choose whether
to invest in a gate.
