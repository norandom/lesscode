# Example: three locks of a batch runner that were one progress

Source: this repository, `native/src/adapters/driver/runner.rs`. Before is commit `6da5952`; after is
the branch `step/batch-progress`. The step was judged by `lesscode-core` 2.1.0 on its own source:
[step.json](step.json), [verdict.json](verdict.json). The whole walk, with the commands, is in
[Untangling scattered state](../../state-refactoring.md). Research record:
[R161](../../research/2026-11-18/27-batch-progress-dogfood.md).

## What the code does

`deconvolute run` plants faults in copies of a source tree. Several workers take jobs from a queue,
collect results, and stop when one of them cannot restore its copy or the run is interrupted.

## Before

```rust
struct Shared<'a> {
    queue: Mutex<VecDeque<Job>>,
    results: Mutex<Vec<MutantResult>>,
    /// A failed restoration: the run stops and is void.
    fatal: Mutex<Option<String>>,
    // ...
}

// in worker
loop {
    if self.fatal.lock().unwrap().is_some() {
        break;
    }
    let Some(job) = self.queue.lock().unwrap().pop_front() else {
        break;
    };
    // ...
}

// in run_batch, at the end
let left = shared.queue.lock().unwrap().len();
let fatal = stopped.or_else(|| shared.fatal.lock().unwrap().clone());
let mut mutants = shared.results.into_inner().unwrap();
```

## After

```rust
struct Shared<'a> {
    progress: Mutex<Progress>,
    // ...
}

struct Progress {
    queue: VecDeque<Job>,
    results: Vec<MutantResult>,
    fatal: Option<String>,
}

impl Progress {
    /// The next mutant to run; none once the run has stopped.
    fn next(&mut self) -> Option<Job> {
        if self.fatal.is_some() {
            return None;
        }
        self.queue.pop_front()
    }
}

// in worker
loop {
    let Some(job) = self.progress().next() else {
        break;
    };
    // ...
}

// in run_batch, at the end
let progress = shared.progress.into_inner().unwrap();
```

## What changed, counted

| Measure | Before | After |
| --- | --- | --- |
| Locked values the workers share | 3 | 1 |
| Questions a worker asks before a job | 2, under two locks | 1, under one lock |
| Tokens in the file (`step check`) | 2,761 | 2,757 |
| Branch conditions in the file (`deconvolute analyze`) | 13 | 13 |
| Planted faults the tests do not notice, in the functions of the step | 13 | 13 |

## Complexity before and after

Measured with `kiro-cli` 1.1.0 ([R162](../../research/2026-11-18/28-complexity-before-after-kiro-cli.md)).

| Measure | Before | After |
| --- | --- | --- |
| Decisions in the file (cyclomatic sum minus routines) | 24 | 24 |
| Cyclomatic, `worker` | 7 | 6 |
| Cyclomatic, new `next` | | 2 |
| Code lines, `run_batch` (limit 60) | 60 | 61 |

The last row blocked the step in `kiro-cli code check --changed`, after our own `step check` had
accepted it. `run_batch` was shortened afterwards and is 56 lines.

## What our own tool said

- The first `step check` refused the step after one minute: the tests did not pass on the unchanged
  code. The declaration had not built an example program the tests start. That was a fault of the
  declaration, and the refusal was right.
- The second `step check` accepted it after 14 minutes: in scope, no growth, tests pass on both
  sides, and in no function are more faults unnoticed than before. `step guard` passes on the branch.
- 13 planted faults in these functions go unnoticed, before and after. One of them is dropping the
  statement that puts a job back into the queue after a failed run. No pin was written for it.

## What this does not show

- Equal behaviour under real concurrency. One lock closes a gap between "has the run stopped" and
  "take a job" that two locks left open; no test observes that gap in either version.
- The branch is not merged. That is the owner's decision.
- The research finder reported two machines and a shared value before. After the change it reports
  one shared value and no machine, because the state is in plain fields now. The machine is still
  there, in `Progress`.
