# Local adoption: hypotheses recorded before the feasibility probe

## P01: Can the existing standard-library proxy matcher replace custom rules?

Selection is manual, from mvibe's installed-capability inventory and the
custom rule family in `mvibe_auth/http.py`. This is a replacement-feasibility
probe, not a full rewrite trial or a general efficacy benchmark.

Hypothesis: replace custom NO_PROXY rule interpretation with the existing
`urllib.request.proxy_bypass_environment` capability, retaining CIDR, host,
subdomain, scheme, port and IPv6 behavior, so the supporting match/parse helpers
can disappear. Keep the original as an alternative. Dependencies remain frozen.

Before rewriting, compare the original pure rule helper and a concrete stdlib
candidate on explicit contract cases. Isolate the original helper family using
Python AST selection; do not import the application or load its environment.
Bind the source to the measured snapshot. Record each original and candidate
outcome, including failed equivalence. Stop this alternative on a mismatch.

The potential whole replacement would include `_should_bypass_proxy`, all five
rule helpers, `_EnvProxyTransport` callers and relevant auth-config/proxy tests.
No production saving is admitted from a helper-only probe. No estimate of that
whole replacement's size or maintenance benefit is asserted in advance.

Disproof: any changed routing decision, lost validation or changed error behavior
rejects the proposed drop-in replacement. Passing these finite cases would only
justify a fuller paired trial, not establish universal equivalence.

## R01: Repeated breadcrumb publication in mmhmem

The three `crumb` helpers in memory, boilerplate and shunt services repeat task
fallback, optional bus handling, breadcrumb publication and warning behavior.
The existing `SessionBus::breadcrumb` API is already shared. Investigate whether
putting the common optional-publication policy at that existing boundary removes
the wrappers and duplicated policy without adding another service abstraction.

Disposition at selection: deferred; no candidate implementation or savings claim.
Count the complete services and changed callers, including the session bus,
adapter cost and unchanged behavioral tests. Preserve task fallback, optional-bus
semantics, event order, error suppression and the exact observable warning policy.
Reject if a generic wrapper merely relocates the code or net reduction is marginal.
