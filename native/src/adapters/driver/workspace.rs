//! Working copies: the only place the driver writes source files.
//!
//! A mutant is planted in a copy of the source tree, never in the tree
//! itself. Before a file is changed its original bytes are saved and a
//! journal entry names the file and its hash; afterwards the original is
//! written back, the hash is verified and the entry removed. A journal entry
//! found when a copy is opened means a run was interrupted; the file is
//! restored then.

use crate::specimen_runner::sha256_hex_bytes;
use std::fs;
use std::path::{Component, Path, PathBuf};

const METADATA: [&str; 3] = [".git", ".hg", ".svn"];

/// The path with its existing part resolved and the rest appended, so that
/// two spellings of one place compare equal before anything is created.
fn resolved(path: &Path) -> Result<PathBuf, String> {
    let absolute = std::path::absolute(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut existing = absolute.clone();
    let mut rest = Vec::new();
    while !existing.exists() {
        match (existing.file_name(), existing.parent()) {
            (Some(name), Some(parent)) => {
                rest.push(name.to_owned());
                existing = parent.to_path_buf();
            }
            _ => return Ok(absolute),
        }
    }
    let mut full =
        fs::canonicalize(&existing).map_err(|e| format!("{}: {e}", existing.display()))?;
    full.extend(rest.into_iter().rev());
    Ok(full)
}

/// Refuse a work directory that is the source root, lies inside it or
/// contains it. Nothing is created by this check.
pub fn check_separate(source_root: &Path, work: &Path) -> Result<(), String> {
    if !source_root.is_dir() {
        return Err(format!(
            "source root {} is not a directory",
            source_root.display()
        ));
    }
    let (source, work_dir) = (resolved(source_root)?, resolved(work)?);
    if work_dir.starts_with(&source) || source.starts_with(&work_dir) {
        return Err(format!(
            "the work directory {} must be outside the source root {} and must not contain it",
            work.display(),
            source_root.display()
        ));
    }
    Ok(())
}

#[cfg_attr(not(unix), allow(clippy::permissions_set_readonly_false))]
fn make_writable(path: &Path) -> std::io::Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    if !permissions.readonly() {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(permissions.mode() | 0o200);
    }
    #[cfg(not(unix))]
    permissions.set_readonly(false);
    fs::set_permissions(path, permissions)
}

/// Copy a tree without version-control metadata and without the excluded
/// names or relative paths; links are not followed. Returns the number of
/// files copied.
pub fn copy_tree(from: &Path, to: &Path, exclusions: &[String]) -> Result<usize, String> {
    let mut copied = 0;
    let mut stack = vec![PathBuf::new()];
    while let Some(relative) = stack.pop() {
        let (source, target) = (from.join(&relative), to.join(&relative));
        fs::create_dir_all(&target).map_err(|e| format!("{}: {e}", target.display()))?;
        let entries = fs::read_dir(&source).map_err(|e| format!("{}: {e}", source.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {e}", source.display()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let below = relative.join(&name);
            let slashed = below.to_string_lossy().replace('\\', "/");
            let excluded = |e: &String| *e == name || *e == slashed;
            if METADATA.contains(&name.as_str()) || exclusions.iter().any(excluded) {
                continue;
            }
            let kind = entry.file_type().map_err(|e| format!("{slashed}: {e}"))?;
            if kind.is_dir() {
                stack.push(below);
            } else if kind.is_file() {
                let target = to.join(&below);
                fs::copy(entry.path(), &target).map_err(|e| format!("{slashed}: {e}"))?;
                make_writable(&target).map_err(|e| format!("{slashed}: {e}"))?;
                copied += 1;
            }
        }
    }
    Ok(copied)
}

/// One working copy with its journal.
#[derive(Debug)]
pub struct Workspace {
    /// The copied tree; commands run here.
    pub tree: PathBuf,
    journal: PathBuf,
    saved: PathBuf,
}

impl Workspace {
    fn at(home: &Path) -> Workspace {
        Workspace {
            tree: home.join("tree"),
            journal: home.join("journal.json"),
            saved: home.join("original.saved"),
        }
    }

    /// A fresh copy of `source_root` under `work/name`.
    pub fn create(
        source_root: &Path,
        work: &Path,
        name: &str,
        exclusions: &[String],
    ) -> Result<Workspace, String> {
        check_separate(source_root, work)?;
        let home = work.join(name);
        if home.exists() {
            return Err(format!(
                "{} already exists; open it or choose another name",
                home.display()
            ));
        }
        let space = Workspace::at(&home);
        copy_tree(source_root, &space.tree, exclusions)?;
        Ok(space)
    }

    /// An existing copy. A file left mutated by an interrupted run is
    /// restored first; its path is returned so that the caller can report it.
    pub fn open(work: &Path, name: &str) -> Result<(Workspace, Option<String>), String> {
        let space = Workspace::at(&work.join(name));
        if !space.tree.is_dir() {
            return Err(format!("no working copy at {}", space.tree.display()));
        }
        let recovered = if space.journal.exists() {
            Some(space.restore()?)
        } else {
            let _ = fs::remove_file(&space.saved);
            None
        };
        Ok((space, recovered))
    }

    fn target(&self, file: &str) -> Result<PathBuf, String> {
        let relative = Path::new(file);
        let inside = relative
            .components()
            .all(|c| matches!(c, Component::Normal(_)));
        if !inside || file.is_empty() {
            return Err(format!("{file}: not a path inside the working copy"));
        }
        Ok(self.tree.join(relative))
    }

    /// Replace `file` in the copy by `mutated`, journalled.
    pub fn plant(&self, file: &str, mutated: &str) -> Result<(), String> {
        if self.journal.exists() {
            return Err("a mutant is already planted; restore it first".into());
        }
        let target = self.target(file)?;
        let original = fs::read(&target).map_err(|e| format!("{}: {e}", target.display()))?;
        fs::write(&self.saved, &original).map_err(|e| format!("{}: {e}", self.saved.display()))?;
        let entry = serde_json::json!({"file": file, "sha256": sha256_hex_bytes(&original)});
        let pending = self.journal.with_extension("pending");
        fs::write(&pending, entry.to_string())
            .map_err(|e| format!("{}: {e}", pending.display()))?;
        fs::rename(&pending, &self.journal)
            .map_err(|e| format!("{}: {e}", self.journal.display()))?;
        fs::write(&target, mutated).map_err(|e| format!("{}: {e}", target.display()))
    }

    /// Write the original back, verify it and clear the journal. Returns
    /// the file restored. An error means the copy must not be used further.
    pub fn restore(&self) -> Result<String, String> {
        let unverified = |why: String| format!("restoration could not be verified: {why}");
        let text =
            fs::read_to_string(&self.journal).map_err(|e| unverified(format!("journal: {e}")))?;
        let entry: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| unverified(format!("journal: {e}")))?;
        let (Some(file), Some(expected)) = (entry["file"].as_str(), entry["sha256"].as_str())
        else {
            return Err(unverified("the journal entry is incomplete".into()));
        };
        let target = self.target(file)?;
        let original = fs::read(&self.saved)
            .map_err(|e| unverified(format!("{file}: saved original: {e}")))?;
        if sha256_hex_bytes(&original) != expected {
            return Err(unverified(format!(
                "{file}: the saved original does not have the journalled hash"
            )));
        }
        fs::write(&target, &original).map_err(|e| unverified(format!("{file}: {e}")))?;
        let written = fs::read(&target).map_err(|e| unverified(format!("{file}: {e}")))?;
        if sha256_hex_bytes(&written) != expected {
            return Err(unverified(format!(
                "{file}: the restored file does not have the journalled hash"
            )));
        }
        fs::remove_file(&self.journal).map_err(|e| unverified(format!("journal: {e}")))?;
        let _ = fs::remove_file(&self.saved);
        Ok(file.to_owned())
    }

    pub fn has_planted(&self) -> bool {
        self.journal.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lesscode-workspace-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("source/src")).unwrap();
        fs::create_dir_all(dir.join("source/.git")).unwrap();
        fs::create_dir_all(dir.join("source/build")).unwrap();
        fs::write(dir.join("source/src/a.txt"), "one\r\ntwo\n").unwrap();
        fs::write(dir.join("source/.git/config"), "x").unwrap();
        fs::write(dir.join("source/build/out.bin"), "y").unwrap();
        dir
    }

    fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut found = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    found.push((path.clone(), fs::read(&path).unwrap()));
                }
            }
        }
        found.sort();
        found
    }

    #[test]
    fn a_work_directory_touching_the_source_root_is_refused() {
        let dir = scratch("refuse");
        let source = dir.join("source");
        for work in [
            source.clone(),
            source.join("work"),
            source.join("src/../work"),
            dir.clone(),
        ] {
            let error = check_separate(&source, &work).unwrap_err();
            assert!(
                error.contains("must be outside"),
                "{}: {error}",
                work.display()
            );
            assert!(Workspace::create(&source, &work, "w1", &[]).is_err());
        }
        assert!(
            !source.join("work").exists(),
            "the check must not create anything"
        );
        check_separate(&source, &dir.join("work")).unwrap();
        assert!(check_separate(&dir.join("missing"), &dir.join("work")).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_copy_leaves_out_metadata_and_exclusions_and_is_writable() {
        let dir = scratch("copy");
        let source = dir.join("source");
        let read_only = source.join("src/a.txt");
        let mut permissions = fs::metadata(&read_only).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&read_only, permissions).unwrap();
        let before = snapshot(&source);
        let space = Workspace::create(&source, &dir.join("work"), "w1", &["build".into()]).unwrap();
        assert!(space.tree.join("src/a.txt").is_file());
        assert!(!space.tree.join(".git").exists() && !space.tree.join("build").exists());
        space.plant("src/a.txt", "changed").unwrap();
        assert_eq!(space.restore().unwrap(), "src/a.txt");
        assert_eq!(snapshot(&source), before, "the source tree changed");
        assert!(
            Workspace::create(&source, &dir.join("work"), "w1", &[]).is_err(),
            "no silent reuse"
        );
        make_writable(&read_only).unwrap();
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn planting_and_restoring_is_byte_exact() {
        let dir = scratch("plant");
        let space = Workspace::create(&dir.join("source"), &dir.join("work"), "w1", &[]).unwrap();
        let target = space.tree.join("src/a.txt");
        space.plant("src/a.txt", "mutated\n").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "mutated\n");
        assert!(space.has_planted());
        assert!(space
            .plant("src/a.txt", "again")
            .unwrap_err()
            .contains("already planted"));
        space.restore().unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"one\r\ntwo\n");
        assert!(!space.has_planted());
        for bad in ["../x", "", "src/../../x"] {
            assert!(
                space
                    .plant(bad, "x")
                    .unwrap_err()
                    .contains("not a path inside"),
                "{bad}"
            );
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_interrupted_run_is_recovered_when_the_copy_is_opened() {
        let dir = scratch("recover");
        let work = dir.join("work");
        let space = Workspace::create(&dir.join("source"), &work, "w1", &[]).unwrap();
        space.plant("src/a.txt", "left behind").unwrap();
        drop(space);
        let (space, recovered) = Workspace::open(&work, "w1").unwrap();
        assert_eq!(recovered.as_deref(), Some("src/a.txt"));
        assert_eq!(
            fs::read(space.tree.join("src/a.txt")).unwrap(),
            b"one\r\ntwo\n"
        );
        let (_, recovered) = Workspace::open(&work, "w1").unwrap();
        assert_eq!(recovered, None);
        assert!(Workspace::open(&work, "w2").is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_restoration_that_cannot_be_verified_is_an_error() {
        let dir = scratch("verify");
        let work = dir.join("work");
        let space = Workspace::create(&dir.join("source"), &work, "w1", &[]).unwrap();
        space.plant("src/a.txt", "mutated").unwrap();
        fs::write(work.join("w1/original.saved"), "tampered").unwrap();
        let error = space.restore().unwrap_err();
        assert!(error.contains("could not be verified"), "{error}");
        assert!(space.has_planted(), "the journal stays for inspection");
        assert!(Workspace::open(&work, "w1")
            .unwrap_err()
            .contains("could not be verified"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
