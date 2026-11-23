# Brief: lesscode as a harness for reworking a subsystem, with a result a developer can see

Written 2026-11-22 from the owner's direction of that day. A brief, not approved requirements.
Nothing here is built except the first page named under "What exists".

## The problem

The effects the research showed came from reworking whole subsystems of six or seven files
(functions 69 to 50 and 102 to 63, calls 89 to 61 and 145 to 101). The examples made since are
single files, by hand, and move nothing by those measures. A developer has no way to run such a
rework with lesscode alone, and no picture of what it did.

## What the owner wants

1. **Whole subsystems.** Single files do not help, except a made-up spaghetti example over several
   files, which is wanted too.
2. **lesscode as the harness.** Today it measures, plants faults and judges a change. Writing the
   tests and writing the rework were done by an agent outside the tool. The tool should drive the
   loop itself, with model calls where needed.
3. **Faults, tests and reduction hand in hand.** The tests of a real project are only half good.
   Plant faults, let the unnoticed ones drive the writing of tests, then reduce.
4. **Code against better code, side by side**, with a graph and the measures; intuitive, and a
   little artful. Tabs or panels for:
   - structure: functions and calls (the first page shows this);
   - state space: the states that can be written against the states that are reached;
   - planted faults and tests: which faults the tests notice, before and after, and the tests that
     were added.
5. **A command `lesscode-metrics` in this repository.** `kiro-cli` stays as it is. The page is
   embedded in the binary (`include_str!`) and written as one HTML file; no server.
6. **How a developer uses it.** The developer creates a branch in the checked-out project; lesscode,
   or the agent driving it, edits the code there.
7. **Examples people can run** in an examples folder.

## What the page shows: three questions, six lenses

Agreed with the owner on 2026-11-22 as the definition of what one rework is measured by.

| Question | Lens | What is counted | Tool | Expected direction |
| --- | --- | --- | --- | --- |
| Is it less? | Size | tokens, statements | lesscode | falls |
| | Structure | functions, calls, calls across files | lesscode `measure-scope` | falls |
| Is it simpler? | State | places that hold state, places that write it, states that can be written against states that are reached, routines that decide on it | new in `lesscode-metrics`; `kiro-cli` flags one kind today | falls |
| | Local complexity | per routine: cyclomatic, nesting, parameters, length; per type: methods, fields | `kiro-cli` | no routine above its limit gets worse |
| Is it still safe? | Behaviour kept | the project's tests pass; decisions in total stay about the same | the project's tests, `kiro-cli` sum | stays |
| | Protection | planted faults noticed and unnoticed per function, tests added | lesscode `step` | unnoticed faults do not rise, and fall where tests were added |

- **Each lens has its own direction.** Size, structure and state should fall. Local complexity is a
  guard: the routine that takes over the decisions grows, and its limit stops it from growing too
  far. Decisions in total should stay; a large fall means behaviour was removed.
- **No lens stands alone.** Fewer functions with one huge routine is not simpler; less code with
  more unnoticed faults is not safe. The page shows all six for one rework.
- **Cost is on the page too**: model spend and run time per rework.

How the lenses map to the page:

- Structure view: the bowls. Each dot gets a ring for local complexity, so the routine that grew is
  visible, and an open part of the ring for its share of unnoticed faults.
- State view: a grid, one cell per writable state, lit where the code can reach it.
- Faults and tests view: one mark per planted fault per function, filled if a test notices it;
  added tests next to the marks they filled.
- A strip under all views: the six lenses, each with before, after, and whether it moved in its
  expected direction.

Division of work: `kiro-cli` measures routines, types and files and keeps the limits.
`lesscode-metrics` measures the subsystem as a whole, the state and the faults, reads `kiro-cli`'s
report for the local-complexity lens, and writes the page. Without `kiro-cli` that lens is shown as
not measured.

## How it is adopted (owner, 2026-11-23)

Through an agent. The tool measures and judges by exit code and contains no model. An agent that
follows a skill reads the measurements and writes the tests and the rework. The person sets the
limits, decides on public functions and merges, and runs the real program where a device is needed.
The skill is [untangle-state](../../../.agents/skills/untangle-state/SKILL.md); it holds the loop as
it was run on 23 November and, in general terms, the cut for code that cannot be tested as it
stands. This replaces the earlier reading of "lesscode as a harness by itself" as a tool that
writes tests and reworks through model calls of its own.

## Sources

- `D:\Source\drift-rdp` at its head: may be edited and its code may be copied into examples.
- lesscode itself.
- `D:\Source\ats-agent` (`.cjs`): not yet checked whether a language pack reads it.
- `D:\Source\facdrone` (Python, finance code): read only; counts and pictures, no source.

## What exists

- `deconvolute measure-scope` and `analyze`: functions, calls, write sites.
- `step check`, `pin`, `record`, `guard`: planted faults per function before and after, and the
  judgement of a change. The verdict lists noticed and unnoticed faults per function, and the
  unnoticed ones by line and kind. It does not record which test noticed which fault.
- `lesscode-model`: model calls with repair turns, used for writing language packs.
- A first page, "Two Bowls" (https://claude.ai/artifact/6ggrijcAhxUDyiEk3gw8SY), hand-assembled from
  the frozen measurements of the two facdrone reworks. Structure only.
- A census of `drift-rdp` at `09b4ce8`
  (`D:\BuildArtifacts\lesscode\state-consolidation\r164\drift-rdp-09b4ce8\census.json`).

## What is missing

- A count of state holders and of writable states, for any language.
- A count of reached states (solver, or a walk over short sequences of events).
- A command that writes tests for unnoticed faults and admits them through `step pin`.
- A command that writes the rework.
- Which test notices which fault.
- The page as an output of a command.

## Open questions for the owner

- Which model writes tests and reworks, and under which spending limit.
- Whether the first run is `renderer` in `drift-rdp` (8 files, 70 functions, 91 calls, 13 state
  cells, 3 machines found).
- Whether `step check` on a subsystem may take hours: every planted fault needs a build of the
  project.

## Related

- Guide: [docs/state-refactoring.md](../../../docs/state-refactoring.md); it describes single-machine
  steps and will need rewriting for subsystems.
- [state-consolidation](../state-consolidation/brief.md): the finder for state machines.
- Research index: [docs/research/README.md](../../../docs/research/README.md).
