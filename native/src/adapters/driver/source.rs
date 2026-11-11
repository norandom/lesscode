//! The files a command works on, read without changing them, and the text
//! conventions the site rules share with the retained prototype receipts.

use super::workspace::METADATA;
use crate::specimen_runner::sha256_hex_bytes;
use std::path::{Component, Path, PathBuf};

/// A source root, the files in scope and the path prefixes that hold tests.
#[derive(Debug, Clone)]
pub struct SourceSet {
    pub root: PathBuf,
    pub files: Vec<String>,
    pub test_paths: Vec<String>,
}

impl SourceSet {
    /// The exact text of a file in the root, line endings kept.
    pub fn raw(&self, file: &str) -> Result<String, String> {
        let relative = Path::new(file);
        let inside = relative
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
        if !inside || file.is_empty() {
            return Err(format!("{file}: not a path inside the source root"));
        }
        let path = self.root.join(relative);
        let bytes =
            std::fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        String::from_utf8(bytes).map_err(|e| format!("{} is not UTF-8: {e}", path.display()))
    }

    /// The text of a file with every line ending as a line feed.
    pub fn text(&self, file: &str) -> Result<String, String> {
        self.raw(file).map(|text| universal_newlines(&text))
    }

    /// Whether a path with forward slashes lies under a test path.
    pub fn is_test_file(&self, normalized: &str) -> bool {
        let slashed = format!("/{normalized}");
        self.test_paths
            .iter()
            .any(|p| normalized.starts_with(p.as_str()) || slashed.contains(&format!("/{p}")))
    }
}

/// One hash over the test files of the tree: every file under a test path,
/// by relative path and content, in sorted order. Version-control metadata
/// and the excluded names or relative paths are left out, as in a copy.
/// `None` when no test path is declared.
pub fn test_files_digest(
    source: &SourceSet,
    exclusions: &[String],
) -> Result<Option<String>, String> {
    if source.test_paths.is_empty() {
        return Ok(None);
    }
    let mut lines = Vec::new();
    let mut stack = vec![PathBuf::new()];
    while let Some(relative) = stack.pop() {
        let dir = source.root.join(&relative);
        let failed = |e: std::io::Error| format!("{}: {e}", dir.display());
        for entry in std::fs::read_dir(&dir).map_err(failed)? {
            let entry = entry.map_err(failed)?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let below = relative.join(&name);
            let slashed = below.to_string_lossy().replace('\\', "/");
            let excluded = |e: &String| *e == name || *e == slashed;
            if METADATA.contains(&name.as_str()) || exclusions.iter().any(excluded) {
                continue;
            }
            let kind = entry.file_type().map_err(failed)?;
            if kind.is_dir() {
                stack.push(below);
            } else if kind.is_file() && source.is_test_file(&slashed) {
                let bytes = std::fs::read(entry.path()).map_err(failed)?;
                lines.push(format!("{slashed}\t{}", sha256_hex_bytes(&bytes)));
            }
        }
    }
    lines.sort();
    Ok(Some(sha256_hex_bytes(lines.join("\n").as_bytes())))
}

/// CRLF and lone CR become LF.
pub fn universal_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// White space as the retained receipts counted it.
pub fn is_space(c: char) -> bool {
    c.is_whitespace() || ('\x1c'..='\x1f').contains(&c)
}

pub fn strip(text: &str) -> &str {
    text.trim_matches(is_space)
}

/// Lines without their endings; CRLF is one ending and a final ending does
/// not open an empty last line.
pub fn split_lines(text: &str) -> Vec<&str> {
    let is_break = |c: char| {
        matches!(
            c,
            '\n' | '\r' | '\x0b' | '\x0c' | '\x1c'..='\x1e' | '\u{85}' | '\u{2028}' | '\u{2029}'
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

#[cfg(test)]
mod tests {
    use super::*;

    fn set(name: &str) -> SourceSet {
        let root =
            std::env::temp_dir().join(format!("lesscode-source-{name}-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        SourceSet {
            root,
            files: vec!["src/a.txt".into()],
            test_paths: vec!["tests/".into(), "src/test".into()],
        }
    }

    #[test]
    fn raw_keeps_line_endings_and_text_normalises_them() {
        let s = set("raw");
        std::fs::write(s.root.join("src/a.txt"), "a\r\nb\rc\n").unwrap();
        assert_eq!(s.raw("src/a.txt").unwrap(), "a\r\nb\rc\n");
        assert_eq!(s.text("src/a.txt").unwrap(), "a\nb\nc\n");
        std::fs::remove_dir_all(&s.root).unwrap();
    }

    #[test]
    fn paths_outside_the_root_and_bad_files_are_errors() {
        let s = set("bad");
        std::fs::write(s.root.join("src/a.txt"), [0xff, 0xfe]).unwrap();
        assert!(s.raw("src/a.txt").unwrap_err().contains("not UTF-8"));
        assert!(s
            .raw("src/missing.txt")
            .unwrap_err()
            .contains("cannot read"));
        let outside = s.root.join("src/a.txt");
        for file in ["../x", "src/../../x", "", outside.to_str().unwrap()] {
            assert!(
                s.raw(file).unwrap_err().contains("not a path inside"),
                "{file}"
            );
        }
        std::fs::remove_dir_all(&s.root).unwrap();
    }

    #[test]
    fn test_paths_match_as_prefix_or_inner_directory() {
        let s = set("tests");
        assert!(s.is_test_file("tests/unit.rs"));
        assert!(s.is_test_file("pkg/tests/unit.rs"));
        assert!(s.is_test_file("src/test/pkg/a.txt"));
        assert!(!s.is_test_file("src/main/contests.rs"));
        std::fs::remove_dir_all(&s.root).unwrap();
    }

    #[test]
    fn lines_and_strip_follow_the_receipt_conventions() {
        assert_eq!(split_lines("a\r\nb\rc\n"), vec!["a", "b", "c"]);
        assert_eq!(split_lines("a\n\nb"), vec!["a", "", "b"]);
        assert_eq!(split_lines("x\x0cy"), vec!["x", "y"]);
        assert!(split_lines("").is_empty());
        assert_eq!(strip("\x1c a;\t\n"), "a;");
    }
}
