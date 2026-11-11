//! Keeping a working copy current with its source.
//!
//! A copy is made once and used again by later runs, with whatever its
//! builds left in it. Before it is used again, every file of the source is
//! compared with what the copy was last given: a changed or new file is
//! copied again, a file that is gone from the source is removed. Without
//! this, a run after a change to a test or to another source file judges
//! the mutants against the files as they were.

use super::workspace::{make_writable, skipped};
use crate::specimen_runner::sha256_hex_bytes;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST: &str = "manifest.json";

/// Relative path, with forward slashes, to the hash of the content.
type Listing = BTreeMap<String, String>;

/// The files a copy of `from` holds, with the hash each has in `from`.
fn listing(from: &Path, exclusions: &[String]) -> Result<Listing, String> {
    let mut found = Listing::new();
    let mut stack = vec![PathBuf::new()];
    while let Some(relative) = stack.pop() {
        let source = from.join(&relative);
        let entries = fs::read_dir(&source).map_err(|e| format!("{}: {e}", source.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {e}", source.display()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let below = relative.join(&name);
            let slashed = below.to_string_lossy().replace('\\', "/");
            if skipped(&name, &slashed, exclusions) {
                continue;
            }
            let kind = entry.file_type().map_err(|e| format!("{slashed}: {e}"))?;
            if kind.is_dir() {
                stack.push(below);
            } else if kind.is_file() {
                let bytes = fs::read(entry.path()).map_err(|e| format!("{slashed}: {e}"))?;
                found.insert(slashed, sha256_hex_bytes(&bytes));
            }
        }
    }
    Ok(found)
}

fn store(home: &Path, listing: &Listing) -> Result<(), String> {
    let path = home.join(MANIFEST);
    let text = serde_json::to_string(listing).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// What the copy at `home` was last given; `None` for a copy made before
/// this record existed, or one whose record cannot be read.
fn known(home: &Path) -> Option<Listing> {
    let text = fs::read_to_string(home.join(MANIFEST)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Record what a fresh copy at `home` was given.
pub fn remember(home: &Path, from: &Path, exclusions: &[String]) -> Result<(), String> {
    store(home, &listing(from, exclusions)?)
}

/// Bring the copy at `home` up to date with `from`. Returns how many files
/// were copied again or removed.
pub fn refresh(home: &Path, from: &Path, exclusions: &[String]) -> Result<usize, String> {
    let now = listing(from, exclusions)?;
    let before = known(home);
    let tree = home.join("tree");
    let mut changed = 0;
    for (path, hash) in &now {
        let target = tree.join(path);
        let current = match &before {
            Some(given) => given.get(path) == Some(hash) && target.is_file(),
            None => fs::read(&target).is_ok_and(|bytes| sha256_hex_bytes(&bytes) == *hash),
        };
        if current {
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{path}: {e}"))?;
        }
        fs::copy(from.join(path), &target).map_err(|e| format!("{path}: {e}"))?;
        make_writable(&target).map_err(|e| format!("{path}: {e}"))?;
        changed += 1;
    }
    let gone = before.iter().flat_map(|given| given.keys());
    for path in gone.filter(|path| !now.contains_key(*path)) {
        if fs::remove_file(tree.join(path)).is_ok() {
            changed += 1;
        }
    }
    store(home, &now)?;
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lesscode-sync-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("source/tests")).unwrap();
        fs::create_dir_all(dir.join("source/.git")).unwrap();
        fs::write(dir.join("source/tests/a.txt"), "one").unwrap();
        fs::write(dir.join("source/tests/gone.txt"), "x").unwrap();
        fs::write(dir.join("source/.git/config"), "x").unwrap();
        dir
    }

    fn copy(dir: &Path) -> PathBuf {
        let home = dir.join("work/w1");
        let none: [String; 0] = [];
        super::super::workspace::copy_tree(&dir.join("source"), &home.join("tree"), &none).unwrap();
        home
    }

    #[test]
    fn a_copy_follows_changed_added_and_removed_files_and_keeps_its_own() {
        let dir = scratch("follow");
        let (source, home) = (dir.join("source"), copy(&dir));
        remember(&home, &source, &[]).unwrap();
        fs::create_dir_all(home.join("tree/build")).unwrap();
        fs::write(home.join("tree/build/out.bin"), "built in the copy").unwrap();
        assert_eq!(refresh(&home, &source, &[]).unwrap(), 0, "nothing changed");

        fs::write(source.join("tests/a.txt"), "two").unwrap();
        fs::create_dir_all(source.join("tests/more")).unwrap();
        fs::write(source.join("tests/more/b.txt"), "new").unwrap();
        fs::remove_file(source.join("tests/gone.txt")).unwrap();
        assert_eq!(refresh(&home, &source, &[]).unwrap(), 3);
        let read = |path: &str| fs::read_to_string(home.join("tree").join(path)).ok();
        assert_eq!(read("tests/a.txt").as_deref(), Some("two"));
        assert_eq!(read("tests/more/b.txt").as_deref(), Some("new"));
        assert_eq!(read("tests/gone.txt"), None);
        assert_eq!(read("build/out.bin").as_deref(), Some("built in the copy"));
        assert!(!home.join("tree/.git").exists());
        assert_eq!(refresh(&home, &source, &[]).unwrap(), 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_copy_without_a_record_is_compared_file_by_file() {
        let dir = scratch("no-record");
        let (source, home) = (dir.join("source"), copy(&dir));
        fs::write(source.join("tests/a.txt"), "two").unwrap();
        assert_eq!(refresh(&home, &source, &[]).unwrap(), 1);
        let copied = fs::read_to_string(home.join("tree/tests/a.txt")).unwrap();
        assert_eq!(copied, "two");
        assert!(home.join(MANIFEST).is_file());
        let _ = fs::remove_dir_all(&dir);
    }
}
