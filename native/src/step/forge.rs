//! The forge (spec signed-approvals, task 1.4): the signing keys a GitHub
//! account lists, each with its title. GitHub is asked through `curl`, an
//! outside program started with fixed arguments, so that the core keeps no
//! HTTP client. The answer is public and needs no token.
//!
//! `Forge` is the seam for the tests: the command always asks `GitHub`; a
//! stand-in exists only in the test suite.

use super::signers::is_account;
use serde::Deserialize;
use std::process::{Command, Stdio};

/// One signing key of the account: `<type> <base64>`, and its title.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ForgeKey {
    pub key: String,
    #[serde(default)]
    pub title: String,
}

/// Why the forge gave no keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForgeError {
    /// It could not be reached (no name, no connection, no answer in time).
    Unreachable(String),
    /// It has no such account.
    NoAccount(String),
    /// It answered with an error, or with something that is not a list.
    Refused(String),
    /// `curl` cannot be started.
    CannotStart(String),
}

impl std::fmt::Display for ForgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForgeError::Unreachable(w) => write!(f, "GitHub cannot be reached: {w}"),
            ForgeError::NoAccount(a) => write!(f, "GitHub has no account {a}"),
            ForgeError::Refused(w) => write!(f, "GitHub refused: {w}"),
            ForgeError::CannotStart(w) => write!(f, "curl cannot be started: {w}"),
        }
    }
}

/// Where the signing keys of an account are read.
pub trait Forge {
    fn signing_keys(&self, account: &str) -> Result<Vec<ForgeKey>, ForgeError>;
}

/// GitHub, through `curl`.
pub struct GitHub;

const PAGE: usize = 100;
const MAX_PAGES: usize = 10;
/// curl's exit codes for a name that does not resolve, a refused
/// connection and a timeout.
const UNREACHABLE: [i32; 3] = [6, 7, 28];

fn url(account: &str, page: usize) -> String {
    format!("https://api.github.com/users/{account}/ssh_signing_keys?per_page={PAGE}&page={page}")
}

/// What curl's run means: its exit code, its output (the body, then the
/// HTTP status on a line of its own) and its error output.
pub fn answer(
    account: &str,
    exit: Option<i32>,
    out: &str,
    err: &str,
) -> Result<Vec<ForgeKey>, ForgeError> {
    let err = err.trim().to_owned();
    match exit {
        Some(0) => {}
        Some(code) if UNREACHABLE.contains(&code) => return Err(ForgeError::Unreachable(err)),
        _ => return Err(ForgeError::Refused(format!("curl: {err}"))),
    }
    let (body, status) = out.trim_end().rsplit_once('\n').unwrap_or(("", out.trim()));
    match status.trim() {
        "200" => serde_json::from_str(body)
            .map_err(|e| ForgeError::Refused(format!("the answer is not a list of keys: {e}"))),
        "404" => Err(ForgeError::NoAccount(account.to_owned())),
        status => {
            let said: Option<serde_json::Value> = serde_json::from_str(body).ok();
            let message = said
                .as_ref()
                .and_then(|v| v["message"].as_str())
                .unwrap_or(body.trim());
            Err(ForgeError::Refused(format!("HTTP {status}: {message}")))
        }
    }
}

fn page(account: &str, n: usize) -> Result<Vec<ForgeKey>, ForgeError> {
    let done = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--max-time",
            "20",
        ])
        .args(["-H", "Accept: application/vnd.github+json"])
        .args(["-H", "User-Agent: lesscode-core"])
        .args(["-w", "\n%{http_code}"])
        .arg(url(account, n))
        .stdin(Stdio::null())
        .output()
        .map_err(|e| ForgeError::CannotStart(e.to_string()))?;
    let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
    answer(
        account,
        done.status.code(),
        &text(&done.stdout),
        &text(&done.stderr),
    )
}

impl Forge for GitHub {
    fn signing_keys(&self, account: &str) -> Result<Vec<ForgeKey>, ForgeError> {
        if !is_account(account) {
            return Err(ForgeError::Refused(format!(
                "`{account}` is not an account name"
            )));
        }
        let mut keys = Vec::new();
        for n in 1..=MAX_PAGES {
            let found = page(account, n)?;
            let last = found.len() < PAGE;
            keys.extend(found);
            if last {
                break;
            }
        }
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shape of GitHub's answer as R192 read it on 2026-11-25 (the key
    /// shortened here as in the recorded file).
    const RECORDED: &str = r#"[
  {
    "id": 616049,
    "key": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGBoSr4+jDm3exR7J4y+Zc2fhHNRAvTiVHya79DoD6eH",
    "title": "infoseclab",
    "created_at": "2025-07-11T06:08:32.812Z"
  }
]"#;

    #[test]
    fn a_list_of_keys_is_read_with_titles() {
        let keys = answer("norandom", Some(0), &format!("{RECORDED}\n200"), "").unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].title, "infoseclab");
        assert!(keys[0].key.starts_with("ssh-ed25519 AAAAC3"));
        assert_eq!(answer("a", Some(0), "[]\n200", ""), Ok(vec![]));
    }

    #[test]
    fn curl_and_http_errors_are_named() {
        let unreachable = answer("a", Some(6), "", "curl: (6) Could not resolve host\n");
        assert_eq!(
            unreachable,
            Err(ForgeError::Unreachable(
                "curl: (6) Could not resolve host".into()
            ))
        );
        assert!(matches!(
            answer("a", Some(28), "", "timeout"),
            Err(ForgeError::Unreachable(_))
        ));
        assert!(matches!(
            answer("a", Some(7), "", "refused"),
            Err(ForgeError::Unreachable(_))
        ));
        assert_eq!(
            answer("ghost", Some(0), "{\"message\":\"Not Found\"}\n404", ""),
            Err(ForgeError::NoAccount("ghost".into()))
        );
        let limited = r#"{"message":"API rate limit exceeded for 1.2.3.4."}"#;
        assert_eq!(
            answer("a", Some(0), &format!("{limited}\n403"), ""),
            Err(ForgeError::Refused(
                "HTTP 403: API rate limit exceeded for 1.2.3.4.".into()
            ))
        );
        assert!(matches!(
            answer("a", Some(0), "<html>\n200", ""),
            Err(ForgeError::Refused(_))
        ));
        assert!(matches!(
            answer("a", Some(35), "", "tls"),
            Err(ForgeError::Refused(_))
        ));
        assert!(matches!(
            answer("a", None, "", ""),
            Err(ForgeError::Refused(_))
        ));
    }

    #[test]
    fn an_account_name_is_checked_before_curl_starts() {
        let refused = GitHub.signing_keys("a b; rm -rf");
        assert!(matches!(refused, Err(ForgeError::Refused(w)) if w.contains("not an account")));
        assert!(
            url("norandom", 2).ends_with("/users/norandom/ssh_signing_keys?per_page=100&page=2")
        );
    }
}
