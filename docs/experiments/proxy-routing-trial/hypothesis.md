# P02: Consolidate proxy rule interpretation without changing routing

Recorded before candidate edits, 2026-11-01. This follows P01's rejected
stdlib-only replacement; it is a second alternative for the same responsibility,
not an independent repository sample.

Replace duplicated IP interpretation and URL-rule parsing in mvibe's
`mvibe_auth/http.py` with a shared use of existing `ipaddress` and `urlparse`
primitives. Preserve wildcard, CIDR, exact-IP, leading-dot, scheme, port,
malformed-rule exceptions, short-circuit order and transport selection/closure.
The original remains the reference; no dependency additions or upgrades.

Manually selected efficacy candidate: two near-identical IP helpers and two
URL parsing branches maintain the same mechanics. Unlike P01, retain the
application-specific rules instead of replacing them with incompatible defaults.
Expected removal: one IP helper and a duplicated URL-port parsing branch.
No fixed reduction target; reject changed behavior or merely denser formatting.

Boundary: complete environment proxy-routing responsibility, from client setup
through `_EnvProxyTransport`, normalization and all rule helpers. Report the
entire `http.py` module and project production footprint too, so helpers/callers
cannot escape accounting. Existing consumers and dependencies remain frozen.

Copy tracked working-tree files into ordinary isolated before/after directories.
Hash source, tests and lockfiles. Do not edit or commit to mvibe. Before candidate
edits, add a frozen differential corpus and transport observations including
invalid rules, IPv4/IPv6, scoped IPv6, default/explicit ports, case and Unicode.
Compare return values and exception type/message. Run unchanged repository tests
on both copies with uv and pytest-xdist; report failures/skips rather than hiding
them. Include a deliberately incorrect routing mutation to challenge the oracle.

Budget: one responsibility, bounded local tests and differential exploration.
If a counterexample appears, retain it and refine this alternative; do not weaken
the reference behavior or count revisions as independent trials. This is finite
testing plus inline review, not universal equivalence or an SMT proof. No
maintenance-effort claim without a separate change exercise.
