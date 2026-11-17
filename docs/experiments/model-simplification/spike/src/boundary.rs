//! AST body masks protect fixed declarations without parsing model-supplied paths.
use super::{identity, Profile};
use lesscode_core::adapters::driver::tree::{first_error, grammar, parse, walk};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scope {
    owners: Vec<String>,
    preserve_exterior: bool,
}

fn skeleton(
    source: &str,
    p: &Profile,
    scope: &Scope,
    original: &BTreeMap<String, String>,
) -> Result<(String, BTreeMap<String, String>), String> {
    let tree = parse(source, grammar(&p.language)?)?;
    if first_error(&tree.root()).is_some() {
        return Err("candidate syntax error".into());
    }
    let owners = identity::index(&tree.root(), p);
    let mut signatures = BTreeMap::new();
    let mut ranges = Vec::new();
    for node in walk(&tree.root())
        .iter()
        .filter(|n| n.kind().as_ref() == p.relations.function)
    {
        let owner = owners.get(&node.range().start).ok_or("missing owner")?;
        let body = node.field("body").ok_or("missing body")?;
        let header = source[node.range().start..body.range().start].to_string();
        signatures.insert(owner.clone(), header);
        if scope.owners.contains(owner) {
            ranges.push(body.range());
        } else if !original.is_empty() && !original.contains_key(owner) {
            let name = node
                .field("name")
                .ok_or("missing name")?
                .text()
                .into_owned();
            if !name.starts_with('_')
                || original
                    .keys()
                    .any(|key| key.contains(&format!("::{name}#")))
            {
                return Err("new declaration is not a new private helper".into());
            }
            ranges.push(node.range());
        }
    }
    let mut masked = source.to_string();
    ranges.sort_by_key(|r| r.start);
    // Nested definitions are already covered by their enclosing body.
    let mut outer = Vec::<std::ops::Range<usize>>::new();
    for range in ranges {
        if !outer.last().is_some_and(|r| r.end >= range.end) {
            outer.push(range);
        }
    }
    for range in outer.into_iter().rev() {
        masked.replace_range(range, "");
    }
    Ok((
        masked
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        signatures,
    ))
}

pub fn check(args: &[String]) -> Result<(), String> {
    if args.len() != 4 {
        return Err("boundary PROFILE ORIGINAL CANDIDATE SCOPE".into());
    }
    let read = |path: &String| fs::read_to_string(path).map_err(|e| e.to_string());
    let p: Profile = serde_json::from_str(&read(&args[0])?).map_err(|e| e.to_string())?;
    let scope: Scope = serde_json::from_str(&read(&args[3])?).map_err(|e| e.to_string())?;
    let (before, signatures) = skeleton(&read(&args[1])?, &p, &scope, &BTreeMap::new())?;
    let (after, candidate) = skeleton(&read(&args[2])?, &p, &scope, &signatures)?;
    if signatures.iter().any(|(k, v)| candidate.get(k) != Some(v)) {
        return Err("original signature changed or disappeared".into());
    }
    if scope.preserve_exterior && before != after {
        return Err("change outside allowed bodies/helpers".into());
    }
    println!("boundary accepted");
    Ok(())
}
