# Rules for changes to the evaluator package

Apply these rules while writing code, before considering a cleanup pass. They
are repository instructions and review criteria, not an automatic policy engine.

## Preserve the evidence contract

- Keep public records, field names, CLI exits, decision precedence and source/plan bindings stable unless an approved change explicitly alters them.
- Preserve input validation, behavior checks, diagnostic retention and integrity protections. Never remove them to reduce a metric.
- Use the existing typed contracts. Keep measurement definitions and unavailable-evidence reasons explicit; do not invent a universal quality score.
- Keep checks, thresholds and exclusions outside candidate control. Do not change a frozen profile or test inventory to make the current candidate pass.

## Make the role of code visible

- Name values for their role when the context contains several objects: `prepared_plan`, `baseline_metrics`, `candidate_snapshot`, `threshold`, `artifact_index` or another precise domain name.
- Distinguish raw vendor data, validated evidence and public records. Names and types should show which boundary a value has crossed.
- Short local names are fine when their role is obvious. Avoid vague names such as `manager`, `helper` or `data` when they conceal a policy or evidence boundary.
- Preserve established public field names. Do not rename serialized fields as a style-only change.
- Explain surprising domain policy in comments. Do not narrate obvious syntax or hide important behavior behind an unexplained abbreviation.

## Remove unnecessary structure

- First consider deletion, reuse of existing code and standard-library features. Introduce a class, wrapper or interface only for a demonstrated state, policy or integration boundary.
- Prefer idiomatic Python with explicit control flow. A comprehension, dispatch table or built-in operation is useful when its semantics and purpose are clearer than the code it replaces.
- Keep conditionals when their evaluation order, side effects or distinct error handling matter. Do not turn them into eager expressions merely to lower CCN.
- A helper should own a coherent operation. Splitting a function only to move branches out of a complexity report does not establish simplification.
- Avoid speculative extension points, pass-through layers and duplicate representations. Record the concrete reason when an abstraction is retained.

## Measure and review the change

- Report physical lines, LLOC and analyzer NLOC under their own definitions. None is a readability or maintenance-hours score.
- Inspect changed functions for branching, repetition, nesting, unnecessary indirection and the number of concepts a reviewer must hold together.
- When complexity increases, explain the requirement and the simpler alternatives considered. Existing hotspots remain visible; they are not erased by improving a smaller function.
- Check signatures, call sites, public fields and test burden as change-surface evidence. Human review must assess idiom and role-revealing names; static counts alone cannot certify them.
- Run the relevant behavior checks and required regression suite. Preserve incomplete and rejected measurements. State which claims remain unmeasured.

See the [self-evaluation example](../../docs/examples/self-evaluation.md) for a
worked change and its limits. General configurable coding-agent rules remain a
separate proposed feature; these instructions apply to this package now.
