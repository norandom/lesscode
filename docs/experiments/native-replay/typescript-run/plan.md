# TS01: degradation notices using native string operations

Selection began 2026-11-01T16:43:13+01:00. This plan was written before
editing any candidate. The selection budget is 30 minutes; the edit/validation
budget is two hours. Stop with an explicit incomplete result if required evidence
cannot finish. No dependencies, upstream changes, feature cuts or success quota.

The bounded local survey used `rg --files D:/Source -g package.json` with
node_modules, .git, target, environments, vendor and dist exclusions. Eight
manifests were found, including Lesscode's analyzer tooling. ATS has JavaScript
release tooling, NPU and Facdrone have no discovered TypeScript project manifest.
Cloudflare has typed tests but its inspected domain helpers are already direct;
mc-sach.de has JavaScript tooling. mvibe's typed package uses Bun-specific checks.
OpenUltraCode has installed TypeScript/tsx, Node tests and a strict existing build.
QOpenBB's manifest is browser verification infrastructure. These are manual
selection observations, not a census of every language or ignored source file.

Selected: OpenUltraCode's complete degradation-notice module, all its exported
notice constructors, summary function, internal helpers and imported type module.
The wider accounting root retains all source, existing tests, scripts, plugin
source and manifest/lock/compiler settings copied from this snapshot. Source
identity, revision, dirty state and per-file hashes will be recorded by the runner.
Only isolated copies under Lesscode's `.runs/` may change.

Hypothesis: replace the custom ASCII slug-building loop in `src/degradation.ts`
with JavaScript `toLowerCase`, regular-expression replacement and empty fallback,
retaining ASCII-only alphanumeric identifiers, separator collapsing, unknown
fallback, Unicode lowercase behavior, all notice fields and summary priority.
Remove the complete loop and its flags without adding helpers, tables or wrappers.
Keep all useful exported types and names. Alternatives: retain original; native
string operations. No shared abstraction or dependency is needed. This is one
single-operation opportunity within a real complete capability, not a synthetic
efficacy example. Expected reduction is modest and local; no project-wide or
maintenance-effort claim follows.

Native source target: at least one fewer production logical statement with
physical lines, statements, decisions and functions frozen from native TS
definitions. Call layers remain contextual, including unresolved external calls;
their unavailability must not be represented as zero. All changed source counts
toward the replacement. Production, tests and experiment support remain separate.

Checks frozen before editing: unchanged existing `tests/degradation.test.ts`
through the installed tsx Node loader; unchanged existing strict `tsc --noEmit`
project build using the installed Node types (absolute typeRoots is environment
binding only); additional frozen public-API differential corpus covering ASCII,
Unicode, separators, empty strings, notice fields and summary precedence. A known
separator-handling mutant must fail the corpus. This is finite evidence, not
universal equivalence. No live opencode host, permissions interaction or external
service execution is claimed. No adoption; baseline remains upstream.
