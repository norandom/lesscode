# Configure a comparison for your code

A profile defines the question the evaluator can answer. Establish its source
scope, check inventory, required measurements and thresholds before preparing
either side. A profile change during a run invalidates the comparison.

The checked-in [Python profile](../benchmarks/profiles/python.json) is an
executable boltons fixture example, not a generic profile for an arbitrary
repository. Its independent golden data and runner test that module specifically.

## Keep evaluation separate from source

A useful layout has four sibling directories:

```text
trial/
  evaluator/     profile.json, fixed runner, expected results
  baseline/      original source and supporting tests/config
  candidate/     supplied changed source and supporting tests/config
  evidence/      new plan and run output paths
```

The evaluator directory and resources must be outside both sources. Outputs
must be outside sources and evaluator resources. Keep analyzer environments
outside the measured trees, or explicitly exclude them. Do not let a rewriting
agent edit the evaluation rules, independent checks or expected results.

Copy the example profile and its runner resources together when learning the
format. For a different project, replace the check runner, expected IDs and
resource list with that project's real executable contract. Required
schema-version-1 fields are validated by
[`Profile.from_dict`](../src/lesscode_baseline/contracts.py); unknown fields and
duplicate JSON keys fail rather than being ignored.

## Select the measured source

The `scope` object requires `include`, `exclude`, `generated`, `vendor`, `tests`
and `product` arrays. Globs are case-sensitive POSIX-style paths on all platforms.
Classification precedence is excluded, generated, vendor, tests, product, then
support. Only selected product `.py` files enter the initial structural adapters.

Declare test, vendor, generated, environment and output paths explicitly. Do not
assume `.gitignore` defines the evaluation scope. Matching added/removed files
remain part of the fixed comparison; untracked bytes can contribute to identity.
Test-file changes are reported separately from product measurements.

## Declare checks that can actually establish evidence

Each `checks` entry needs an ID, kind, required flag, command, result path,
format and expected result IDs. A command declares an argv array, cwd, explicit
environment and positive finite time/log limits. `cwd` is relative to a fresh
source copy. Profile-relative runner resource arguments are resolved before
that working directory changes.

Use JUnit (`result_format: "junit"`) or the typed normalized evidence contract
(`result_format: "normalized"`). The result file must be freshly produced at the
declared path in the check copy. Expected IDs must match the reader's normalized
identities; examine the fixture runner and
[`tests/test_checks.py`](../tests/test_checks.py) for examples. Merely running
`pytest`, `unittest` or another command with exit 0 does not supply a complete
result inventory.

At least one behavior check must be required. Every declared baseline behavior
check must pass, including any marked optional, before the trial is eligible.
On the candidate, observed failures reject; missing or skipped required evidence
leaves it incomplete. Keep optional metrics separate from optional behavior
checks when deciding what to declare.

List the profile, independent runner and expected-result resources under
`evaluator_resources`. Preparation records their bytes. Changes to that inventory,
configuration or tool environment require a new plan. The sample runner uses
only the standard library; your project's test dependencies must be provisioned
explicitly in the interpreter environment that executes those commands.

## Choose supported thresholds

Profiles declare `required_metrics` and a `thresholds` array, which can be empty.
The baseline requires complexity, duplication and source bytes. The example also
requires physical line count and compressed source size. Tool availability is
not an acceptance threshold: missing required metrics makes a run incomplete.

A threshold names `metric_key`, `operator`, finite `value` and `unit`. The
supported operators are inclusive maximums:

| Operator | Compared value | Unit |
| --- | --- | --- |
| `absolute_maximum` | Candidate value | Metric's exact unit |
| `delta_maximum` | Candidate minus baseline | Metric's exact unit |
| `percentage_delta_maximum` | 100 times delta divided by baseline | `percent` |

For example, this complete threshold object allows no source-byte growth:

```json
{
  "metric_key": "source_bytes",
  "operator": "delta_maximum",
  "value": 0,
  "unit": "bytes"
}
```

It is one entry in `thresholds`, not a complete profile. The fixture uses this
rule to test unchanged and shorter candidates; it is not a universal policy for
new features. Choose a budget suitable for the change and record it first.

The built-in complexity and duplication measurements are structured payloads.
They retain function distributions and clone records; the current generic
threshold engine does not extract maximum CCN or clone ratio as implicit scalar
values. Setting a numeric threshold on those keys cannot establish a pass.
Scalar companion evidence needs an explicit producer definition and its own
key. Per-function prevention rules and debt ratchets belong to the proposed
agent-quality-rules feature, not this profile format.

Zero-baseline percentages and undefined scalar projections remain unavailable.
Do not replace them with zero or remove a required metric to make a run pass.

## Bind tools and inspect the plan

Use the pinned Lizard/jscpd versions and supported fixed options documented in
the [CLI reference](cli.md#fixed-analyzer-settings-and-audit-files). Prepare
with explicit `--tool-paths` metadata from [analyzer setup](analyzer-setup.md).
Relative metadata values resolve from that file's directory. Python bindings
preserve the lexical venv interpreter path while separately fingerprinting the
resolved runtime and environment configuration.

Inspect resolved argv, environments, roots and limits before executing `run`.
Preparation can record unavailable tools; it does not install them or guarantee
that execution will be accepted. Preserve tool environments for the lifetime of
the plan. Source edits, runner changes or environment replacement require new
preparation and new output paths.

For report states and unsupported evidence, see
[decision rules](technical-overview.md#decision-rules). For optional evidence
imports and experiment manifests, see the [CLI reference](cli.md).

The [self-evaluation example](examples/self-evaluation.md) supplies a second
complete profile for Lesscode's own tests and a separately scoped module
profile. It retains an incomplete whole-package result rather than silently
dropping a required measurement. Its auxiliary LLOC and MI observations do not
change the built-in profile contract.
