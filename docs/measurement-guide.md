# Measure convolution without confusing it with size

A useful simplification removes unnecessary decisions, repetition or concepts
while leaving the code's responsibilities and behavior clear. Fewer physical
lines alone do not establish that. A dense expression can be shorter and harder
to change; a named intermediate value can add a line and explain an important
role.

The [self-evaluation example](examples/self-evaluation.md) shows these distinctions
on Lesscode's own code. Use several observations together, with a written reason
for the change, rather than one compensating simplicity score.

## Keep the line-count definitions separate

| Measure | What it describes | How this example obtains it |
| --- | --- | --- |
| Physical lines | Original product bytes split into lines, including an unterminated final line | Core evaluator |
| LLOC | Logical lines under Radon's statement-counting definition | Auxiliary Radon 6.0.1 `raw.analyze` |
| SLOC | Radon's source-line count, distinct from its logical count | Auxiliary Radon 6.0.1 |
| NLOC | Lizard's non-comment line count for files/functions | Core complexity evidence |

Radon's [definitions](https://radon.readthedocs.io/en/stable/intro.html#raw-metrics)
and [API](https://radon.readthedocs.io/en/stable/api.html#module-radon.raw)
specify its counters. These are not interchangeable with the evaluator's physical
line definition or Lizard's NLOC. LLOC is auxiliary evidence in this example;
the current core CLI does not emit a built-in LLOC metric or gate on it.

Do not turn formatting changes into a simplification claim. Keep source scope
and line endings consistent, and retain source hashes so the measured bytes can
be checked.

## Assess simplicity through concrete questions

| Question | Evidence to inspect |
| --- | --- |
| How many decisions must a reader follow? | Function CCN, control-flow shape and nesting; inspect remaining hotspots, not only the changed function. |
| Is the same policy repeated? | Clone locations and manual review of repeated validation, branching or translation. Similar syntax does not always mean the responsibilities should be merged. |
| Are responsibilities easy to locate? | Call paths, wrappers, module boundaries and whether a helper owns a coherent operation. |
| Do names reveal roles? | Distinctions such as `baseline_metrics`, `candidate_snapshot`, `vendor_report` and `validated_evidence`; public fields retain stable meaning. |
| Is the implementation idiomatic and explicit? | Appropriate built-ins and data structures, clear evaluation order, resource lifetime and error handling. |
| What must change together next time? | Changed files, call sites, signatures, serialized fields and test setup. |

Names and idiom need semantic review. Counting identifiers, penalizing every
short name or banning a word such as `data` cannot establish clarity. Ask what a
value represents at that point and whether its name distinguishes it from nearby
values. Keep concise loop names when the role is already obvious.

Likewise, moving branches into new helpers can lower a function's CCN without
reducing the work a maintainer must understand. Record the new indirection and
why it is justified. The [package instructions](../src/lesscode_baseline/AGENTS.md)
apply these rules to future agent edits in this repository.

## Distinguish maintenance proxies from observed effort

Static evidence can suggest where maintenance is difficult. It cannot tell us
how many engineer-hours a change will save. The example reports local CCN/LLOC,
unchanged public signatures and the remaining high-complexity functions. It
does not report a maintenance-time reduction.

Radon's Maintainability Index combines size, complexity, Halstead volume and
comment information. Its own documentation calls the index
[experimental](https://radon.readthedocs.io/en/stable/intro.html#maintainability-index).
The example retains per-file MI as a descriptive auxiliary measurement, with
`multi=False`; it is not an acceptance score or a forecast of hours. Do not pool
it with another tool's MI definition or treat a small increase as proof of
easier maintenance. A metric called Halstead effort is also a static formula,
not observed maintenance work.

To measure actual effort, define a concrete follow-up task on both versions:

1. Choose the same bug fix or feature and fixed expected checks. Record the starting source and task identity.
2. Use comparable reviewers/developers or declared model settings, tool access and budgets. Counterbalance the order to reduce learning effects.
3. Record time spent understanding, editing, debugging and reviewing, along with failed attempts and the final behavior result.
4. Repeat enough trials to describe variation. Keep human time, agent time, tool time and provider cost separate.

The current example contains no such maintenance experiment. Until one is run,
effort savings remain unmeasured. A maintenance task that becomes slower or more
error-prone is evidence against the simplification, even when its LLOC decreases.

## Apply the rules before writing more code

Repository `AGENTS.md` instructions can guide coding agents at the point of
editing: preserve evidence boundaries, use role-revealing names, prefer existing
idioms and justify additional layers. The instructions are inspectable text;
they do not independently enforce a gate.

Use fixed profiles for executable acceptance rules and semantic review for
responsibility, naming and readability. General policy distribution, automatic
per-function ratchets and configurable agent violations remain proposed work.
This separates guidance that agents can follow now from checks that tools can
actually execute.
