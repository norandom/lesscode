//! Research census only: never an admitted simplicity evaluator or acceptance judge.
use lesscode_core::adapters::driver::tree::{
    ancestors, first_error, grammar, kind_is_valid, parse, walk, SgNode,
};
use lesscode_core::specimen_runner::sha256_hex_bytes;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    version: String,
    language: String,
    owners: Vec<String>,
    branches: Vec<String>,
    decisions: Vec<String>,
    handlers: Vec<String>,
    effects: Vec<String>,
    unsupported: Vec<String>,
}

#[derive(Serialize, Debug, PartialEq)]
struct Fact {
    role: String,
    owner: String,
    kind: String,
    line: usize,
    start: usize,
    end: usize,
    text: String,
}

fn has(kinds: &[String], node: &SgNode<'_>) -> bool {
    kinds.iter().any(|kind| kind == node.kind().as_ref())
}

fn owner(node: &SgNode<'_>, profile: &Profile) -> String {
    let mut names: Vec<_> = ancestors(node)
        .into_iter()
        .filter(|p| has(&profile.owners, p))
        .map(|p| {
            let name = p
                .field("name")
                .map(|n| n.text().into_owned())
                .unwrap_or_else(|| p.kind().into_owned());
            // Byte identity is deliberately provisional, not a cross-revision match.
            format!("{}@{}", name, p.range().start)
        })
        .collect();
    names.reverse();
    if names.is_empty() {
        "<module>".into()
    } else {
        names.join("::")
    }
}

fn census(source: &str, profile: &Profile) -> Result<Vec<Fact>, String> {
    let lang = grammar(&profile.language)?;
    let roles = [
        ("owner", &profile.owners),
        ("branch_site", &profile.branches),
        ("decision_expression", &profile.decisions),
        ("exception_handler", &profile.handlers),
        ("unresolved_effect", &profile.effects),
        ("unsupported", &profile.unsupported),
    ];
    for (_, kinds) in roles {
        for kind in kinds {
            if !kind_is_valid(lang, kind) {
                return Err(format!("unknown grammar kind: {kind}"));
            }
        }
    }
    let tree = parse(source, lang)?;
    if let Some((line, col)) = first_error(&tree.root()) {
        return Err(format!("parse error at {line}:{col}"));
    }
    let mut facts = Vec::new();
    for node in walk(&tree.root()) {
        for (role, kinds) in roles {
            if has(kinds, &node) {
                facts.push(Fact {
                    role: role.into(),
                    owner: owner(&node, profile),
                    kind: node.kind().into_owned(),
                    line: node.start_pos().line() + 1,
                    start: node.range().start,
                    end: node.range().end,
                    text: node.text().into_owned(),
                });
            }
        }
    }
    Ok(facts)
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: simplicity-spike PROFILE SOURCE".into());
    }
    let profile_bytes = fs::read(&args[0]).map_err(|e| e.to_string())?;
    let profile: Profile = serde_json::from_slice(&profile_bytes).map_err(|e| e.to_string())?;
    let source_bytes = fs::read(&args[1]).map_err(|e| e.to_string())?;
    let source = std::str::from_utf8(&source_bytes).map_err(|e| e.to_string())?;
    let facts = census(source, &profile)?;
    let mut counts = BTreeMap::<&str, usize>::new();
    for fact in &facts {
        *counts.entry(&fact.role).or_default() += 1;
    }
    let report = serde_json::json!({
        "schema": "syntax-census-v1", "profile": profile.version,
        "profile_sha256": sha256_hex_bytes(&profile_bytes),
        "source_sha256": sha256_hex_bytes(&source_bytes),
        "source_path": args[1], "counts": counts, "facts": facts,
        "simplicity_status": "not_measurable",
        "limits": ["Raw syntax census only; no admitted metric profile", "No binding/alias resolution or cross-revision identity", "Decision expressions are not atomic predicate counts", "All selected effects unresolved; no local/write/dispersion claim"]
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn profile() -> Profile {
        serde_json::from_str(include_str!("../profile.json")).unwrap()
    }
    #[test]
    fn retained_logic_and_effects_are_visible() {
        let facts = census(
            "def f(a, b):\n    if a and b:\n        a.value = b\n    return a.update(b)\n",
            &profile(),
        )
        .unwrap();
        assert_eq!(facts.iter().filter(|f| f.role == "branch_site").count(), 1);
        assert_eq!(
            facts
                .iter()
                .filter(|f| f.role == "decision_expression")
                .count(),
            1
        );
        assert_eq!(
            facts
                .iter()
                .filter(|f| f.role == "unresolved_effect")
                .count(),
            2
        );
    }
    #[test]
    fn syntax_errors_are_not_empty_successes() {
        assert!(census("def f(:", &profile()).is_err());
    }
    #[test]
    fn unknown_selector_is_rejected() {
        let mut p = profile();
        p.branches.push("not_a_grammar_kind".into());
        assert!(census("x = 1", &p).is_err());
    }
    #[test]
    fn same_name_methods_have_separate_owners() {
        let f = census(
            "class A:\n def f(self):\n  x = 1\nclass B:\n def f(self):\n  x = 1\n",
            &profile(),
        )
        .unwrap();
        let owners: Vec<_> = f
            .iter()
            .filter(|f| f.role == "unresolved_effect")
            .map(|f| &f.owner)
            .collect();
        assert_eq!(owners.len(), 2);
        assert_ne!(owners[0], owners[1]);
    }
}
