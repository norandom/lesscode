//! `step guard`: the boundary. A step branch is accepted only if a verdict
//! says "accepted" for exactly the committed scope, tests and declaration
//! it has now, or if an override with a written reason is on the ledger
//! for exactly that content. It runs nothing and asks nobody.

use super::context::{hashes, Hashes};
use super::declaration::Declaration;
use super::ledger::{parse, OVERRIDDEN};
use super::revisions::{blob, resolve};
use super::trees::{tree, Tree};
use super::verdict::ACCEPTED;
use serde_json::Value;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Arguments of `step guard`.
#[derive(clap::Args, Clone, Debug)]
pub struct GuardArgs {
    /// The repository.
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,
    /// The branch to judge.
    #[arg(long, conflicts_with = "pushed")]
    pub branch: Option<String>,
    /// The commit of that branch to judge; its tip when absent.
    #[arg(long, requires = "branch")]
    pub revision: Option<String>,
    /// Judge the branches git names on standard input, as a pre-push hook receives them.
    #[arg(long)]
    pub pushed: bool,
}

/// What the guard found for a branch.
#[derive(Debug, PartialEq, Eq)]
pub enum Standing {
    NotAStep,
    Accepted,
    Overridden(String),
    Refused(String),
}

/// A committed file of the revision, else the file on disk, else nothing.
fn committed_or_on_disk(repo: &Path, tree: &Tree, path: &str) -> Result<Option<Vec<u8>>, String> {
    match tree.get(path) {
        Some(hash) => blob(repo, hash).map(Some),
        None => Ok(std::fs::read(repo.join(path)).ok()),
    }
}

/// The declaration in the tree that names `branch`, with its path.
fn declaration_of(
    repo: &Path,
    tree: &Tree,
    branch: &str,
) -> Result<Option<(String, Declaration)>, String> {
    let declared =
        |path: &&String| path.starts_with(".kiro/specs/") && path.ends_with("/step.json");
    for path in tree.keys().filter(declared) {
        let bytes = blob(repo, &tree[path])?;
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("{path} is not JSON: {e}"))?;
        if value["branch"] == branch {
            let declaration = Declaration::parse(&bytes).map_err(|e| format!("{path}: {e}"))?;
            return Ok(Some((path.clone(), declaration)));
        }
    }
    Ok(None)
}

/// Why a verdict does not stand for the content as it is, if so.
fn stale(verdict: &Value, now: &Hashes, base: &str) -> Option<String> {
    if verdict["result"] != ACCEPTED {
        let result = verdict["result"].as_str().unwrap_or("unreadable");
        return Some(format!("the verdict is {result}"));
    }
    let Ok(then) = serde_json::from_value::<Hashes>(verdict["hashes"].clone()) else {
        return Some("the verdict has no hashes".into());
    };
    if verdict["base"] != base {
        return Some("the base is another commit than the one the verdict was written for".into());
    }
    let changed = now
        .scope
        .iter()
        .find(|(file, hash)| then.scope.get(*file) != Some(hash));
    let what = match changed {
        Some((file, _)) => file.as_str(),
        None if then.tests != now.tests => "a test file",
        None if then.declaration != now.declaration => "the declaration",
        None => return None,
    };
    Some(format!("{what} changed after the verdict was written"))
}

/// Judge one branch at one commit.
pub fn standing(
    repo: &Path,
    branch: &str,
    revision: &str,
) -> Result<(Standing, Option<String>), String> {
    let revision = resolve(repo, revision)?;
    let tree = tree(repo, &revision)?;
    let Some((path, declaration)) = declaration_of(repo, &tree, branch)? else {
        return Ok((Standing::NotAStep, None));
    };
    let spec = Some(declaration.spec.clone());
    let now = hashes(&declaration, &path, &tree);
    let base = resolve(repo, &declaration.base)?;
    let verdict = committed_or_on_disk(repo, &tree, &declaration.outputs.verdict)?;
    let refusal = match verdict {
        None => "there is no verdict; run step check".to_owned(),
        Some(bytes) => {
            let verdict: Value = serde_json::from_slice(&bytes)
                .map_err(|e| format!("{} is not JSON: {e}", declaration.outputs.verdict))?;
            match stale(&verdict, &now, &base) {
                None => return Ok((Standing::Accepted, spec)),
                Some(why) => why,
            }
        }
    };
    let ledger =
        committed_or_on_disk(repo, &tree, &declaration.outputs.ledger)?.unwrap_or_default();
    let entries = parse(&String::from_utf8_lossy(&ledger))?;
    let same = serde_json::to_value(&now).map_err(|e| e.to_string())?;
    let overridden = entries
        .iter()
        .rfind(|e| e["result"] == OVERRIDDEN && e["hashes"] == same);
    Ok(match overridden {
        Some(entry) => {
            let reason = entry["reason"].as_str().unwrap_or("").to_owned();
            (Standing::Overridden(reason), spec)
        }
        None => (Standing::Refused(refusal), spec),
    })
}

/// The branches and commits a pre-push hook is given on standard input:
/// `<local ref> <local commit> <remote ref> <remote commit>` per line.
/// Deletions and refs that are not branches are left out.
pub fn pushed(input: &str) -> Vec<(String, String)> {
    let branch = |line: &str| {
        let mut parts = line.split_whitespace();
        let name = parts.next()?.strip_prefix("refs/heads/")?;
        let commit = parts.next()?;
        let deleted = commit.chars().all(|c| c == '0');
        (!deleted).then(|| (name.to_owned(), commit.to_owned()))
    };
    input.lines().filter_map(branch).collect()
}

/// Exit 0 when every branch is accepted, overridden or not a step branch;
/// 1 when one is refused.
pub fn run_guard(args: GuardArgs) -> Result<u8, String> {
    let targets = match (&args.branch, args.pushed) {
        (Some(branch), _) => {
            let revision = args.revision.clone().unwrap_or_else(|| branch.clone());
            vec![(branch.clone(), revision)]
        }
        (None, true) => {
            let mut input = String::new();
            let read = std::io::stdin().read_to_string(&mut input);
            read.map_err(|e| format!("standard input: {e}"))?;
            pushed(&input)
        }
        (None, false) => return Err("give --branch or --pushed".into()),
    };
    let (refused, _) = judge(&args.repo, &targets)?;
    Ok(u8::from(refused))
}

/// Judge branches at commits and say what was found: whether one is
/// refused, and the specs of the steps among them.
pub fn judge(repo: &Path, targets: &[(String, String)]) -> Result<(bool, Vec<String>), String> {
    let (mut refused, mut specs) = (false, Vec::new());
    for (branch, revision) in targets {
        let (found, spec) = standing(repo, branch, revision)?;
        specs.extend(spec);
        match found {
            Standing::NotAStep => println!("{branch}: not a step branch"),
            Standing::Accepted => println!("{branch}: accepted"),
            Standing::Overridden(reason) => println!("{branch}: overridden ({reason})"),
            Standing::Refused(why) => {
                println!("{branch}: refused: {why}");
                refused = true;
            }
        }
    }
    Ok((refused, specs))
}
