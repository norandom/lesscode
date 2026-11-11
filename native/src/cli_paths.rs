//! Reading inputs and choosing fresh outputs for the `prepare` and
//! `compare` commands: paths are checked before anything is resolved
//! through a link, and an output never lies in a source root.

use serde::de::DeserializeOwned;
use std::{
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

#[rustfmt::skip]
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?)
        .map_err(|e| format!("invalid JSON in {}: {e}", path.display()))
}
fn linked(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}
// Walk before canonicalization so links/junctions cannot hide behind their targets.
// Missing suffixes are resolved against the canonical existing ancestor.
#[rustfmt::skip]
fn checked_path(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(|e| e.to_string())?.join(path)
    };
    let mut resolved = PathBuf::new();
    let mut missing = false;
    for part in absolute.components() {
        match part {
            Component::ParentDir => {
                if missing || !resolved.pop() {
                    return Err(format!("path traversal is not allowed: {}", path.display()));
                }
            }
            Component::Normal(name)
                if name == ".."
                    || name == "."
                    || name
                        .to_str()
                        .is_some_and(|s| s.contains('/') || s.contains(char::from(92))) =>
            {
                return Err(format!("nonportable path component: {}", path.display()))
            }
            Component::CurDir => continue,
            Component::Prefix(_) => {
                resolved.push(part.as_os_str());
                continue;
            }
            _ => resolved.push(part.as_os_str()),
        }
        match fs::symlink_metadata(&resolved) {
            Ok(metadata) => {
                if linked(&metadata) {
                    return Err(format!("symlink/reparse point is not allowed: {}", resolved.display()));
                }
                resolved = fs::canonicalize(&resolved).map_err(|e| format!("{}: {e}", resolved.display()))?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => missing = true,
            Err(e) => return Err(format!("{}: {e}", resolved.display())),
        }
    }
    Ok(resolved)
}
pub fn fresh_output(out: &Path, roots: &[&Path]) -> Result<PathBuf, String> {
    let out = checked_path(out)?;
    for root in roots {
        let root = checked_path(root)?;
        if !root.is_dir() {
            return Err(format!(
                "source root is not a directory: {}",
                root.display()
            ));
        }
        if out.starts_with(&root) || root.starts_with(&out) {
            return Err(format!(
                "output must be outside source roots: {}",
                out.display()
            ));
        }
    }
    match fs::symlink_metadata(&out) {
        Ok(_) => Err(format!(
            "output already exists; choose a fresh path: {}",
            out.display()
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(out),
        Err(e) => Err(format!("{}: {e}", out.display())),
    }
}
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("cannot create {}: {e}", path.display()))?
        .write_all(bytes)
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}
pub fn create_parent(out: &Path) -> Result<(), String> {
    fs::create_dir_all(out.parent().ok_or("output needs a parent directory")?)
        .map_err(|e| format!("cannot create output parent: {e}"))
}
