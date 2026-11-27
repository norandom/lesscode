//! The dashboard: everything that shows a run or a set of runs, and nothing else.
//!
//! Governed by `.kiro/specs/run-dashboard/design.md` and `.kiro/specs/report-set/design.md`
//! ("The border between core and dashboard"). Core measures, plants faults, runs and judges; it
//! holds no page text and no drawing. The commands under `step` decide what to show and call into
//! this module; nothing here names `step`. Of core, only the batch runner and the command runner
//! name this module: they tell the watch what happens in a run. `tests/dashboard_border.rs` keeps
//! these rules. The page itself is the text under `native/page/`.

pub mod list;
pub mod log;
pub mod names;
pub mod open;
pub mod report;
pub mod serve;
pub mod snapshot;
pub mod subject;
pub mod text;
pub mod watch;
