//! Scopes from the manifest, file reading and the Python string semantics the
//! prototype relies on.

use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub language: String,
    pub root: PathBuf,
    pub files: Vec<String>,
    pub test_paths: Vec<String>,
}

#[derive(Deserialize)]
struct Entry {
    name: String,
    language: String,
    repository: String,
    files: Vec<String>,
    test_paths: Vec<String>,
}

#[derive(Deserialize)]
struct Manifest {
    scopes: Vec<Entry>,
}

impl Scope {
    /// `Scope.source`: UTF-8 text read with universal newlines.
    pub fn source(&self, file: &str) -> Result<String, String> {
        read_text(&self.root, file).map(|text| universal_newlines(&text))
    }

    /// `engine.read_text`: the exact file text, line endings kept.
    pub fn raw(&self, file: &str) -> Result<String, String> {
        read_text(&self.root, file)
    }

    pub fn is_test_file(&self, normalized: &str) -> bool {
        let slashed = format!("/{normalized}");
        self.test_paths
            .iter()
            .any(|p| normalized.starts_with(p.as_str()) || slashed.contains(&format!("/{p}")))
    }
}

fn read_text(root: &Path, file: &str) -> Result<String, String> {
    let path = root.join(file);
    let bytes = std::fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    String::from_utf8(bytes).map_err(|e| format!("{} is not UTF-8: {e}", path.display()))
}

/// `validator.load_scopes`: scope name to scope; a later duplicate name wins.
pub fn load_scopes(manifest: &Path, scopes_dir: &Path) -> Result<Vec<Scope>, String> {
    let text = std::fs::read_to_string(manifest)
        .map_err(|e| format!("cannot read {}: {e}", manifest.display()))?;
    let parsed: Manifest = serde_json::from_str(&text)
        .map_err(|e| format!("{} is not a scope manifest: {e}", manifest.display()))?;
    Ok(parsed
        .scopes
        .into_iter()
        .map(|s| Scope {
            name: s.name,
            language: s.language,
            root: scopes_dir.join(s.repository),
            files: s.files,
            test_paths: s.test_paths,
        })
        .collect())
}

pub fn find_scope<'a>(scopes: &'a [Scope], name: &str) -> Result<&'a Scope, String> {
    scopes
        .iter()
        .rev()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("unknown scope: {name}"))
}

/// Python's universal newlines: CRLF and lone CR become LF.
pub fn universal_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// `str.isspace`.
pub fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\x1c'..='\x1f').contains(&c)
}

/// `str.strip()`.
pub fn py_strip(text: &str) -> &str {
    text.trim_matches(py_space)
}

/// `str.splitlines()`: every Unicode line boundary Python knows, CRLF as one.
pub fn py_splitlines(text: &str) -> Vec<&str> {
    let is_break = |c: char| {
        matches!(
            c,
            '\n' | '\r'
                | '\x0b'
                | '\x0c'
                | '\x1c'
                | '\x1d'
                | '\x1e'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        )
    };
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if !is_break(c) {
            continue;
        }
        lines.push(&text[start..i]);
        start = i + c.len_utf8();
        if c == '\r' && matches!(chars.peek(), Some((_, '\n'))) {
            chars.next();
            start += 1;
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

/// `str(value)` for the JSON values an annotation can carry.
pub fn py_str(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "None".to_owned(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(true)) => "True".to_owned(),
        Some(Value::Bool(false)) => "False".to_owned(),
        Some(other) => other.to_string(),
    }
}

/// `isinstance(value, int)`: JSON integers and booleans.
pub fn py_int(value: Option<&Value>) -> Option<i64> {
    match value {
        Some(Value::Number(n)) => n.as_i64(),
        Some(Value::Bool(b)) => Some(i64::from(*b)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitlines_matches_python() {
        assert_eq!(py_splitlines("a\r\nb\rc\n"), vec!["a", "b", "c"]);
        assert_eq!(py_splitlines("a\n\nb"), vec!["a", "", "b"]);
        assert_eq!(py_splitlines("x\x0cy"), vec!["x", "y"]);
        assert!(py_splitlines("").is_empty());
    }

    #[test]
    fn strip_matches_python() {
        assert_eq!(py_strip("\x1c a;\t\n"), "a;");
    }
}
