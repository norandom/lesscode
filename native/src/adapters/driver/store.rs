//! Reusable outcomes: a mutant that was already run under the same
//! configuration is not run again.

use crate::specimen_runner::sha256_hex_bytes;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stored {
    pub class: String,
    pub reason: Option<String>,
    pub exit_code: Option<i32>,
    pub seconds: f64,
    /// The receipt of the run that produced the outcome.
    pub run: String,
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn open(dir: &Path) -> Result<Store, String> {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        Ok(Store {
            dir: dir.to_path_buf(),
        })
    }

    /// The key of a mutant: the original file, the mutated text and
    /// everything in the configuration that decides an outcome.
    pub fn key(original_sha256: &str, mutated_sha256: &str, config_identity: &str) -> String {
        let joined = format!("{original_sha256}\n{mutated_sha256}\n{config_identity}");
        sha256_hex_bytes(joined.as_bytes())
    }

    fn path(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}.json"))
    }

    pub fn get(&self, key: &str) -> Option<Stored> {
        let bytes = std::fs::read(self.path(key)).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Store an outcome by temporary file and rename. An error outcome is
    /// never stored: it says nothing about the mutant.
    pub fn put(&self, key: &str, outcome: &Stored) -> Result<(), String> {
        if outcome.class == "error" {
            return Ok(());
        }
        let path = self.path(key);
        let pending = path.with_extension(format!("pending-{}", std::process::id()));
        let text = serde_json::to_string(outcome).map_err(|e| e.to_string())?;
        std::fs::write(&pending, text).map_err(|e| format!("{}: {e}", pending.display()))?;
        std::fs::rename(&pending, &path).map_err(|e| format!("{}: {e}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(class: &str) -> Stored {
        Stored {
            class: class.into(),
            reason: None,
            exit_code: Some(1),
            seconds: 0.5,
            run: "r1".into(),
        }
    }

    #[test]
    fn outcomes_are_kept_by_key_and_errors_are_not() {
        let dir = std::env::temp_dir().join(format!("lesscode-store-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = Store::open(&dir).unwrap();
        let (a, b) = (Store::key("o", "m1", "c"), Store::key("o", "m2", "c"));
        assert_ne!(a, b);
        assert_ne!(a, Store::key("o", "m1", "other configuration"));
        assert_eq!(store.get(&a), None);
        store.put(&a, &outcome("killed")).unwrap();
        store.put(&b, &outcome("error")).unwrap();
        assert_eq!(store.get(&a), Some(outcome("killed")));
        assert_eq!(store.get(&b), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
