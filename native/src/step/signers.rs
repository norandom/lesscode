//! The signer list of a project (spec signed-approvals, task 1.3):
//! `.lesscode/allowed_signers` as committed on the branch, in Git's
//! allowed-signers form. It names the GitHub account whose signing keys are
//! asked, and the keys that may approve. A key counts only where this list
//! and the account's keys on the forge both have it.

use super::context::Context;
use super::revisions::blob;
use super::sshsig::fingerprint;

/// Where the list lies in the project under work.
pub const PATH: &str = ".lesscode/allowed_signers";
const PRINCIPAL: &str = "github.com/";
const GUIDE: &str = "docs/approvals.md of lesscode says how to write it";

/// One key of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedKey {
    /// `ssh-ed25519` and the like.
    pub kind: String,
    pub fingerprint: String,
    /// The line of the list, from 1.
    pub line: usize,
}

/// The account and its listed keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerList {
    pub account: String,
    pub keys: Vec<ListedKey>,
}

/// What the branch has: a list, or why it gives none to judge with.
#[derive(Debug, PartialEq, Eq)]
pub enum Listed {
    List(SignerList),
    NotJudged(String),
}

/// A GitHub account name: letters, digits and hyphens, at most 39.
pub fn is_account(name: &str) -> bool {
    let allowed = |c: char| c.is_ascii_alphanumeric() || c == '-';
    (1..=39).contains(&name.len()) && name.chars().all(allowed)
}

/// The list as committed on the branch. `Err` is bad input (exit 3).
pub fn read(context: &Context) -> Result<Listed, String> {
    let Some(hash) = context.step_tree.get(PATH) else {
        let branch = &context.declaration.branch;
        return Ok(Listed::NotJudged(format!(
            "an approval is signed, and {branch} has no signer list at {PATH} to judge it \
             with; {GUIDE}."
        )));
    };
    let bytes = blob(&context.repo, hash)?;
    parse(&String::from_utf8_lossy(&bytes))
}

/// Split options at commas outside quotes.
fn options(text: &str) -> Vec<String> {
    let (mut parts, mut part, mut quoted) = (Vec::new(), String::new(), false);
    for c in text.chars() {
        match (c, quoted) {
            ('"', _) => quoted = !quoted,
            (',', false) => parts.push(std::mem::take(&mut part)),
            _ => part.push(c),
        }
    }
    parts.push(part);
    parts
}

/// Only `namespaces` is allowed, and it must include `git`.
fn check_options(text: &str, line: usize) -> Result<(), String> {
    for option in options(text) {
        let namespaces = option.strip_prefix("namespaces=");
        let has_git = namespaces.is_some_and(|n| n.split(',').any(|n| n.trim() == "git"));
        if !has_git {
            return Err(format!(
                "{PATH}, line {line}: the option `{option}` is not supported; only \
                 namespaces=\"git\" is."
            ));
        }
    }
    Ok(())
}

fn is_key_kind(word: &str) -> bool {
    ["ssh-", "sk-ssh-", "ecdsa-", "sk-ecdsa-"]
        .iter()
        .any(|p| word.starts_with(p))
}

/// One line: its account (if it names one) and its key. `Err` is bad input.
fn line_of(text: &str, line: usize) -> Result<(Option<String>, ListedKey), String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let unreadable = || format!("{PATH}, line {line}: not a line of principals and a key.");
    let at = match words.get(1) {
        Some(w) if is_key_kind(w) => 1,
        Some(w) => {
            check_options(w, line)?;
            2
        }
        None => return Err(unreadable()),
    };
    let (kind, blob) = match (words.get(at), words.get(at + 1)) {
        (Some(kind), Some(blob)) if is_key_kind(kind) => (*kind, *blob),
        _ => return Err(unreadable()),
    };
    let fingerprint = fingerprint(blob).map_err(|e| format!("{PATH}, line {line}: {e}"))?;
    let accounts: Vec<&str> = words[0]
        .split(',')
        .filter_map(|p| p.strip_prefix(PRINCIPAL))
        .collect();
    if let Some(bad) = accounts.iter().find(|a| !is_account(a)) {
        return Err(format!(
            "{PATH}, line {line}: `{bad}` is not a GitHub account name."
        ));
    }
    let key = ListedKey {
        kind: kind.to_owned(),
        fingerprint,
        line,
    };
    match accounts.as_slice() {
        [] => Ok((None, key)),
        [one] => Ok((Some((*one).to_owned()), key)),
        _ => Err(format!("{PATH}, line {line}: names more than one account.")),
    }
}

/// The list from its text. Every line must name the same account as
/// `github.com/<account>`; other principals are allowed and ignored.
pub fn parse(text: &str) -> Result<Listed, String> {
    let mut account: Option<String> = None;
    let mut keys = Vec::new();
    let lines = text.lines().enumerate().map(|(i, l)| (i + 1, l.trim()));
    for (n, line) in lines.filter(|(_, l)| !l.is_empty() && !l.starts_with('#')) {
        let (named, key) = line_of(line, n)?;
        let Some(named) = named else {
            return Ok(Listed::NotJudged(format!(
                "{PATH}, line {n}, names no account as {PRINCIPAL}<account>; {GUIDE}."
            )));
        };
        if account.as_ref().is_some_and(|a| *a != named) {
            return Ok(Listed::NotJudged(format!(
                "{PATH} names two accounts, {} and {named}; one list names one account.",
                account.unwrap_or_default()
            )));
        }
        account = Some(named);
        keys.push(key);
    }
    match account {
        Some(account) => Ok(Listed::List(SignerList { account, keys })),
        None => Ok(Listed::NotJudged(format!("{PATH} names no key; {GUIDE}."))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGBoSr4+jDm3exR7J4y+Zc2fhHNRAvTiVHya79DoD6eH";
    const PRINT: &str = "SHA256:XWnFnHsrIGlcBugd/aj0NIsR6T0rqMUBJ8mNG9F2UdE";

    fn list(text: &str) -> SignerList {
        match parse(text).unwrap() {
            Listed::List(list) => list,
            other => panic!("{other:?}"),
        }
    }

    fn not_judged(text: &str, words: &str) {
        match parse(text).unwrap() {
            Listed::NotJudged(why) => assert!(why.contains(words), "{why}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_list_names_its_account_and_the_fingerprints_of_its_keys() {
        let text = format!(
            "# the owner's key\n\ngithub.com/norandom,me@example.invalid namespaces=\"git\" {KEY} lesscode\n\
             github.com/norandom {KEY}\n"
        );
        let list = list(&text);
        assert_eq!(list.account, "norandom");
        assert_eq!(list.keys.len(), 2);
        assert_eq!(list.keys[0].fingerprint, PRINT);
        assert_eq!((list.keys[0].line, list.keys[1].line), (3, 4));
        assert_eq!(list.keys[1].kind, "ssh-ed25519");
        assert!(list.keys.iter().all(|k| k.fingerprint == PRINT));
    }

    #[test]
    fn a_list_without_an_account_or_with_two_is_not_judged() {
        not_judged("", "names no key");
        not_judged("# only a comment\n", "names no key");
        not_judged(
            &format!("me@example.invalid {KEY}\n"),
            "line 1, names no account",
        );
        let two = format!("github.com/a {KEY}\ngithub.com/b {KEY}\n");
        not_judged(&two, "names two accounts, a and b");
    }

    #[test]
    fn other_options_and_unreadable_lines_are_bad_input() {
        let ca = format!("github.com/a cert-authority {KEY}\n");
        assert!(parse(&ca)
            .unwrap_err()
            .contains("line 1: the option `cert-authority`"));
        let file_only = format!("github.com/a namespaces=\"file\" {KEY}\n");
        assert!(parse(&file_only).unwrap_err().contains("namespaces=file"));
        let both = format!("github.com/a namespaces=\"file,git\" {KEY}\n");
        assert_eq!(list(&both).account, "a");
        assert!(parse("github.com/a\n")
            .unwrap_err()
            .contains("line 1: not a line"));
        let broken = "github.com/a ssh-ed25519 !!!\n";
        assert!(parse(broken).unwrap_err().contains("not a public key"));
        let bad = format!("github.com/not_an_account {KEY}\n");
        assert!(parse(&bad)
            .unwrap_err()
            .contains("is not a GitHub account name"));
        let twice = format!("github.com/a,github.com/b {KEY}\n");
        assert!(parse(&twice).unwrap_err().contains("more than one account"));
    }

    #[test]
    fn account_names_follow_github() {
        assert!(is_account("norandom") && is_account("a-b-9"));
        assert!(!is_account("") && !is_account("a_b") && !is_account("a/b"));
        assert!(!is_account(&"x".repeat(40)));
    }
}
