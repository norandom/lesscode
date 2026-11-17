//! Structural predicate accounting. No purity or semantic equivalence inference.
use super::{has, Profile};
use lesscode_core::adapters::driver::tree::{walk, SgNode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    pub kind: String,
    pub field: Option<String>,
    pub named_child: Option<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub conditions: Vec<Condition>,
    pub boolean: Vec<String>,
    pub comparisons: Vec<String>,
    pub wrappers: Vec<String>,
    pub unary_roots: Vec<String>,
}

#[derive(Serialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Predicate {
    pub start: usize,
    pub end: usize,
    role: String,
    ordinal: usize,
}

fn emit(node: &SgNode<'_>, role: &str, ordinal: usize, out: &mut BTreeSet<Predicate>) {
    out.insert(Predicate {
        start: node.range().start,
        end: node.range().end,
        role: role.into(),
        ordinal,
    });
}

fn expand(node: &SgNode<'_>, rules: &Rules, out: &mut BTreeSet<Predicate>) -> Result<(), String> {
    let named: Vec<_> = node.children().filter(|c| c.is_named()).collect();
    if has(&rules.boolean, node) {
        if named.len() != 2 {
            return Err("boolean selector requires two operands".into());
        }
        emit(node, "short_circuit", 0, out);
        for child in named {
            expand(&child, rules, out)?;
        }
    } else if has(&rules.comparisons, node) {
        if named.len() < 2 {
            return Err("comparison selector requires operands".into());
        }
        for index in 1..named.len() {
            emit(node, "comparison", index, out);
        }
    } else if has(&rules.wrappers, node) {
        if named.len() != 1 {
            return Err("predicate wrapper requires one operand".into());
        }
        expand(&named[0], rules, out)?;
    } else {
        emit(node, "atom", 0, out);
    }
    Ok(())
}

pub fn measure(root: &SgNode<'_>, profile: &Profile) -> Result<Vec<Predicate>, String> {
    let rules = &profile.predicates;
    let mut out = BTreeSet::new();
    for node in walk(root) {
        if has(&rules.boolean, &node)
            || has(&rules.comparisons, &node)
            || has(&rules.unary_roots, &node)
        {
            expand(&node, rules, &mut out)?;
        }
        for rule in &rules.conditions {
            if node.kind().as_ref() != rule.kind {
                continue;
            }
            let condition = match (&rule.field, rule.named_child) {
                (Some(field), None) => node.field(field),
                (None, Some(index)) => node.children().filter(|n| n.is_named()).nth(index),
                _ => return Err("condition requires exactly one selector".into()),
            }
            .ok_or_else(|| format!("missing condition on {}", node.kind()))?;
            expand(&condition, rules, &mut out)?;
        }
    }
    Ok(out.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lesscode_core::adapters::driver::tree::{grammar, parse};
    fn count(source: &str) -> usize {
        let p: Profile = serde_json::from_str(include_str!("../profile.json")).unwrap();
        let t = parse(source, grammar(&p.language).unwrap()).unwrap();
        measure(&t.root(), &p).unwrap().len()
    }
    #[test]
    fn folding_keeps_decision_work() {
        assert_eq!(count("if a:\n if b:\n  pass\n"), 2);
        assert_eq!(count("if a and b:\n pass\n"), 3);
    }
    #[test]
    fn assigned_and_returned_conditions_remain_visible() {
        assert_eq!(count("x = a and b\ndef f():\n return c < d < e\n"), 5);
    }
    #[test]
    fn selectors_do_not_double_charge_a_comparison() {
        assert_eq!(count("if a < b and c > d:\n pass\n"), 3);
    }
    #[test]
    fn formatting_and_local_names_do_not_change_totals() {
        assert_eq!(
            count("if a and b:\n pass\n"),
            count("# note\nif (left and right):\n    pass\n")
        );
        assert_eq!(count("flag = not ready\ndef f():\n return not flag\n"), 2);
    }
}
