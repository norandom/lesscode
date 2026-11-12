# R106: Gates P2 and P3 of the model seam. The library builds for five targets with either TLS option; a wait is recognised and announced on a stub, and no real refusal could be provoked

- Report date/time and timezone: 2026-11-12, 19:10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-12, 18:57 to 19:10
- Status: P2 complete; P3 complete on the stub, its live part not shown
- Spec: [model-seam](../../../.kiro/specs/model-seam/requirements.md), [prototype gate](../../../.kiro/specs/model-seam/prototype-gate.md)
- Hypothesis ID and predecessor: assumptions P2 and P3 of the gate; [R105](12-model-seam-gate-p1.md)
- Type: prototype gate, pass and fail conditions written before the run (commit `2eb3f2c`)

## Decision

P2 passes: see the table; the TLS option is a free choice, and the design takes the library's default.
P3 passes for what a stub can show: the rule finds a stated wait in a header and in a message, falls back
to a schedule, tells the user before each wait, stops at the limit and does not wait out an ordinary
refusal. Its live condition, one real refusal waited out on the local endpoint, is not met: 38 calls sent
at once were all answered. The gate keeps that as an open item, to be closed during implementation when
a long run meets the limit.

## Hypothesis and falsification

As in the gate. P2: the seam builds for the five release targets with pinned versions and without system
packages beyond the hosted runners' images; fails when a target fails for both TLS options. P3: in each
stated case the seam prints an estimate before waiting, the call then succeeds, and a wait beyond the
limit ends with a report; fails when an estimate is missing or wrong by more than a factor of two in a
stated case, or the call does not recover.

## Frozen inputs and method

- P2: the prototype crate of R103 (`docs/experiments/model-seam/prototype/`, the library, an async runtime
  and the core as a library), built with `cargo build --release --locked` for the release workflow's five
  targets, once with the library's `rustls` option and once with the platform's TLS, by a workflow that
  lives on the branch `model-seam-gate-p2` only. The built binary is started without arguments and must
  print its usage line.
- P3: `docs/experiments/model-seam/prototype-wait/` (Rust). A stub endpoint on a local port answers from
  a script. The wait before the next attempt is the larger of what the endpoint states (header, else
  message, else a first step) and twice the wait that has just failed. For the stub the first step is one
  second and the limit six; for a real endpoint ten seconds and ten minutes.

## Evidence and results

P2, workflow run 43275325942:

| Target | `rustls` | platform TLS |
| --- | --- | --- |
| x86_64-unknown-linux-gnu | built, 2 min 47 s | built, 2 min 34 s |
| aarch64-unknown-linux-gnu | built, 2 min 00 s | built, 1 min 50 s |
| x86_64-pc-windows-msvc | built, 4 min 34 s | built, 4 min 39 s |
| x86_64-apple-darwin | built, 8 min 00 s | built, 9 min 55 s |
| aarch64-apple-darwin | built, 4 min 22 s | built, 2 min 38 s |

All ten jobs succeeded, each with `--locked`, no package installed beyond the runner image, and the
binary printing its usage line. Times are whole jobs, toolchain install included.

P3 on the stub (`model-seam-prototype-wait stub`, exit code 0):

| Case | Script | Waits announced and made | Result |
| --- | --- | --- | --- |
| wait stated in a header | 429 with `retry-after: 2`, then an answer | 2 s (header) | answered |
| wait stated in the message | 429 with "retry after 1 seconds", then an answer | 1 s (message) | answered |
| no wait stated | 429, 503, then an answer | 1 s, 2 s (none stated) | answered |
| refused for good | 429 with "retry after 1 seconds", always | 1 s, 2 s; the next of 4 s would pass the limit of 6 | failed: waited 3 s, limit 6 s, 3 attempts |
| refused for another reason | 400 | none | failed at once with the endpoint's message |

The notice before each wait reads, for example: "127.0.0.1:54074 asks to come back later (attempt 2);
waiting about 2 s (none stated), 1 s waited so far".

P3 live, local endpoint, GLM 5.3: 8 calls at once, then 30 at once. All 38 answered within 31 seconds;
no refusal, so nothing to wait out. The refusal seen earlier today ("Upstream rate limit persisted; retry
after 1 seconds", R102) came while a run of ours held four long calls for most of an hour. Its message is
the one the second and fourth stub cases use.

## Limits and interpretation

- The stub shows the rule, not an endpoint's behaviour. Whether a real endpoint's stated wait is honest is
  not known; the fourth case shows what the rule does when it is not (the wait doubles).
- "Still running" notices for a slow answer (requirement 3.5) are not part of this prototype.
- The build covers the library with the core as a dependency, not the finished seam.

## Tool/spec consequence and next gate

- Design, waiting: the next wait is the larger of the stated wait or first step and twice the wait that
  just failed. This replaces "never below the longest wait that failed", which would have repeated a
  dishonest one-second wait until the limit.
- Design, TLS: the library's default.
- Open: P3's live condition; P4 (the core through the process boundary, receipts).

## COBESY check

Conditions were fixed before the run. The unmet live condition is reported as unmet, with what was tried.
