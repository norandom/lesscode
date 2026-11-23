# Brief: a page that shows a run while it happens, and its result afterwards

Written 2026-11-23 from the owner's words of that day. The requirements are in
[requirements.md](requirements.md) and wait for the owner's approval.

## The owner's words

> can we build these cool dashboards you make inside of lesscode, with a go:embed style dash?

> can we also open it up during the tool runs and show the progress. I am thinking of meaningful
> step by step, progress bars and a friendly guidance way. assuming developers find their code base
> precious and want to see what happens. we should assure that we work on a branch and not messing
> things up, explain what fault injection is, but also make clear, that if there are not tests,
> things will break

## The problem

A developer who runs lesscode on a codebase they care about sees a terminal that is silent for
twenty minutes to two hours and then prints a verdict. They cannot see what is being done to their
code, whether their checkout is safe, what a planted fault is, or why a run without tests means
nothing. Afterwards the result is a JSON receipt. The one page that shows a result well, "Two
Bowls", was assembled by hand and is not in the repository.

## What is wanted

One page, part of the binary, that a command writes and can open. While a run goes on it shows
which step is running, how far it is, and in plain words what that step does to the code and what
it does not. When the run ends the same page is the report. Several reports can be listed side by
side as a gallery of improvements.

## What Rust has in place of `go:embed`

`include_str!` and `include_bytes!` are part of the language: they put a file's contents into the
binary when it is built. The research programs of 23 November already use `include_str!` for their
templates. For a whole folder of assets there are crates (`rust-embed`, `include_dir`); one page
with its style and script inline does not need them.

## Questions for the design, not decided here

1. **How the page gets fresh data during a run.** Two ways, both without a new dependency:
   - the tool rewrites a small data file beside the page and the page loads it again every second
     (as a script, which a browser allows for a page opened from disk). Nothing listens on the
     machine, and the page still works after the tool has ended;
   - the tool serves the page and the data on `127.0.0.1` with the standard library's own listener
     while it runs. Simpler for the page, and it ends with the tool.
   My recommendation is the first. It has to be shown in a prototype before the design is fixed.
2. **Where the state numbers come from.** The six lenses need the count of state holders and the
   shapes the tests see. Those are research programs today, for Rust only. Until they are part of
   the released tool the page shows those lenses as not measured.
3. **Size.** The native crate has size budgets per file. The page's text is data, not code, but the
   module that fills it counts.

## Relations

- [lesscode-metrics brief](../lesscode-metrics/brief.md): the six lenses and how lesscode is adopted.
- Skill [untangle-state](../../../.agents/skills/untangle-state/SKILL.md): the loop whose steps the page shows.
- Research R164 to R173, [docs/research/2026-11-23/](../../../docs/research/2026-11-23/): the first data.
- Steering [prototype-first](../../steering/prototype-first.md): a prototype gate before tasks.
