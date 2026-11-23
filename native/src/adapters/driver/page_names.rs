//! Leaving the names of a project out of the run inside a report
//! (`.kiro/specs/run-dashboard`, task 7.1; requirement 8.4).
//!
//! The files, functions and types the run names get a number each, the same
//! one wherever the name stands, so that the report still shows which row
//! belongs to which. In a text of the run a file's name is replaced wherever
//! it stands and the name of a function or type where it stands as a word of
//! its own. The project's commands, the files the run wrote, the shapes of a
//! type and the line of source of the example fault are left out as a whole.
//! The tool's own sentences are not searched: a function named like a common
//! word would otherwise take that word out of them.

use super::page_list::project_of;
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap};

const COMMAND: &str = "a command of the project, left out";
const WRITTEN: &str = "a file the run wrote, name left out";

/// Keys whose text is the tool's own or was already dealt with.
const OWN: [&str; 20] = [
    "title",
    "what",
    "next",
    "id",
    "status",
    "side",
    "started",
    "heartbeat",
    "outcome",
    "tool",
    "command",
    "commands",
    "writes_in_repository",
    "branch",
    "commit",
    "compared_with",
    "repository",
    "directory",
    "shape",
    "what_was_changed",
];
/// What a row of a lens says in the tool's words, unless it was handed in.
const ROW: [&str; 4] = ["lens", "measure", "expected", "note"];

/// The names a run has, by kind.
#[derive(Default)]
struct Found {
    files: BTreeSet<String>,
    functions: BTreeSet<String>,
    types: BTreeSet<String>,
}

impl Found {
    fn read(&mut self, key: &str, value: &Value) {
        match value {
            Value::String(name) => self.name(key, name),
            Value::Array(edges) if key == "edges" => edges.iter().for_each(|e| self.edge(e)),
            Value::Array(items) => {
                // Under `functions` stand names or rows; a row has its own keys.
                let key = if key == "functions" { "function" } else { "" };
                items.iter().for_each(|item| self.read(key, item));
            }
            Value::Object(map) => map.iter().for_each(|(key, value)| self.read(key, value)),
            _ => {}
        }
    }

    fn name(&mut self, key: &str, name: &str) {
        let kind = match key {
            "file" => &mut self.files,
            "function" => &mut self.functions,
            "type" => &mut self.types,
            _ => return,
        };
        kind.insert(name.to_owned());
    }

    /// A call: caller file, caller, callee file, callee.
    fn edge(&mut self, edge: &Value) {
        let ends = edge.as_array().map(Vec::as_slice).unwrap_or_default();
        for (at, end) in ends.iter().enumerate() {
            let key = if at % 2 == 0 { "file" } else { "function" };
            self.read(key, end);
        }
    }
}

/// A name, what stands for it, and whether it counts only as a whole word.
type Swap = (String, String, bool);

struct Names {
    exact: HashMap<String, String>,
    /// The longest name first, so that a name inside another is not taken.
    free: Vec<Swap>,
    /// The run was accepted: its last sentence is the tool's own.
    accepted: bool,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Whether `text[at..at + len]` has no letter, digit or `_` on either side.
fn alone(text: &str, at: usize, len: usize) -> bool {
    let before = text[..at].chars().next_back();
    let after = text[at + len..].chars().next();
    !before.is_some_and(is_word) && !after.is_some_and(is_word)
}

impl Names {
    fn of(run: &Value) -> Names {
        let mut found = Found::default();
        found.read("", run);
        let kinds = [
            ("file", found.files, false),
            ("function", found.functions, true),
            ("type", found.types, true),
        ];
        let (mut exact, mut free) = (HashMap::new(), Vec::new());
        for (kind, names, whole) in kinds {
            let names = names.into_iter().filter(|name| !name.is_empty());
            for (at, name) in names.enumerate() {
                let with = format!("{kind} {}", at + 1);
                exact.entry(name.clone()).or_insert_with(|| with.clone());
                free.push((name, with, whole));
            }
        }
        free.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
        Names {
            exact,
            free,
            accepted: run.get("exit_code") == Some(&json!(0)),
        }
    }

    /// The name that begins at `at`, if one does.
    fn at<'a>(&'a self, text: &str, at: usize, words: bool) -> Option<&'a Swap> {
        self.free.iter().find(|(name, _, whole)| {
            (words || !whole)
                && text[at..].starts_with(name.as_str())
                && (!whole || alone(text, at, name.len()))
        })
    }

    /// `text` without the names; those of functions and types only where
    /// `words` says so.
    fn plain(&self, text: &str, words: bool) -> String {
        if let Some(with) = self.exact.get(text).filter(|_| words) {
            return with.clone();
        }
        let (mut out, mut at) = (String::with_capacity(text.len()), 0);
        while let Some(c) = text[at..].chars().next() {
            match self.at(text, at, words) {
                Some((name, with, _)) => {
                    out.push_str(with);
                    at += name.len();
                }
                None => {
                    out.push(c);
                    at += c.len_utf8();
                }
            }
        }
        out
    }

    /// A command of the tool keeps its words and loses the names of files;
    /// a command of the project is left out.
    fn command(&self, row: &mut Value, tool: &str) {
        let Some(command) = row.get_mut("command") else {
            return;
        };
        match command.as_str() {
            Some(text) if text.split(' ').next() == Some(tool) => {
                *command = self.plain(text, false).into();
            }
            Some(_) => *command = COMMAND.into(),
            None => {}
        }
    }

    /// What is left out as a whole.
    fn withhold(&self, run: &mut Value, project: &str) {
        let tool = run.pointer("/tool/name").and_then(Value::as_str);
        let tool = tool.unwrap_or_default().to_owned();
        items(run, "/commands").for_each(|c| *c = json!([COMMAND]));
        items(run, "/writes_in_repository").for_each(|file| *file = WRITTEN.into());
        if let Some(text) = run.pointer_mut("/faults/example/text") {
            *text = "".into();
        }
        items(run, "/report/lenses").for_each(|row| self.command(row, &tool));
        for kind in items(run, "/report/shapes") {
            self.command(kind, &tool);
            for (at, cell) in items(kind, "/cells").enumerate() {
                if let Some(shape) = cell.get_mut("shape") {
                    *shape = format!("shape {}", at + 1).into();
                }
            }
        }
        if let Some(repository) = run.pointer_mut("/subject/repository") {
            let said = format!("{project}; the names of its files and functions are left out");
            *repository = said.into();
        }
    }

    fn rewrite(&self, value: &mut Value, own: bool) {
        match value {
            Value::String(text) if !own => *text = self.plain(text, true),
            Value::Array(items) => items.iter_mut().for_each(|item| self.rewrite(item, own)),
            Value::Object(map) => {
                let row = map.get("handed_in") == Some(&Value::Bool(false));
                for (key, value) in map.iter_mut() {
                    let key = key.as_str();
                    let tools = OWN.contains(&key)
                        || (row && ROW.contains(&key))
                        || (self.accepted && key == "outcome_detail");
                    self.rewrite(value, own || tools);
                }
            }
            _ => {}
        }
    }
}

/// The items of the list at `at`; none where there is no list.
fn items<'a>(value: &'a mut Value, at: &str) -> impl Iterator<Item = &'a mut Value> {
    let list = value.pointer_mut(at).and_then(Value::as_array_mut);
    list.into_iter().flatten()
}

/// Takes the names of files, functions and types out of a run.
pub fn leave_out(run: &mut Value) {
    let names = Names::of(run);
    let project = project_of(run).unwrap_or_else(|| "a project".to_owned());
    names.withhold(run, &project);
    names.rewrite(run, false);
}
