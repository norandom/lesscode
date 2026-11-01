# Implementation record

All task implementers returned READY_FOR_REVIEW and all task reviews returned
APPROVED before selective commit. Review mode was independent. Tasks 3.3 and 4.1
required one repair round each; concrete adverse findings and regression tests
were retained. Completion was checked against fresh task evidence.

| Task | Boundary / changed files | Commit |
| --- | --- | --- |
| 1.1 | Native package, parser probes, pins/notices | 05c4ee0 |
| 1.2 | model.rs and module export | 3e2e9ba |
| 2.1 | Scope inventory | 736c60b |
| 2.2 | Reviewed selections | 3b26ff1 |
| 2.3 | Paired scope accounting | 64ce688 |
| 3.1 | Python metrics | ee70493 |
| 3.2 | TypeScript/TSX metrics | 8b2f0f3 |
| 3.3 | Conservative call layers | 530418b |
| 4.1 | Paired evidence admission | 17a7ffe |
| 4.2 | Frozen targets and decisions | 9de047f |
| 4.3 | JSON/Markdown reports | 323a4e7 |
| 5.1 | Library workflow and integration tests | 0154340 |
| 5.2 | CLI and binary tests | 4ba40d3 |
| 5.3 | Portable correctness workflow | e6f43d7 |
| 6.1 | External replay preparation | ee7cf32 |
| 6.2 | Fresh ATS replay and negative control | e14b238 |
| 6.3 | Fresh typed provider replay and adverse scope/evidence | d17194c |
| 6.4 | Verified skill recipe | b75aed7 |
| 6.5 | Real TypeScript trial | d596965 |
| 7.1 | Frozen supported self-trial | 3f03b52 |
| 7.2 | Self-trial evidence and exact reviewed source adoption | 366196b |
| 7.3 | Readiness, criterion coverage and adopter indexes | 63d4cfb |

Additional independently reviewed changes:

- b5e0714: upstream scope fixture follows actual filesystem case sensitivity.
- 155f17b: baseline capability inventory stays baseline-bound during candidate comparison.
- 6cb78e6: user-requested pinned nextest/pytest-xdist, CI and test commands.

The test-command/configuration iterations and failed candidate/harness attempts
are preserved in the self-trial record. Final paired evidence uses explicit
configuration/root/source paths and per-worker source provenance.

Latest Windows checks: native nextest 102 passed; paired Python full suites each
504 passed and 11 skipped, with identical 515 identities; live adoption checks
70 passed and 104 subtests passed. Scope and limits are in readiness.md. This
record does not supply missing Linux/macOS results or establish a feature-wide GO.
