//! The configuration of a mutation run: which files, which commands, which
//! limits. Commands are argument vectors; nothing is passed through a shell.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn default_floor() -> u64 {
    60
}
fn default_ceiling() -> u64 {
    900
}
fn default_factor() -> f64 {
    5.0
}
fn default_workers() -> usize {
    1
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunConfig {
    /// The language pack to use.
    pub language: String,
    /// The tree to copy; relative to the configuration file.
    pub source_root: PathBuf,
    /// Files to mutate, relative to the source root, with forward slashes.
    pub files: Vec<String>,
    /// Per file, the functions to mutate in; a file without an entry is
    /// mutated everywhere. Does not change what a mutant's outcome is.
    #[serde(default)]
    pub functions: BTreeMap<String, Vec<String>>,
    /// Path prefixes that hold tests; files under them are never mutated.
    #[serde(default)]
    pub test_paths: Vec<String>,
    /// Commands run once in each fresh copy, in order.
    #[serde(default)]
    pub setup: Vec<Vec<String>>,
    /// Optional build step before the tests of each mutant.
    #[serde(default)]
    pub build: Option<Vec<String>>,
    /// The test command; exit code 0 means the tests passed. In any command
    /// `{limit_s}` is replaced by the step's time limit in seconds.
    pub test: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Names or relative paths left out of the copies.
    #[serde(default)]
    pub exclusions: Vec<String>,
    /// The time limit of a step is `factor` times the slowest unmutated
    /// step, at least the floor and at most the ceiling, in seconds.
    #[serde(default = "default_floor")]
    pub timeout_floor_s: u64,
    #[serde(default = "default_ceiling")]
    pub timeout_ceiling_s: u64,
    #[serde(default = "default_factor")]
    pub timeout_factor: f64,
    /// Exit codes of the test command that mean the tests could not be loaded.
    #[serde(default)]
    pub invalid_exit_codes: Vec<i32>,
    /// Exit codes that mean a time limit inside the command was reached, for
    /// commands whose work cannot be killed from outside (a container).
    #[serde(default)]
    pub timeout_exit_codes: Vec<i32>,
    /// Output fragments that mean the mutant did not compile or load.
    #[serde(default)]
    pub invalid_output_patterns: Vec<String>,
    /// Output fragments that mean the machine failed, not the mutant.
    #[serde(default)]
    pub error_output_patterns: Vec<String>,
    /// Output fragments that show the test runner itself ran. When given, a
    /// failing exit without one of them is an error, not a kill.
    #[serde(default)]
    pub runner_evidence_patterns: Vec<String>,
    #[serde(default = "default_workers")]
    pub workers: usize,
    /// The project cannot be built or tested in several copies at once.
    #[serde(default)]
    pub serial_only: bool,
}

impl RunConfig {
    /// Read and check a configuration; `source_root` becomes absolute.
    pub fn read(path: &Path) -> Result<RunConfig, String> {
        let bytes =
            std::fs::read(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        let mut config: RunConfig = serde_json::from_slice(&bytes)
            .map_err(|e| format!("{} is not a run configuration: {e}", path.display()))?;
        if config.source_root.is_relative() {
            let base = path.parent().unwrap_or(Path::new("."));
            config.source_root = base.join(&config.source_root);
        }
        config.check()?;
        Ok(config)
    }

    pub fn check(&self) -> Result<(), String> {
        let problem = if self.files.is_empty() {
            "files is empty"
        } else if self.test.is_empty() {
            "test is empty"
        } else if self.setup.iter().any(Vec::is_empty)
            || self.build.as_ref().is_some_and(Vec::is_empty)
        {
            "a command is empty"
        } else if self.workers == 0 {
            "workers is 0"
        } else if self.timeout_floor_s == 0 || self.timeout_ceiling_s < self.timeout_floor_s {
            "the time limits are not ordered"
        } else if self.timeout_factor.is_nan() || self.timeout_factor < 1.0 {
            "timeout_factor is below 1"
        } else {
            return Ok(());
        };
        Err(format!("run configuration: {problem}"))
    }

    /// How many copies run at once.
    pub fn parallel(&self, requested: Option<usize>) -> usize {
        if self.serial_only {
            return 1;
        }
        requested.unwrap_or(self.workers).max(1)
    }

    /// The time limit for a step whose unmutated run took `baseline_s`.
    pub fn limit_s(&self, baseline_s: f64) -> u64 {
        let scaled = (baseline_s * self.timeout_factor).ceil() as u64;
        scaled.clamp(self.timeout_floor_s, self.timeout_ceiling_s)
    }

    /// A stable text of everything that decides an outcome, for the store.
    pub fn identity(&self) -> String {
        format!(
            "{:?}|{:?}|{:?}|{:?}|{}|{}|{}|{:?}|{:?}|{:?}|{:?}|{:?}",
            self.setup,
            self.build,
            self.test,
            self.env,
            self.timeout_floor_s,
            self.timeout_ceiling_s,
            self.timeout_factor,
            self.invalid_exit_codes,
            self.invalid_output_patterns,
            self.error_output_patterns,
            self.runner_evidence_patterns,
            self.timeout_exit_codes
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<RunConfig, String> {
        let config: RunConfig = serde_json::from_str(text).map_err(|e| e.to_string())?;
        config.check().map(|()| config)
    }

    const SMALL: &str =
        r#"{"language": "x", "source_root": "src", "files": ["a.txt"], "test": ["run", "tests"]}"#;

    #[test]
    fn defaults_are_filled_and_limits_scale() {
        let config = parse(SMALL).unwrap();
        assert_eq!(
            (
                config.timeout_floor_s,
                config.timeout_ceiling_s,
                config.workers
            ),
            (60, 900, 1)
        );
        assert_eq!(config.limit_s(2.0), 60);
        assert_eq!(config.limit_s(40.0), 200);
        assert_eq!(config.limit_s(1000.0), 900);
        assert_eq!(config.parallel(None), 1);
        assert_eq!(config.parallel(Some(6)), 6);
    }

    #[test]
    fn a_serial_project_never_runs_in_parallel() {
        let config = parse(&SMALL.replace("}", r#", "workers": 4, "serial_only": true}"#)).unwrap();
        assert_eq!(config.parallel(Some(6)), 1);
    }

    #[test]
    fn wrong_configurations_are_named() {
        for (bad, why) in [
            (SMALL.replace(r#"["a.txt"]"#, "[]"), "files is empty"),
            (SMALL.replace(r#"["run", "tests"]"#, "[]"), "test is empty"),
            (
                SMALL.replace("}", r#", "build": []}"#),
                "a command is empty",
            ),
            (SMALL.replace("}", r#", "workers": 0}"#), "workers is 0"),
            (
                SMALL.replace("}", r#", "timeout_ceiling_s": 5}"#),
                "not ordered",
            ),
            (SMALL.replace("}", r#", "timeout_factor": 0.5}"#), "below 1"),
            (
                SMALL.replace("}", r#", "shell": "sh -c"}"#),
                "unknown field",
            ),
        ] {
            assert!(parse(&bad).unwrap_err().contains(why), "{why}");
        }
    }

    #[test]
    fn the_identity_changes_with_what_decides_an_outcome() {
        let config = parse(SMALL).unwrap();
        let other = parse(&SMALL.replace("}", r#", "env": {"A": "1"}}"#)).unwrap();
        let same = parse(&SMALL.replace("}", r#", "workers": 3}"#)).unwrap();
        assert_ne!(config.identity(), other.identity());
        assert_eq!(config.identity(), same.identity());
    }
}
