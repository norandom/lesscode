//! A task set and its check. A task is a request; the check says whether a
//! reply to it is acceptable. The built-in set is four small Python modules,
//! each with a request in the form of a regenerate turn, judged by the
//! core's own parsing and static checks on a copy in a temporary directory.
//!
//! A task set on disk is a directory with one directory per task. A task's
//! directory is the tree the model is asked to rewrite, with two files
//! beside it: `task.json` (what the module has to do, the files in scope,
//! the targets) and `public-surface.json` (the names that must survive).

use crate::repair::Verdict;
use crate::request::CallRequest;
use crate::structure::unfenced;
use lesscode_core::jsonval::{J, from_serde, to_serde};
use lesscode_core::regenerator::{
    RegenError, RewardEvaluation, TurnOutcome, TurnRequest, build_prompt, evaluate_tier1_static,
    parse_result, prepare_candidate, write_candidate,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The name of the check that `CoreCheck` makes, for reports.
pub const CORE_CHECK: &str = "the core's parsing and static checks on a copy \
                              (parse_result, prepare_candidate, evaluate_tier1_static)";

/// The name of the built-in set, for reports.
pub const BUILT_IN: &str = "built-in";

/// The files of the built-in set, graded: a function every model tried so
/// far rewrites at once (R102), a public name that must survive, a statement
/// target that a careless rewrite misses, state spread over two functions.
const BUILT_IN_FILES: [(&str, &str); 12] = [
    (
        "1-total-price/task.json",
        include_str!("../tasks/1-total-price/task.json"),
    ),
    (
        "1-total-price/public-surface.json",
        include_str!("../tasks/1-total-price/public-surface.json"),
    ),
    (
        "1-total-price/src/prices.py",
        include_str!("../tasks/1-total-price/src/prices.py"),
    ),
    (
        "2-shipping-fee/task.json",
        include_str!("../tasks/2-shipping-fee/task.json"),
    ),
    (
        "2-shipping-fee/public-surface.json",
        include_str!("../tasks/2-shipping-fee/public-surface.json"),
    ),
    (
        "2-shipping-fee/src/shipping.py",
        include_str!("../tasks/2-shipping-fee/src/shipping.py"),
    ),
    (
        "3-discount/task.json",
        include_str!("../tasks/3-discount/task.json"),
    ),
    (
        "3-discount/public-surface.json",
        include_str!("../tasks/3-discount/public-surface.json"),
    ),
    (
        "3-discount/src/discount.py",
        include_str!("../tasks/3-discount/src/discount.py"),
    ),
    (
        "4-orders/task.json",
        include_str!("../tasks/4-orders/task.json"),
    ),
    (
        "4-orders/public-surface.json",
        include_str!("../tasks/4-orders/public-surface.json"),
    ),
    (
        "4-orders/src/orders.py",
        include_str!("../tasks/4-orders/src/orders.py"),
    ),
];

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub request: CallRequest,
}

#[derive(Debug, Clone)]
pub struct TaskSet {
    pub name: String,
    pub check_name: String,
    pub tasks: Vec<Task>,
}

/// A temporary directory that is removed when this is dropped.
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new() -> Result<Scratch, String> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        let dir =
            std::env::temp_dir().join(format!("lesscode-model-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&dir).map_err(|problem| format!("{}: {problem}", dir.display()))?;
        Ok(Scratch(dir))
    }

    pub fn dir(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `task.json` of one task.
#[derive(Deserialize)]
struct TaskFile {
    definitions: String,
    scope_files: Vec<String>,
    targets: BTreeMap<String, i64>,
    #[serde(default = "all_declared")]
    contract: String,
    #[serde(default = "python")]
    language: String,
    #[serde(default = "output_tokens")]
    max_output_tokens: u32,
}

fn all_declared() -> String {
    "all functions declared".to_owned()
}

fn python() -> String {
    "python".to_owned()
}

/// Room for the answer and for the reasoning of a model that reasons.
fn output_tokens() -> u32 {
    8_000
}

/// The core's checks for the tasks of a set that was read from disk. Every
/// reply is judged on a copy of its task in the temporary directory; the
/// task's own directory is only read.
pub struct CoreCheck {
    scratch: Scratch,
    turns: Vec<(String, TurnRequest, J)>,
    copies: usize,
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|problem| format!("{}: {problem}", path.display()))
}

/// One task from its directory: the request the core writes for it, and
/// what the core's checks need to judge a reply.
fn task_in(dir: &Path, id: &str) -> Result<(Task, TurnRequest, J), String> {
    let file = dir.join("task.json");
    let stated: TaskFile = serde_json::from_str(&read(&file)?)
        .map_err(|problem| format!("{}: {problem}", file.display()))?;
    let surface_file = dir.join("public-surface.json");
    let surface = serde_json::from_str(&read(&surface_file)?)
        .map_err(|problem| format!("{}: {problem}", surface_file.display()))?;
    let turn = TurnRequest {
        scope_root: dir.to_string_lossy().into_owned(),
        scope_files: stated.scope_files,
        surface_path: surface_file.to_string_lossy().into_owned(),
        definitions: stated.definitions,
        targets: stated.targets.into_iter().collect(),
        contract: stated.contract,
        acceptance_command: String::new(),
        feedback: None,
        language: stated.language,
    };
    let (system, user) =
        build_prompt(&turn).map_err(|problem| format!("task {id}: {}", problem.message()))?;
    let task = Task {
        id: id.to_owned(),
        request: CallRequest {
            purpose: format!("measure {id}"),
            system,
            user,
            max_output_tokens: stated.max_output_tokens,
            temperature: None,
            structure: None,
        },
    };
    Ok((task, turn, from_serde(&surface)))
}

/// The task set in `dir`, its tasks in the order of their directories'
/// names, with the core's checks for it. `dir` is only read.
pub fn load(dir: &Path, name: &str, scratch: Scratch) -> Result<(TaskSet, CoreCheck), String> {
    let entries =
        std::fs::read_dir(dir).map_err(|problem| format!("{}: {problem}", dir.display()))?;
    let mut ids: Vec<String> = entries
        .filter_map(|entry| Some(entry.ok()?.file_name().to_string_lossy().into_owned()))
        .filter(|id| dir.join(id).join("task.json").is_file())
        .collect();
    ids.sort();
    if ids.is_empty() {
        return Err(format!(
            "{}: no task found; a task is a directory with a task.json",
            dir.display()
        ));
    }
    let mut tasks = Vec::new();
    let mut turns = Vec::new();
    for id in ids {
        let (task, turn, surface) = task_in(&dir.join(&id), &id)?;
        tasks.push(task);
        turns.push((id, turn, surface));
    }
    let set = TaskSet {
        name: name.to_owned(),
        check_name: CORE_CHECK.to_owned(),
        tasks,
    };
    Ok((
        set,
        CoreCheck {
            scratch,
            turns,
            copies: 0,
        },
    ))
}

/// The built-in set, written to a temporary directory and read from there.
pub fn built_in() -> Result<(TaskSet, CoreCheck), String> {
    let scratch = Scratch::new()?;
    let dir = scratch.dir().join("tasks");
    for (name, text) in BUILT_IN_FILES {
        let file = dir.join(name);
        let parent = file.parent().expect("a file of the set has a directory");
        std::fs::create_dir_all(parent)
            .and_then(|()| std::fs::write(&file, text))
            .map_err(|problem| format!("{}: {problem}", file.display()))?;
    }
    load(&dir, BUILT_IN, scratch)
}

impl CoreCheck {
    /// The core's checks on one reply: its parsing, then a fresh copy of the
    /// task with the reply's files, then the static checks on that copy.
    fn evaluate(&mut self, id: &str, text: &str) -> Result<RewardEvaluation, RegenError> {
        let (_, turn, surface) = self
            .turns
            .iter()
            .find(|(known, ..)| known == id)
            .ok_or_else(|| RegenError::FailedTurn(format!("no task {id} in this set")))?;
        let result = parse_result(unfenced(text)).map_err(|problem| {
            RegenError::FailedTurn(format!(
                "the answer is not the JSON object asked for: {problem}"
            ))
        })?;
        self.copies += 1;
        let copy = self.scratch.dir().join(format!("copy-{}", self.copies));
        prepare_candidate(Path::new(&turn.scope_root), &copy, &turn.scope_files)?;
        let written = write_candidate(&result, &copy, &turn.scope_files)?;
        let outcome = TurnOutcome {
            ok: true,
            kind: String::new(),
            detail: None,
            result: Some(result),
            written,
        };
        evaluate_tier1_static(turn, &outcome, &copy, surface, &turn.targets)
    }

    /// What the core's checks say about one reply to `task`, with the check
    /// that rejected it named first.
    pub fn judge(&mut self, task: &Task, text: &str) -> Verdict {
        let reason = match self.evaluate(&task.id, text) {
            Ok(reward) if reward.passed_tier1 => None,
            Ok(reward) => Some(format!(
                "{}: {}",
                reward.failure_kind.unwrap_or_default(),
                to_serde(&reward.diagnostics)
            )),
            Err(error) => Some(error.message().to_owned()),
        };
        Verdict {
            passed: reason.is_none(),
            reasons: reason.into_iter().collect(),
        }
    }
}
