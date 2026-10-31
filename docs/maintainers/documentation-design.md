# Documentation design notes

This is the COBESY composition plan behind the adopter README and technical
guides. It records design assumptions, not findings from an audience study.
Adopters can start at the [docs index](../README.md).

## Inputs and target behavior

- Raw knowledge: implemented baseline code, approved requirements/design, CLI/setup guides and retained native/readiness evidence.
- Inferred cultural assumption: maintainers want simpler code but need evidence before trusting an automated cleanup or adding review work. Avoid language that treats existing code or its authors as a failure.
- Inferred network topology: a module owner and regular reviewer can assess one change together; a CI maintainer joins before enforcement. No actual organizational network has been measured.
- Target behavior: a Python maintainer and reviewer run one supplied baseline/candidate comparison, inspect the evidence and decide whether it belongs in their review process.
- Suspected hidden assumption: smaller code or a green test command can be mistaken for enough evidence to accept a rewrite. Address it with concrete result semantics rather than a lecture.
- High-leverage anchor: a trusted reviewer of the selected module, working with its owner.

## Framing protocol

Visible artifacts are source diffs, test output and review decisions. The stated
value is preserved behavior with less unnecessary structure. The suspected
tension is the added cost of verifying an attractive simplification.

Psychological safety and accountability have not been assessed. The default is
a voluntary learning trial with clear standards. If failures are used to blame
individuals, establish a safe review agreement before promoting enforcement.
Frame the trial as testing the evaluator's usefulness on one change. Avoid
promises to fix every AI-generated codebase or rank developers by LOC.

Disproof condition: do not rely on the evaluator as a gate if matching inputs
produce unexplained structural differences, missing mandatory evidence appears
accepted, or ordinary check execution changes original sources. Reconsider
adoption if its cost exceeds the review decisions it improves.

## Composition blueprint

Governing idea: a reviewer can inspect a supplied simplification under fixed
checks and measurements, then decide whether to accept it within those limits.
The audience is an experienced Python maintainer; the purpose is a first useful
comparison. Use plain technical prose, exact commands and explicit limits.

| Section | Reader's question | Material and emphasis | Keywords |
| --- | --- | --- | --- |
| README opening | What review problem does this solve? | Behavior plus structural evidence; supplied candidate; local execution | compare, declared checks, evidence |
| What you get | What can I inspect? | Outcomes, complexity, duplication, size, identity; no universal score | report, scope, profile |
| Unchanged pair | What can I try first? | Platform-specific commands and expected fixture result; setup links | uv, plan, run, summary |
| Technical links | Where is the depth? | Indexed guides, contracts and actual validation | definitions, recovery, provenance |
| Current and planned scope | Can it enforce agent rules today? | Delivered evaluator versus proposed prevention/skills/MCP | supplied snapshots, proposed rules |

The opening names the maintainer's existing review conflict. The strongest
objection, that passing checks does not prove equivalence, is answered before
the first command. The technical writeup carries the full definitions and failure
semantics so the README does not become an implementation inventory.

## Adoption path

This is an optional process change with real setup and review cost. Seed one
owner/reviewer pair around a module they know. Share an actual report with an
adjacent team before asking for broader use. Bring in the CI maintainer when
independent enforcement becomes useful. This is a suggested path, not a claim
about measured influence or consensus.

Remove barriers through explicit setup, a reproducible fixture, a reversible
advisory trial and a choice about enforcement. The [adoption guide](../adoption.md)
puts those actions next to the documentation needed to perform them.

## Composition and humanizer checks

The README separates payoff, first action, technical references and future
scope. The docs index groups setup, interpretation, evidence and maintenance.
Detailed definitions stay in the technical overview; invocation details stay in
the CLI reference. The framing includes a disproof condition and a low-risk
trial rather than a mandate.

The prose pass removes generic claims, decorative emphasis, em/en dashes,
curly quotes and rollout jargon from adopter-facing text. The remaining risk is
reference density: keep specialized definitions in linked guides and review
each new paragraph against the governing idea. Do not turn future plans into
current product claims.
