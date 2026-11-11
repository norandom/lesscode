//! What a step is judged on: its declaration as committed, the two
//! revisions and their trees. Opening it fails, with nothing written, when
//! what is on disk is not what is committed, because the judgement would
//! then be about another state than the one the developer sees.

use super::declaration::{Declaration, Revisions};
use super::revisions::{blob, resolve};
use super::trees::{digest, tree, Tree};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The hashes git gives the committed content a verdict is bound to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct Hashes {
    /// The files in scope, by path.
    pub scope: BTreeMap<String, String>,
    /// One hash over the files under the test paths.
    pub tests: String,
    pub declaration: String,
}

pub struct Context {
    pub repo: PathBuf,
    /// The declaration's path, relative to the repository root.
    pub path: String,
    pub declaration: Declaration,
    pub revisions: Revisions,
    pub base_tree: Tree,
    pub step_tree: Tree,
}

fn unix(bytes: &[u8]) -> Vec<u8> {
    String::from_utf8_lossy(bytes)
        .replace("\r\n", "\n")
        .into_bytes()
}

impl Context {
    /// Read the declaration at `path` and the state of the repository.
    pub fn open(repo: &Path, path: &str) -> Result<Context, String> {
        let path = path.replace('\\', "/");
        let declaration = Declaration::read(&repo.join(&path))?;
        let revisions = declaration.revisions(repo)?;
        let context = Context {
            repo: repo.to_path_buf(),
            base_tree: tree(repo, &revisions.base)?,
            step_tree: tree(repo, &revisions.step)?,
            path,
            declaration,
            revisions,
        };
        if !context.step_tree.contains_key(&context.path) {
            return Err(format!(
                "{} is not committed on {}",
                context.path, context.declaration.branch
            ));
        }
        context.same_as_committed(&context.path)?;
        // With the branch checked out, uncommitted work in what is judged
        // would be left out without a word.
        if resolve(repo, "HEAD")? == context.revisions.step {
            let judged = context.step_tree.keys().filter(|p| context.is_judged(p));
            for file in judged {
                context.same_as_committed(file)?;
            }
        }
        Ok(context)
    }

    /// A scope file or a file under a test path.
    pub fn is_judged(&self, path: &str) -> bool {
        self.declaration.scope.iter().any(|f| f == path) || self.declaration.is_test_path(path)
    }

    /// The file on disk has the committed content, line endings aside.
    fn same_as_committed(&self, path: &str) -> Result<(), String> {
        let committed = blob(&self.repo, &self.step_tree[path])?;
        let on_disk = std::fs::read(self.repo.join(path)).unwrap_or_default();
        if unix(&on_disk) != unix(&committed) {
            return Err(format!(
                "{path} on disk differs from what is committed on {}; commit it first",
                self.declaration.branch
            ));
        }
        Ok(())
    }

    /// The hashes of what is judged, at any revision's tree.
    pub fn hashes_of(&self, tree: &Tree) -> Hashes {
        let scope = &self.declaration.scope;
        let file = |f: &String| (f.clone(), tree.get(f).cloned().unwrap_or_default());
        Hashes {
            scope: scope.iter().map(file).collect(),
            tests: digest(tree, |p| self.declaration.is_test_path(p)),
            declaration: tree.get(&self.path).cloned().unwrap_or_default(),
        }
    }

    /// The text of a scope file at a revision; empty when it is not there.
    pub fn text(&self, tree: &Tree, file: &str) -> Result<String, String> {
        let Some(hash) = tree.get(file) else {
            return Ok(String::new());
        };
        String::from_utf8(blob(&self.repo, hash)?).map_err(|e| format!("{file} is not UTF-8: {e}"))
    }

    /// A path of the repository the step may change besides its scope and
    /// its tests: its spec, and its own outputs.
    pub fn is_own(&self, path: &str) -> bool {
        let spec = format!(".kiro/specs/{}/", self.declaration.spec);
        let outputs = &self.declaration.outputs;
        let named = [&outputs.verdict, &outputs.evidence, &outputs.ledger];
        path.starts_with(&spec) || named.iter().any(|o| *o == path)
    }
}
