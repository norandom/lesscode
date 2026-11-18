//! An SMT solver run as an external program on SMT-LIB text.
//!
//! The program is named by path and pinned by its SHA-256. Every call keeps
//! its query, its answer and its command line in the output directory. An
//! answer other than `sat` or `unsat` is an error, never a verdict.

use super::process;
use crate::specimen_runner::sha256_hex_bytes;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// One term of an answer: a word, or terms in parentheses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Word(String),
    List(Vec<Term>),
}

impl Term {
    /// The term as SMT-LIB text.
    pub fn text(&self) -> String {
        match self {
            Term::Word(word) => word.clone(),
            Term::List(items) => {
                let items: Vec<_> = items.iter().map(Term::text).collect();
                format!("({})", items.join(" "))
            }
        }
    }
}

/// The terms of an answer, in order. Unbalanced parentheses are an error.
pub fn terms(text: &str) -> Result<Vec<Term>, String> {
    let spaced = text.replace('(', " ( ").replace(')', " ) ");
    let mut open: Vec<Vec<Term>> = vec![Vec::new()];
    for word in spaced.split_whitespace() {
        match word {
            "(" => open.push(Vec::new()),
            ")" => {
                let done = open.pop().filter(|_| !open.is_empty());
                let done = done.ok_or("the answer closes a parenthesis it did not open")?;
                open.last_mut().expect("checked").push(Term::List(done));
            }
            word => open
                .last_mut()
                .expect("never empty")
                .push(Term::Word(word.to_owned())),
        }
    }
    match open.len() {
        1 => Ok(open.remove(0)),
        _ => Err("the answer leaves a parenthesis open".into()),
    }
}

/// The values of a `get-value` answer by name. Each of `names` must be
/// bound exactly once and nothing else may be bound.
pub fn bindings(answer: &Term, names: &[String]) -> Result<BTreeMap<String, String>, String> {
    let Term::List(pairs) = answer else {
        return Err(format!("not a list of values: {}", answer.text()));
    };
    let bad = |term: &Term| format!("not a name and a value: {}", term.text());
    let mut found = BTreeMap::new();
    for pair in pairs {
        let Term::List(items) = pair else {
            return Err(bad(pair));
        };
        let [Term::Word(name), value] = items.as_slice() else {
            return Err(bad(pair));
        };
        if !names.contains(name) || found.insert(name.clone(), value.text()).is_some() {
            return Err(format!("{name} is not asked for or is bound twice"));
        }
    }
    match names.iter().find(|name| !found.contains_key(*name)) {
        Some(missing) => Err(format!("{missing} has no value")),
        None => Ok(found),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Solver {
    pub path: PathBuf,
    pub sha256: String,
    /// What the program says about itself.
    pub version: String,
    #[serde(skip)]
    out: PathBuf,
}

impl Solver {
    /// The program at `path`, which must have the given SHA-256. Its files
    /// go to `out`.
    pub fn pinned(path: &Path, sha256: &str, out: &Path) -> Result<Solver, String> {
        let bytes = fs::read(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        let found = sha256_hex_bytes(&bytes);
        if found != sha256 {
            return Err(format!(
                "{} has SHA-256 {found}, expected {sha256}",
                path.display()
            ));
        }
        let mut solver = Solver {
            path: path.to_owned(),
            sha256: found,
            version: String::new(),
            out: out.to_owned(),
        };
        solver.version = solver.call("solver-version", &["-version"])?;
        Ok(solver)
    }

    fn call(&self, label: &str, args: &[&str]) -> Result<String, String> {
        let mut argv = vec![self.path.to_string_lossy().into_owned()];
        argv.extend(args.iter().map(|arg| (*arg).to_owned()));
        let done = process::run(&argv, &self.out, &BTreeMap::new(), Duration::from_secs(60));
        let command = serde_json::json!({"argv": argv, "exit": done.exit_code,
            "timed_out": done.timed_out, "seconds": done.seconds, "launch_error": done.launch_error});
        let keep = |name: String, text: &str| {
            fs::write(self.out.join(&name), text).map_err(|e| format!("Cannot write {name}: {e}"))
        };
        keep(format!("{label}.txt"), &done.output)?;
        keep(format!("{label}.command.json"), &command.to_string())?;
        if !done.passed() {
            return Err(format!("the solver failed on {label}: {}", done.output));
        }
        Ok(done.output.trim().to_owned())
    }

    /// Run the query `<label>.smt2` and give the terms of the answer.
    pub fn ask(&self, label: &str, query: &str) -> Result<Vec<Term>, String> {
        let file = format!("{label}.smt2");
        fs::write(self.out.join(&file), query).map_err(|e| format!("Cannot write {file}: {e}"))?;
        terms(&self.call(label, &["-smt2", &file])?)
    }

    /// Whether the query, which ends in one `check-sat`, can be satisfied.
    pub fn satisfiable(&self, label: &str, query: &str) -> Result<bool, String> {
        match self.ask(label, query)?.as_slice() {
            [Term::Word(word)] if word == "sat" => Ok(true),
            [Term::Word(word)] if word == "unsat" => Ok(false),
            other => {
                let said: Vec<_> = other.iter().map(Term::text).collect();
                Err(format!(
                    "the solver gave no verdict on {label}: {}",
                    said.join(" ")
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn values_are_bound_by_name_exactly_once() {
        let asked = names(&["a", "b"]);
        let read = |text: &str| bindings(&terms(text).unwrap()[0], &asked);
        let values = read("((b (- 1))\r\n (a true))").unwrap();
        assert_eq!(values["a"], "true");
        assert_eq!(values["b"], "(- 1)");
        for bad in [
            "((a 1))",
            "((a 1) (a 2) (b 1))",
            "((a 1) (b 2) (c 3))",
            "((a 1) b)",
            "a",
        ] {
            assert!(read(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn unbalanced_answers_are_errors() {
        assert_eq!(terms("sat\n(x)").unwrap().len(), 2);
        assert!(terms("(a").is_err());
        assert!(terms("a)").is_err());
    }
}
