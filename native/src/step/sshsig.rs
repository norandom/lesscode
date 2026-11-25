//! The SSH signature of a commit (spec signed-approvals, task 1.2): the
//! signed bytes and the signature taken apart, the signature checked by
//! `ssh-keygen -Y check-novalidate`, and the `SHA256:` fingerprint of a
//! public key computed here.
//!
//! `ssh-keygen` is started with fixed arguments, without a terminal and
//! without the SSH agent. It reads no allowed-signers file: which keys count
//! is decided by the caller from the fingerprint.

use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The headers Git keeps a commit's signature in.
const HEADERS: [&str; 2] = ["gpgsig", "gpgsig-sha256"];
const SSH_ARMOR: &[u8] = b"-----BEGIN SSH SIGNATURE-----";

/// A commit object taken apart: the bytes a signature covers, and the
/// signature when there is one.
#[derive(Debug, PartialEq, Eq)]
pub struct Split {
    pub payload: Vec<u8>,
    pub signature: Option<Vec<u8>>,
}

impl Split {
    /// Whether the signature is an SSH signature (and not, say, PGP's).
    pub fn is_ssh(&self) -> bool {
        self.signature
            .as_deref()
            .is_some_and(|s| s.starts_with(SSH_ARMOR))
    }
}

/// The name of a signature header that `line` begins, if any.
fn header_of(line: &[u8]) -> Option<&'static str> {
    let named = |name: &&'static str| {
        line.strip_prefix(name.as_bytes())
            .is_some_and(|rest| rest.first() == Some(&b' '))
    };
    HEADERS.into_iter().find(named)
}

/// Take a raw commit object apart. Only the header block, up to the first
/// empty line, is searched; a header's continuation lines begin with a
/// space. `gpgsig` is taken before `gpgsig-sha256`; both leave the payload.
pub fn split(raw: &[u8]) -> Split {
    let mut payload = Vec::with_capacity(raw.len());
    let mut found: Vec<(&str, Vec<u8>)> = Vec::new();
    let mut current: Option<usize> = None;
    let mut in_header = true;
    for line in raw.split_inclusive(|b| *b == b'\n') {
        if in_header && (line == b"\n") {
            in_header = false;
        }
        let continued = in_header && current.is_some() && line.first() == Some(&b' ');
        if let (true, Some(at)) = (continued, current) {
            found[at].1.extend_from_slice(&line[1..]);
            continue;
        }
        current = None;
        if let Some(name) = header_of(line).filter(|_| in_header) {
            found.push((name, line[name.len() + 1..].to_vec()));
            current = Some(found.len() - 1);
            continue;
        }
        payload.extend_from_slice(line);
    }
    let first = |name: &str| found.iter().find(|(n, _)| *n == name).map(|f| f.1.clone());
    let signature = first(HEADERS[0]).or_else(|| first(HEADERS[1]));
    Split { payload, signature }
}

/// What `ssh-keygen` said of a signature.
#[derive(Debug, PartialEq, Eq)]
pub enum Checked {
    /// The signature verifies; made with the key of this fingerprint.
    Good { fingerprint: String },
    /// It does not verify, with `ssh-keygen`'s words.
    Bad(String),
}

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A file of its own for one signature, under the temporary directory.
fn signature_file(signature: &[u8]) -> Result<PathBuf, String> {
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let name = format!("lesscode-sshsig-{}-{n}.sig", std::process::id());
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, signature).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// The first `SHA256:<base64>` word of a text.
fn fingerprint_in(text: &str) -> Option<String> {
    let word = text.split_whitespace().find(|w| w.starts_with("SHA256:"))?;
    let base64 = |c: char| c.is_ascii_alphanumeric() || c == '+' || c == '/';
    let end = word[7..]
        .find(|c: char| !base64(c))
        .map_or(word.len(), |i| i + 7);
    Some(word[..end].to_owned()).filter(|f| f.len() > 7)
}

/// Check a signature over `payload` in the namespace `git`. `Err` when
/// `ssh-keygen` cannot be started or answers nothing that can be read.
pub fn check(payload: &[u8], signature: &[u8]) -> Result<Checked, String> {
    let file = signature_file(signature)?;
    let result = run_check(payload, &file);
    let _ = std::fs::remove_file(&file);
    result
}

fn run_check(payload: &[u8], file: &std::path::Path) -> Result<Checked, String> {
    let mut child = Command::new("ssh-keygen")
        .args(["-Y", "check-novalidate", "-n", "git", "-s"])
        .arg(file)
        .env_remove("SSH_AUTH_SOCK")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("ssh-keygen cannot be started: {e}"))?;
    let mut stdin = child.stdin.take().expect("piped");
    let fed = stdin.write_all(payload);
    drop(stdin);
    let done = child
        .wait_with_output()
        .map_err(|e| format!("ssh-keygen: {e}"))?;
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&done.stdout),
        String::from_utf8_lossy(&done.stderr)
    );
    let said = said.trim().to_owned();
    match (done.status.success(), fed.is_ok(), fingerprint_in(&said)) {
        (true, true, Some(fingerprint)) => Ok(Checked::Good { fingerprint }),
        (true, _, _) => Err(format!("ssh-keygen gave no fingerprint: {said}")),
        (false, _, _) => Ok(Checked::Bad(said)),
    }
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64, padding optional.
fn decode(text: &str) -> Option<Vec<u8>> {
    let digits = text.trim_end_matches('=').as_bytes();
    let value = |c: u8| ALPHABET.iter().position(|a| *a == c).map(|v| v as u32);
    let mut out = Vec::with_capacity(digits.len() * 3 / 4);
    for chunk in digits.chunks(4) {
        let mut word = 0u32;
        for (i, c) in chunk.iter().enumerate() {
            word |= value(*c)? << (18 - 6 * i);
        }
        let bytes = word.to_be_bytes();
        let count = match chunk.len() {
            4 => 3,
            3 => 2,
            2 => 1,
            _ => return None,
        };
        out.extend_from_slice(&bytes[1..=count]);
    }
    Some(out)
}

/// Standard base64 without padding, as OpenSSH prints fingerprints.
fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 4 / 3 + 1);
    for chunk in bytes.chunks(3) {
        let mut word = 0u32;
        for (i, b) in chunk.iter().enumerate() {
            word |= u32::from(*b) << (16 - 8 * i);
        }
        for i in 0..=chunk.len() {
            out.push(ALPHABET[((word >> (18 - 6 * i)) & 63) as usize] as char);
        }
    }
    out
}

/// The `SHA256:` fingerprint of a public key from its base64 blob (the
/// second word of an OpenSSH public key line).
pub fn fingerprint(blob_base64: &str) -> Result<String, String> {
    let blob = decode(blob_base64.trim())
        .filter(|b| !b.is_empty())
        .ok_or_else(|| format!("not a public key: {blob_base64:?}"))?;
    Ok(format!("SHA256:{}", encode(&Sha256::digest(&blob))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn keygen(args: &[&str]) -> String {
        let done = Command::new("ssh-keygen")
            .args(args)
            .env_remove("SSH_AUTH_SOCK")
            .stdin(Stdio::null())
            .output()
            .expect("ssh-keygen");
        let said = String::from_utf8_lossy(&done.stdout).into_owned();
        assert!(done.status.success(), "ssh-keygen {args:?}: {said}");
        said
    }

    /// A key pair and a signature of `message` in `namespace`.
    fn signed(dir: &Path, namespace: &str, message: &[u8]) -> (PathBuf, Vec<u8>) {
        std::fs::create_dir_all(dir).unwrap();
        let key = dir.join(format!("key-{namespace}"));
        let _ = std::fs::remove_file(&key);
        let _ = std::fs::remove_file(key.with_extension("pub"));
        let k = key.to_string_lossy().into_owned();
        keygen(&["-q", "-t", "ed25519", "-N", "", "-C", "unit", "-f", &k]);
        let file = dir.join(format!("message-{namespace}"));
        std::fs::write(&file, message).unwrap();
        let f = file.to_string_lossy().into_owned();
        keygen(&["-Y", "sign", "-q", "-n", namespace, "-f", &k, &f]);
        let signature = std::fs::read(format!("{f}.sig")).unwrap();
        (PathBuf::from(format!("{k}.pub")), signature)
    }

    fn scratch(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("lc-sshsig-{name}-{}", std::process::id()))
    }

    const SIGNED: &[u8] = b"tree 1\nparent 2\nauthor a\ncommitter c\ngpgsig -----BEGIN SSH SIGNATURE-----\n U1NIU0lH\n -----END SSH SIGNATURE-----\n\nmessage\n with a line that begins with a space\n";

    #[test]
    fn split_takes_the_signature_out_of_the_header_only() {
        let split = split(SIGNED);
        let payload = b"tree 1\nparent 2\nauthor a\ncommitter c\n\nmessage\n with a line that begins with a space\n";
        assert_eq!(split.payload, payload.to_vec());
        let signature = b"-----BEGIN SSH SIGNATURE-----\nU1NIU0lH\n-----END SSH SIGNATURE-----\n";
        assert_eq!(split.signature.as_deref(), Some(&signature[..]));
        assert!(split.is_ssh());
    }

    #[test]
    fn split_reads_gpgsig_sha256_and_sees_an_unsigned_or_pgp_commit() {
        let sha256 = String::from_utf8_lossy(SIGNED).replace("gpgsig ", "gpgsig-sha256 ");
        assert_eq!(split(sha256.as_bytes()).signature, split(SIGNED).signature);
        let unsigned = b"tree 1\ncommitter c\n\ngpgsig in the message\n";
        assert_eq!(split(unsigned).signature, None);
        assert_eq!(split(unsigned).payload, unsigned.to_vec());
        let pgp = b"tree 1\ngpgsig -----BEGIN PGP SIGNATURE-----\n x\n\nm\n";
        assert!(split(pgp).signature.is_some() && !split(pgp).is_ssh());
    }

    #[test]
    fn a_good_signature_names_its_key_and_a_changed_one_fails() {
        let dir = scratch("check");
        let (public, signature) = signed(&dir, "git", b"approved\n");
        let expected = keygen(&["-l", "-f", &public.to_string_lossy()]);
        let Checked::Good { fingerprint } = check(b"approved\n", &signature).unwrap() else {
            panic!("the signature does not verify");
        };
        assert!(
            expected.contains(&fingerprint),
            "{fingerprint} in {expected}"
        );
        let changed = check(b"Approved\n", &signature).unwrap();
        assert!(matches!(changed, Checked::Bad(_)), "{changed:?}");
        let (_, other) = signed(&dir, "file", b"approved\n");
        let namespace = check(b"approved\n", &other).unwrap();
        assert!(matches!(namespace, Checked::Bad(_)), "{namespace:?}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_fingerprint_is_the_one_ssh_keygen_prints() {
        let dir = scratch("fingerprint");
        let (public, _) = signed(&dir, "git", b"x");
        let line = std::fs::read_to_string(&public).unwrap();
        let blob = line.split_whitespace().nth(1).unwrap();
        let printed = keygen(&["-l", "-f", &public.to_string_lossy()]);
        assert_eq!(Some(fingerprint(blob).unwrap()), fingerprint_in(&printed));
        assert!(fingerprint("not base64!").is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn base64_round_trips() {
        for bytes in [&b""[..], b"a", b"ab", b"abc", b"abcd", &[0, 255, 128, 7, 9]] {
            assert_eq!(decode(&encode(bytes)).unwrap(), bytes.to_vec());
        }
        assert_eq!(encode(b"abc"), "YWJj");
        assert_eq!(decode("YWI=").unwrap(), b"ab".to_vec());
    }
}
