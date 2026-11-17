//! Conservative assignment classification; unresolved effects remain explicit.
use super::{has, identity, Profile};
use lesscode_core::adapters::driver::tree::{ancestors, walk, SgNode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub assignments: Vec<String>,
    pub identifier: String,
    pub target_field: String,
    pub global_declaration: String,
    pub nonlocal_declaration: String,
    pub lexical_functions: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct Binding {
    pub start: usize,
    pub owner: String,
    pub class: String,
    pub target: Option<String>,
    pub reason: String,
}

pub fn classify(root: &SgNode<'_>, p: &Profile) -> Vec<Binding> {
    let owners = identity::index(root, p);
    let r = &p.bindings;
    let mut declarations = BTreeMap::<(String, String), String>::new();
    for node in walk(root) {
        if node.kind().as_ref() == r.global_declaration
            || node.kind().as_ref() == r.nonlocal_declaration
        {
            let owner = identity::nearest(&node, &owners);
            for name in node
                .children()
                .filter(|n| n.kind().as_ref() == r.identifier)
            {
                declarations.insert(
                    (owner.clone(), name.text().into_owned()),
                    node.kind().into_owned(),
                );
            }
        }
    }
    walk(root)
        .iter()
        .filter(|n| has(&p.effects, n))
        .map(|node| {
            let owner = identity::nearest(node, &owners);
            let mut fact = Binding {
                start: node.range().start,
                owner: owner.clone(),
                class: "U".into(),
                target: None,
                reason: "effect or alias target unresolved".into(),
            };
            if !has(&r.assignments, node) {
                return fact;
            }
            let Some(target) = node.field(&r.target_field) else {
                return fact;
            };
            if target.kind().as_ref() != r.identifier {
                return fact;
            }
            let name = target.text().into_owned();
            let declaration = declarations.get(&(owner.clone(), name.clone()));
            if declaration.is_some_and(|d| *d == r.nonlocal_declaration) {
                fact.reason = "nonlocal binding resolution not implemented".into();
            } else if declaration.is_some_and(|d| *d == r.global_declaration) || owner == "<module>"
            {
                fact.class = "W".into();
                fact.target = Some(format!("<module>::{name}"));
                fact.reason = "explicit module binding write".into();
            } else {
                let enclosing = ancestors(node).into_iter().find(|n| has(&p.owners, n));
                if enclosing.is_some_and(|n| has(&r.lexical_functions, &n)) {
                    fact.class = "L".into();
                    fact.reason =
                        "function-local name rebinding; RHS effects counted separately".into();
                }
            }
            fact
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lesscode_core::adapters::driver::tree::{grammar, parse};
    fn classes(source: &str) -> Vec<String> {
        let p: Profile = serde_json::from_str(include_str!("../profile.json")).unwrap();
        let tree = parse(source, grammar(&p.language).unwrap()).unwrap();
        classify(&tree.root(), &p)
            .iter()
            .map(|f| f.class.clone())
            .collect()
    }
    #[test]
    fn local_assignment_does_not_hide_rhs_effects() {
        assert_eq!(classes("def f():\n x = mutate()\n"), ["L", "U"]);
        // Augmented assignment can mutate an aliased value through __iadd__.
        assert_eq!(classes("def f(x):\n x += [1]\n"), ["U"]);
    }
    #[test]
    fn explicit_module_write_differs_from_local() {
        assert_eq!(
            classes("x = 1\ndef f():\n global x\n x = 2\n y = 3\n"),
            ["W", "U", "W", "L"]
        );
    }
    #[test]
    fn alias_and_nonlocal_effects_cannot_be_zero() {
        assert_eq!(
            classes("def f():\n nonlocal x\n x = 1\n a = obj\n a.value = 2\n"),
            ["U", "U", "L", "U"]
        );
    }
}
