//! Reading a repository at a revision, without changing it: only the git
//! commands listed in `READ_ONLY` are ever started, and a tree is copied out
//! by reading its blobs, so nothing is checked out and no branch is touched.

use crate::adapters::driver::workspace::check_separate;
use std::io::{BufRead, BufReader, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

/// The git commands this module may start. None of them writes to the
/// repository.
pub const READ_ONLY: &[&str] = &["rev-parse", "merge-base", "ls-tree", "cat-file", "log"];

fn command(repo: &Path, args: &[&str]) -> Result<Command, String> {
    if !args.first().is_some_and(|a| READ_ONLY.contains(a)) {
        return Err(format!("git {args:?} is not a read-only command"));
    }
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .stderr(Stdio::piped());
    Ok(command)
}

fn output(repo: &Path, args: &[&str]) -> Result<(bool, Vec<u8>), String> {
    let done = command(repo, args)?
        .output()
        .map_err(|e| format!("Cannot start git: {e}"))?;
    if !done.status.success() && done.status.code() != Some(1) {
        let said = String::from_utf8_lossy(&done.stderr);
        return Err(format!("git {} failed: {}", args.join(" "), said.trim()));
    }
    Ok((done.status.success(), done.stdout))
}

/// The full hash of the commit `revision` names.
pub fn resolve(repo: &Path, revision: &str) -> Result<String, String> {
    let name = format!("{revision}^{{commit}}");
    let (found, out) = output(repo, &["rev-parse", "--verify", "--quiet", &name])?;
    let hash = String::from_utf8_lossy(&out).trim().to_owned();
    if !found || hash.is_empty() {
        return Err(format!("{revision} is not a commit of {}", repo.display()));
    }
    Ok(hash)
}

/// Whether `base` is `step` or one of its ancestors.
pub fn is_ancestor(repo: &Path, base: &str, step: &str) -> Result<bool, String> {
    Ok(output(repo, &["merge-base", "--is-ancestor", base, step])?.0)
}

/// What a copy holds and what it could not hold.
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Copied {
    pub files: usize,
    /// Entries that are not plain files (links, other repositories), with why.
    pub skipped: Vec<(String, String)>,
}

/// One entry of a tree: a file, a link or another repository.
#[derive(Debug, Clone)]
pub struct Entry {
    pub mode: String,
    pub kind: String,
    pub hash: String,
    pub path: String,
}

impl Entry {
    pub fn is_plain_file(&self) -> bool {
        self.kind == "blob" && self.mode != "120000"
    }
}

/// The entries below `root` (the whole tree when empty) at a revision.
pub fn entries(repo: &Path, revision: &str, root: &str) -> Result<Vec<Entry>, String> {
    let mut args = vec!["ls-tree", "-r", "-z", "--full-tree", revision];
    if !root.is_empty() {
        args.extend(["--", root]);
    }
    let (_, out) = output(repo, &args)?;
    let mut found = Vec::new();
    for record in out.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let text = String::from_utf8_lossy(record);
        let parsed = text.split_once('\t').and_then(|(head, path)| {
            let mut parts = head.split(' ');
            Some(Entry {
                mode: parts.next()?.to_owned(),
                kind: parts.next()?.to_owned(),
                hash: parts.next()?.to_owned(),
                path: path.to_owned(),
            })
        });
        found.push(parsed.ok_or_else(|| format!("git ls-tree: cannot read {text:?}"))?);
    }
    Ok(found)
}

/// Where an entry goes below `dest`: its path without the root prefix.
fn target(dest: &Path, root: &str, path: &str) -> Result<PathBuf, String> {
    let prefix = root.trim_matches('/');
    let relative = match prefix.is_empty() {
        true => path,
        false => path
            .strip_prefix(prefix)
            .map(|rest| rest.trim_start_matches('/'))
            .unwrap_or(path),
    };
    let plain = Path::new(relative)
        .components()
        .all(|c| matches!(c, Component::Normal(_)));
    if !plain || relative.is_empty() {
        return Err(format!("{path}: not a path inside the copy"));
    }
    Ok(dest.join(relative))
}

#[cfg(unix)]
fn mark_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn mark_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// The next answer of `git cat-file --batch`, which must be for `hash`.
fn next_blob(reader: &mut impl BufRead, hash: &str) -> Result<Vec<u8>, String> {
    let mut header = String::new();
    reader.read_line(&mut header).map_err(|e| e.to_string())?;
    let size = header
        .trim()
        .rsplit(' ')
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|_| header.starts_with(hash))
        .ok_or_else(|| format!("git cat-file: unexpected answer {header:?}"))?;
    let mut content = vec![0; size + 1];
    reader.read_exact(&mut content).map_err(|e| e.to_string())?;
    content.pop();
    Ok(content)
}

/// Write the plain files among `entries` below `dest`, reading their
/// content from the repository. File contents are the committed bytes; no
/// line-ending conversion is applied. Other entries are listed as skipped.
pub fn write_entries(
    repo: &Path,
    entries: Vec<Entry>,
    root: &str,
    dest: &Path,
) -> Result<Copied, String> {
    check_separate(repo, dest)?;
    let io = |e: std::io::Error| format!("copy into {}: {e}", dest.display());
    let (mut blobs, mut copied) = (Vec::new(), Copied::default());
    for entry in entries {
        match entry.is_plain_file() {
            true => blobs.push(entry),
            false => {
                let why = format!("mode {} {}", entry.mode, entry.kind);
                copied.skipped.push((entry.path, why));
            }
        }
    }
    std::fs::create_dir_all(dest).map_err(io)?;
    let mut child = command(repo, &["cat-file", "--batch"])?
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Cannot start git: {e}"))?;
    let mut stdin = child.stdin.take().expect("piped");
    let wanted: Vec<u8> = blobs
        .iter()
        .flat_map(|b| format!("{}\n", b.hash).into_bytes())
        .collect();
    let feeder = std::thread::spawn(move || stdin.write_all(&wanted));
    let mut reader = BufReader::new(child.stdout.take().expect("piped"));
    for blob in &blobs {
        let content = next_blob(&mut reader, &blob.hash)?;
        let path = target(dest, root, &blob.path)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(io)?;
        }
        std::fs::write(&path, content).map_err(io)?;
        if blob.mode == "100755" {
            mark_executable(&path).map_err(io)?;
        }
        copied.files += 1;
    }
    drop(reader);
    let _ = feeder.join();
    child.wait().map_err(io)?;
    Ok(copied)
}

/// Copy the files below `root` (the whole tree when empty) as they are at
/// `revision` into the new directory `dest`, which must not exist.
pub fn copy_tree(repo: &Path, revision: &str, root: &str, dest: &Path) -> Result<Copied, String> {
    if dest.exists() {
        return Err(format!("{} exists already", dest.display()));
    }
    write_entries(repo, entries(repo, revision, root)?, root, dest)
}

/// The content of one committed file.
pub fn blob(repo: &Path, hash: &str) -> Result<Vec<u8>, String> {
    let (found, out) = output(repo, &["cat-file", "blob", hash])?;
    match found {
        true => Ok(out),
        false => Err(format!("git has no blob {hash}")),
    }
}

/// The commits after `base` up to `step`, newest first.
pub fn commits(repo: &Path, base: &str, step: &str) -> Result<Vec<String>, String> {
    let range = format!("{base}..{step}");
    let (_, out) = output(repo, &["log", "--format=%H", &range])?;
    let text = String::from_utf8_lossy(&out);
    Ok(text.lines().map(str::to_owned).collect())
}
