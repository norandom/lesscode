//! Research census only: never an admitted simplicity evaluator or acceptance judge.
use lesscode_core::adapters::driver::tree::{
    first_error, grammar, kind_is_valid, parse, walk, SgNode,
};
use lesscode_core::specimen_runner::sha256_hex_bytes;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};
mod bindings;
mod identity;
mod predicates;

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
    predicates: predicates::Rules,
    bindings: bindings::Rules,
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
    let extra = profile
        .predicates
        .conditions
        .iter()
        .map(|r| &r.kind)
        .chain(profile.predicates.boolean.iter())
        .chain(profile.predicates.comparisons.iter())
        .chain(profile.predicates.wrappers.iter())
        .chain(profile.predicates.unary_roots.iter())
        .chain(profile.bindings.assignments.iter())
        .chain(profile.bindings.lexical_functions.iter())
        .chain([
            &profile.bindings.identifier,
            &profile.bindings.global_declaration,
            &profile.bindings.nonlocal_declaration,
        ]);
    for kind in extra {
        if !kind_is_valid(lang, kind) {
            return Err(format!("unknown grammar kind: {kind}"));
        }
    }
    for rule in &profile.predicates.conditions {
        if rule.field.is_some() == rule.named_child.is_some() {
            return Err("condition requires exactly one selector".into());
        }
    }
    let tree = parse(source, lang)?;
    if let Some((line, col)) = first_error(&tree.root()) {
        return Err(format!("parse error at {line}:{col}"));
    }
    let mut facts = Vec::new();
    let owners = identity::index(&tree.root(), profile);
    for node in walk(&tree.root()) {
        for (role, kinds) in roles {
            if has(kinds, &node) {
                facts.push(Fact {
                    role: role.into(),
                    owner: identity::nearest(&node, &owners),
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
    let tree = parse(source, grammar(&profile.language)?)?;
    let predicates = predicates::measure(&tree.root(), &profile)?;
    let bindings = bindings::classify(&tree.root(), &profile);
    let mut counts = BTreeMap::<&str, usize>::new();
    for fact in &facts {
        *counts.entry(&fact.role).or_default() += 1;
    }
    let report = serde_json::json!({
        "schema": "syntax-census-v1", "profile": profile.version,
        "profile_sha256": sha256_hex_bytes(&profile_bytes),
        "source_sha256": sha256_hex_bytes(&source_bytes),
        "source_path": args[1], "counts": counts, "facts": facts,
        "predicate_count": predicates.len(), "predicates": predicates,
        "bindings": bindings,
        "simplicity_status": "not_measurable",
        "limits": ["Draft structural profile, not admitted for comparisons", "Attribute/subscript aliases and nonlocal bindings unresolved; declaration renames/reordering need explicit correspondence", "Predicate extraction does not resolve opaque calls or implicit iteration decisions", "Binding facts do not establish target dispersion or complete effect coverage"]
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
        let mut p = profile();
        p.predicates.conditions[0].kind = "unknown_condition_kind".into();
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

    #[test]
    fn lexical_owners_survive_formatting_and_disambiguate_redefinitions() {
        let owners = |text| {
            census(text, &profile())
                .unwrap()
                .into_iter()
                .filter(|f| f.role == "unresolved_effect")
                .map(|f| f.owner)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            owners("def f():\n x=1\n"),
            owners("# note\ndef f():\n    renamed = 1\n")
        );
        let duplicate = owners("def f():\n x=1\ndef f():\n x=2\n");
        assert_ne!(duplicate[0], duplicate[1]);
    }
}
