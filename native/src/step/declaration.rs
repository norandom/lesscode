//! The declaration of a step: what it starts from, what it may touch, how
//! the project is tested and where the results go. Paths are relative to
//! the repository root, with forward slashes; commands run at that root.

use super::revisions::{is_ancestor, resolve};
use crate::adapters::driver::config::RunConfig;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Component, Path};

/// A function the step leaves unprotected on purpose.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutOfReach {
    pub function: String,
    pub reason: String,
}

/// A command whose passing is evidence for one acceptance criterion.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Criterion {
    pub id: String,
    pub command: Vec<String>,
}

/// Where the step tooling writes inside the repository.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outputs {
    pub verdict: String,
    pub evidence: String,
    pub ledger: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    pub step: String,
    /// The identifier of the step's spec.
    pub spec: String,
    pub branch: String,
    /// The revision the step starts from.
    pub base: String,
    pub language: String,
    /// The files the step may change.
    pub scope: Vec<String>,
    #[serde(default)]
    pub test_paths: Vec<String>,
    #[serde(default)]
    pub setup: Vec<Vec<String>>,
    #[serde(default)]
    pub build: Option<Vec<String>>,
    pub test: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Further fields of the driver's run configuration (limits, patterns).
    #[serde(default)]
    pub run: Map<String, Value>,
    /// Tokens the scope may grow by.
    #[serde(default)]
    pub max_token_growth: usize,
    #[serde(default)]
    pub frame_spec: Option<String>,
    #[serde(default)]
    pub out_of_reach: Vec<OutOfReach>,
    #[serde(default)]
    pub criteria: Vec<Criterion>,
    pub outputs: Outputs,
}

/// The two commits of a step, as full hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revisions {
    pub base: String,
    pub step: String,
}

fn inside(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

impl Declaration {
    pub fn read(path: &Path) -> Result<Declaration, String> {
        let bytes =
            std::fs::read(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        Declaration::parse(&bytes).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn parse(bytes: &[u8]) -> Result<Declaration, String> {
        let declaration: Declaration =
            serde_json::from_slice(bytes).map_err(|e| format!("not a step declaration: {e}"))?;
        declaration.check()?;
        Ok(declaration)
    }

    fn is_test_path(&self, file: &str) -> bool {
        self.test_paths.iter().any(|p| file.starts_with(p.as_str()))
    }

    fn check(&self) -> Result<(), String> {
        let named = [
            ("step", &self.step),
            ("spec", &self.spec),
            ("branch", &self.branch),
            ("base", &self.base),
        ];
        if let Some((name, _)) = named.iter().find(|(_, value)| value.trim().is_empty()) {
            return Err(format!("{name} is empty"));
        }
        if self.scope.is_empty() {
            return Err("scope is empty".into());
        }
        let outputs = [
            &self.outputs.verdict,
            &self.outputs.evidence,
            &self.outputs.ledger,
        ];
        let paths = self.scope.iter().chain(outputs).chain(&self.frame_spec);
        if let Some(bad) = paths.clone().find(|p| !inside(p)) {
            return Err(format!("{bad}: not a path inside the repository"));
        }
        if let Some(test) = self.scope.iter().find(|f| self.is_test_path(f)) {
            return Err(format!("scope: {test} lies under a test path"));
        }
        if let Some(o) = self
            .out_of_reach
            .iter()
            .find(|o| o.reason.trim().is_empty())
        {
            return Err(format!("out_of_reach: {} has no reason", o.function));
        }
        let empty = |c: &&Criterion| c.id.trim().is_empty() || c.command.is_empty();
        if let Some(c) = self.criteria.iter().find(empty) {
            return Err(format!("criteria: {:?} needs an id and a command", c.id));
        }
        // The run fields are checked as the driver checks them.
        self.run_config(Path::new("."), &BTreeMap::new())
            .map(|_| ())
    }

    /// The driver's run configuration for a copy at `source_root`, with
    /// faults limited to `functions` where any are named.
    pub fn run_config(
        &self,
        source_root: &Path,
        functions: &BTreeMap<String, Vec<String>>,
    ) -> Result<RunConfig, String> {
        let mut fields = self.run.clone();
        let own = json!({
            "language": self.language, "source_root": source_root, "files": self.scope,
            "test_paths": self.test_paths, "setup": self.setup, "build": self.build,
            "test": self.test, "env": self.env, "functions": functions,
        });
        for (key, value) in own.as_object().expect("an object") {
            if fields.insert(key.clone(), value.clone()).is_some() {
                return Err(format!("run: {key} is set by the declaration itself"));
            }
        }
        let config: RunConfig =
            serde_json::from_value(Value::Object(fields)).map_err(|e| format!("run: {e}"))?;
        config.check().map(|()| config)
    }

    /// The commits of the base and of the step's branch; the base must be
    /// an ancestor of the step.
    pub fn revisions(&self, repo: &Path) -> Result<Revisions, String> {
        let base = resolve(repo, &self.base)?;
        let step = resolve(repo, &self.branch)?;
        if !is_ancestor(repo, &base, &step)? {
            return Err(format!(
                "base {} is not an ancestor of {}",
                self.base, self.branch
            ));
        }
        Ok(Revisions { base, step })
    }
}
