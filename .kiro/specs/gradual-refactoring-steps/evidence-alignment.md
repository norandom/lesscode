# Evidence for step specs: how lesscode's file relates to openidea-kiro's `verification-evidence`

Written 2026-11-11 (Requirement 8.3). It replaces an issue text written earlier the same day, which
addressed openidea-kiro as an outside project and asked questions its own spec had already answered.
openidea-kiro is the owner's co-project (`../openidea-kiro`). Owner decision, 2026-11-11: no issue; this
note, `RunRecord` as the target, and the evidence file stays `step-evidence-provisional-v0` in release 1.2.0.

## What exists on each side

- **openidea-kiro**, spec `verification-evidence`: requirements and design approved, tasks generated and not
  approved, nothing implemented (state of 2026-11-11, commit `6f66d5a`). The design defines a capability
  ledger `.kiro/breadth.toml`, an execution record `RunRecord`, and `kiro-cli breadth accept --spec <name>
  --records <path>...`, which judges records and never runs a command itself.
- **lesscode**: `step check` writes `step-evidence-provisional-v0` beside the step's spec. Example:
  `docs/experiments/gradual-steps/steps/one-exit/evidence.json`.

## Field by field

| `RunRecord` (openidea-kiro design) | lesscode today | Gap |
| --- | --- | --- |
| `schema_version` | `receipt` name with a version, `provisional: true` | form only |
| `capability`: an ID in `.kiro/breadth.toml` | none; `step` and `spec` | a step has no capability entry; either the step's spec gets one or the step name serves as the ID |
| `criteria`: `<spec>:<N.M>` | `spec` plus `criteria[].id` as `N.M` | none in substance |
| `command`, `exit_code` | per criterion | none |
| `cwd` | not written; always the root of the copy | small |
| `collected`, `results`: every test by suite, binary and name, with its status | not written | **the real gap.** Their Requirement 3.3: a missing, uncollected, skipped or failed required test rejects the claim even when the command exits 0. lesscode reads exit codes only and does not parse runner output |
| `identity`: digests of source, specs, config, lockfile, tests, binary, toolchain, platform | `hashes`: scope files, tests, declaration; `base` and `revision` commits | lesscode covers source (scope only), tests and the declaration; no lockfile, toolchain, platform, binary |
| `fixtures`, `artifacts` | not written | not applicable to a step so far |

What already agrees: evidence is bound to content, not to a commit, and the evidence file itself is excluded
from that identity (their 3.8; lesscode's guard, Amendment 4). The verifier computes the expected identity
itself (their 3.2; lesscode's guard does the same for its own hashes).

## What follows for lesscode

1. The target format is `RunRecord`. lesscode does not define a competing agreed format; `provisional-v0`
   stays until `breadth accept` exists and a lesscode record can be tested against it.
2. Closing the real gap needs per-test results from the project's runner (nextest and JUnit XML, Node's test
   reporter, Maven surefire reports). That is a language-pack or run-configuration matter, since the step
   tooling may hold no rule for a particular language (Requirement 1.5). It is not designed here.
3. Until then, a lesscode evidence file says: this command ran on this content and exited with this code.
   It does not say which tests ran. Under openidea-kiro's rules that is not enough for a `passed` capability,
   and lesscode's documents must not claim otherwise.

## Open, for the owner, when `verification-evidence` is implemented

- Whether a step is a capability in `.kiro/breadth.toml` or stays outside the ledger.
- Whether openidea-kiro should accept a weaker, command-level record from tools that cannot name tests. That
  would be an amendment to `verification-evidence` Requirement 3, in that repository.
