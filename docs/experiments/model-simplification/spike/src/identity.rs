use super::{has, Profile};
use lesscode_core::adapters::driver::tree::{ancestors, walk, SgNode};
use std::collections::BTreeMap;

pub fn nearest(node: &SgNode<'_>, owners: &BTreeMap<usize, String>) -> String {
    ancestors(node)
        .iter()
        .find_map(|p| owners.get(&p.range().start).cloned())
        .unwrap_or_else(|| "<module>".into())
}

pub fn index(root: &SgNode<'_>, profile: &Profile) -> BTreeMap<usize, String> {
    let mut owners = BTreeMap::new();
    let mut ordinals = BTreeMap::<(String, String), usize>::new();
    for node in walk(root) {
        if !has(&profile.owners, &node) {
            continue;
        }
        let parent = nearest(&node, &owners);
        let name = node
            .field("name")
            .map(|n| n.text().into_owned())
            .unwrap_or_else(|| node.kind().into_owned());
        let ordinal = ordinals.entry((parent.clone(), name.clone())).or_default();
        *ordinal += 1;
        owners.insert(node.range().start, format!("{parent}::{name}#{ordinal}"));
    }
    owners
}
