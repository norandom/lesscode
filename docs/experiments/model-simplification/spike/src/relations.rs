//! Lexical instance-field families under explicit profile contracts; aliases remain U.
use super::{bindings::Binding, identity, Profile};
use lesscode_core::adapters::driver::tree::{ancestors, walk, SgNode};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub attribute: String,
    pub object_field: String,
    pub member_field: String,
    pub receiver: String,
    pub class: String,
    pub function: String,
    pub parameters_field: String,
    pub call: String,
    pub callee_field: String,
    pub mutators: Vec<Mutation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mutation {
    pub class_name: String,
    pub target: String,
    pub method: String,
    pub contract: String,
}

#[derive(Serialize, Debug)]
pub struct Relation {
    pub target: String,
    pub sites: BTreeSet<usize>,
    pub writers: BTreeSet<String>,
    pub repeated_sites: usize,
    pub dispersion: usize,
}

fn field_target(node: &SgNode<'_>, target: &SgNode<'_>, p: &Profile) -> Option<(String, String)> {
    let r = &p.relations;
    if target.kind().as_ref() != r.attribute {
        return None;
    }
    let object = target.field(&r.object_field)?;
    let member = target.field(&r.member_field)?;
    if object.kind().as_ref() != p.bindings.identifier {
        return None;
    }
    let enclosing = ancestors(node);
    let function = enclosing.iter().find(|n| super::has(&p.owners, n))?;
    if function.kind().as_ref() != r.function {
        return None;
    }
    let parameters = function.field(&r.parameters_field)?;
    let first = parameters.children().find(|n| n.is_named())?;
    if first.kind().as_ref() != p.bindings.identifier || first.text() != object.text() {
        return None;
    }
    // A reassigned/shadowed receiver cannot establish an instance-field family.
    if walk(function).iter().any(|n| {
        n.field(&p.bindings.target_field)
            .is_some_and(|t| t.text() == first.text())
    }) {
        return None;
    }
    let class = enclosing.iter().find(|n| n.kind().as_ref() == r.class)?;
    let owners = identity::index(enclosing.last()?, p);
    let class_name = class.field("name")?.text().into_owned();
    let owner = owners.get(&class.range().start)?;
    Some((
        format!("{owner}::{}.{}", r.receiver, member.text()),
        class_name,
    ))
}

pub fn measure(root: &SgNode<'_>, p: &Profile, bindings: &mut [Binding]) -> Vec<Relation> {
    let mut grouped = BTreeMap::<String, (BTreeSet<usize>, BTreeSet<String>)>::new();
    for node in walk(root) {
        let resolved = resolve(&node, p);
        if let Some((target, reason)) = resolved {
            if let Some(binding) = bindings.iter_mut().find(|b| b.start == node.range().start) {
                binding.class = "W".into();
                binding.target = Some(target);
                binding.reason = reason;
            }
        }
    }
    for binding in bindings.iter().filter(|b| b.class == "W") {
        if let Some(target) = &binding.target {
            let row = grouped.entry(target.clone()).or_default();
            row.0.insert(binding.start);
            row.1.insert(binding.owner.clone());
        }
    }
    grouped
        .into_iter()
        .map(|(target, (sites, writers))| Relation {
            target,
            repeated_sites: sites.len().saturating_sub(1),
            dispersion: writers.len().saturating_sub(1),
            sites,
            writers,
        })
        .collect()
}

fn resolve(node: &SgNode<'_>, p: &Profile) -> Option<(String, String)> {
    if super::has(&p.bindings.assignments, node) {
        let target = node.field(&p.bindings.target_field)?;
        let (name, _) = field_target(node, &target, p)?;
        return Some((
            name,
            "lexical field write; plain-instance contract required".into(),
        ));
    }
    if node.kind().as_ref() != p.relations.call {
        return None;
    }
    let callee = node.field(&p.relations.callee_field)?;
    let target = callee.field(&p.relations.object_field)?;
    let method = callee.field(&p.relations.member_field)?;
    let (name, class) = field_target(node, &target, p)?;
    let contract = p.relations.mutators.iter().find(|m| {
        m.class_name == class
            && name.ends_with(&format!("::{}", m.target))
            && method.text() == m.method
            && !m.contract.is_empty()
    })?;
    Some((
        name,
        format!("conditional on admitted contract: {}", contract.contract),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lesscode_core::adapters::driver::tree::{grammar, parse};
    #[test]
    fn relations_do_not_discharge_alias_uncertainty() {
        let p: Profile = serde_json::from_str(include_str!("../profile.json")).unwrap();
        let source = "class A:\n def f(self):\n  self.x = []\n  alias = self.x\n  alias.append(1)\n def g(self):\n  self.x = []\n";
        let tree = parse(source, grammar(&p.language).unwrap()).unwrap();
        let mut bindings = super::super::bindings::classify(&tree.root(), &p);
        let relations = measure(&tree.root(), &p, &mut bindings);
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].sites.len(), 2);
        assert_eq!(relations[0].dispersion, 1);
        assert_eq!(relations[0].repeated_sites, 1);
        assert_eq!(bindings.iter().filter(|b| b.class == "U").count(), 1);
    }

    #[test]
    fn only_the_declared_mutator_has_a_conditional_write_contract() {
        let p: Profile = serde_json::from_str(include_str!("../profile.json")).unwrap();
        let source = "class XLikesExporter:\n def f(this):\n  this.tweets.extend([])\n  this.other.extend([])\n  setattr(this, 'tweets', [])\n";
        let tree = parse(source, grammar(&p.language).unwrap()).unwrap();
        let mut bindings = super::super::bindings::classify(&tree.root(), &p);
        let relations = measure(&tree.root(), &p, &mut bindings);
        assert_eq!(relations.len(), 1);
        assert_eq!(bindings.iter().filter(|b| b.class == "U").count(), 2);
    }

    #[test]
    fn shadowed_receivers_are_unknown_and_classes_do_not_merge() {
        let p: Profile = serde_json::from_str(include_str!("../profile.json")).unwrap();
        let source = "class A:\n def f(self):\n  self.x = []\nclass B:\n def f(self):\n  self.x = []\n def g(self):\n  self = other\n  self.x = []\n";
        let tree = parse(source, grammar(&p.language).unwrap()).unwrap();
        let mut bindings = super::super::bindings::classify(&tree.root(), &p);
        let relations = measure(&tree.root(), &p, &mut bindings);
        assert_eq!(relations.len(), 2);
        assert!(relations.iter().all(|r| r.dispersion == 0));
        assert_eq!(bindings.iter().filter(|b| b.class == "U").count(), 1);
    }

    #[test]
    fn merging_writers_keeps_sites_and_cannot_meet_the_state_target() {
        let p: Profile = serde_json::from_str(include_str!("../profile.json")).unwrap();
        let sources = [
            "class A:\n def f(self):\n  self.x=1\n def g(self):\n  self.x=2\n",
            "class A:\n def f(self):\n  self.x=1\n  self.x=2\n",
        ];
        for (source, dispersion) in sources.into_iter().zip([1, 0]) {
            let tree = parse(source, grammar(&p.language).unwrap()).unwrap();
            let mut bindings = super::super::bindings::classify(&tree.root(), &p);
            let relations = measure(&tree.root(), &p, &mut bindings);
            assert_eq!(relations[0].sites.len(), 2);
            assert_eq!(relations[0].repeated_sites, 1);
            assert_eq!(relations[0].dispersion, dispersion);
        }
    }
}
