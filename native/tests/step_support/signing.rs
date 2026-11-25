//! Signed approvals in tests (signed-approvals, task 1.1): SSH keys made per
//! test without a passphrase, commits signed with them, a signature grafted
//! from one commit onto another, and the answer a stand-in forge gives.
//!
//! Nothing here touches a Git setting outside the command it runs: signing
//! is switched on with `-c` for one commit, and the SSH agent is kept out.

use super::Repo;
use lesscode_core::step::forge::{Forge, ForgeError, ForgeKey};
use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// An Ed25519 key pair without a passphrase, in its own directory.
pub struct Key {
    pub private: PathBuf,
    pub public: PathBuf,
}

impl Key {
    /// Make the pair `<dir>/<name>` and `<dir>/<name>.pub`.
    pub fn new(dir: &Path, name: &str) -> Key {
        fs::create_dir_all(dir).unwrap();
        let private = dir.join(name);
        let _ = fs::remove_file(&private);
        let _ = fs::remove_file(private.with_extension("pub"));
        let path = private.to_string_lossy().into_owned();
        let args = ["-q", "-t", "ed25519", "-N", "", "-C", name, "-f", &path];
        ssh_keygen(&args, None);
        let public = PathBuf::from(format!("{path}.pub"));
        Key { private, public }
    }

    /// The pair `<dir>/<name>` made earlier in the test, else a new one.
    pub fn kept(dir: &Path, name: &str) -> Key {
        let private = dir.join(name);
        let public = PathBuf::from(format!("{}.pub", private.display()));
        match public.exists() {
            true => Key { private, public },
            false => Key::new(dir, name),
        }
    }

    /// `<type> <base64>`, without the comment.
    pub fn public_key(&self) -> String {
        let line = fs::read_to_string(&self.public).unwrap();
        let mut parts = line.split_whitespace();
        format!("{} {}", parts.next().unwrap(), parts.next().unwrap())
    }

    /// `SHA256:…`, as `ssh-keygen -l` prints it.
    pub fn fingerprint(&self) -> String {
        let path = self.public.to_string_lossy().into_owned();
        let said = ssh_keygen(&["-l", "-f", &path], None);
        let word = said.split_whitespace().find(|w| w.starts_with("SHA256:"));
        word.unwrap_or_else(|| panic!("no fingerprint in: {said}"))
            .to_owned()
    }
}

/// Run `ssh-keygen` with these arguments and stdin; its stdout, or a panic
/// with what it said.
pub fn ssh_keygen(args: &[&str], stdin: Option<&[u8]>) -> String {
    let (ok, out, err) = ssh_keygen_status(args, stdin);
    assert!(ok, "ssh-keygen {args:?}: {out}{err}");
    out
}

/// Run `ssh-keygen`; whether it succeeded, its stdout and its stderr.
pub fn ssh_keygen_status(args: &[&str], stdin: Option<&[u8]>) -> (bool, String, String) {
    let mut command = Command::new("ssh-keygen");
    command
        .args(args)
        .env_remove("SSH_AUTH_SOCK")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("ssh-keygen cannot be started");
    let mut input = child.stdin.take().unwrap();
    if let Some(bytes) = stdin {
        std::io::Write::write_all(&mut input, bytes).unwrap();
    }
    drop(input);
    let done = child.wait_with_output().unwrap();
    let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
    (
        done.status.success(),
        text(&done.stdout),
        text(&done.stderr),
    )
}

impl Repo {
    /// Write the files and commit them with a signature made with `key`;
    /// returns the commit. Signing is switched on for this commit only.
    pub fn signed_commit(&self, key: &Key, files: &[(&str, &str)], message: &str) -> String {
        for (path, text) in files {
            self.write(path, text);
        }
        self.git(&["add", "-A"]);
        let signing_key = format!("user.signingkey={}", key.private.display());
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .args([
                "-c",
                "user.name=test",
                "-c",
                "user.email=test@example.invalid",
            ])
            .args(["-c", "gpg.format=ssh", "-c", &signing_key])
            .args(["commit", "-q", "-S", "-m", message])
            .env_remove("SSH_AUTH_SOCK")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "signed commit: {said}");
        self.git(&["rev-parse", "HEAD"])
    }

    /// A new commit object: `onto` with the signature header of `from`, so
    /// that the signature belongs to other content. Written to the object
    /// store and returned; no branch is moved.
    pub fn grafted(&self, from: &str, onto: &str) -> String {
        let signature = signature_header(&self.git(&["cat-file", "commit", from]));
        assert!(!signature.is_empty(), "{from} carries no signature");
        let object = self.git(&["cat-file", "commit", onto]);
        let unsigned = without_signature(&object);
        let (head, message) = unsigned.split_once("\n\n").unwrap();
        let grafted = format!("{head}\n{signature}\n\n{message}\n");
        let path = self.dir.join(".git").join("grafted-commit");
        fs::write(&path, grafted).unwrap();
        let file = path.to_string_lossy().into_owned();
        let hash = self.git(&["hash-object", "-w", "-t", "commit", &file]);
        fs::remove_file(path).unwrap();
        hash
    }
}

/// The `gpgsig` header with its continuation lines, as stored.
fn signature_header(object: &str) -> String {
    let head = object.split_once("\n\n").map_or(object, |(head, _)| head);
    let mut lines = Vec::new();
    for line in head.lines() {
        let continues = !lines.is_empty() && line.starts_with(' ');
        if line.starts_with("gpgsig ") || continues {
            lines.push(line);
        } else if !lines.is_empty() {
            break;
        }
    }
    lines.join("\n")
}

/// The commit object without its `gpgsig` header.
fn without_signature(object: &str) -> String {
    let signature = signature_header(object);
    match signature.is_empty() {
        true => object.to_owned(),
        false => object.replacen(&format!("{signature}\n"), "", 1),
    }
}

/// What the stand-in forge answers for any account: these signing keys, or
/// that it cannot be reached. It counts how often it was asked. It is the
/// forge of the library entry in the tests; the command has no way to name it.
#[derive(Debug, Default)]
pub struct StandIn {
    keys: Option<Vec<ForgeKey>>,
    pub asked: Cell<usize>,
}

impl StandIn {
    /// The forge lists exactly these keys, each under its title.
    pub fn listing(keys: &[(&Key, &str)]) -> StandIn {
        let listed = |(key, title): &(&Key, &str)| ForgeKey {
            key: key.public_key(),
            title: (*title).to_owned(),
        };
        StandIn {
            keys: Some(keys.iter().map(listed).collect()),
            asked: Cell::new(0),
        }
    }

    /// The forge cannot be reached.
    pub fn unreachable() -> StandIn {
        StandIn::default()
    }

    /// The keys it lists, for a test to look at.
    pub fn keys(&self) -> &[ForgeKey] {
        self.keys.as_deref().unwrap_or(&[])
    }
}

impl Forge for StandIn {
    fn signing_keys(&self, _account: &str) -> Result<Vec<ForgeKey>, ForgeError> {
        self.asked.set(self.asked.get() + 1);
        let unreachable = || ForgeError::Unreachable("the stand-in is unreachable".into());
        self.keys.clone().ok_or_else(unreachable)
    }
}
