//! Validation of sites selected elsewhere (annotation contract version 0):
//! `{file, start_line, end_line, kind, anchor, reason}`. An annotation is
//! accepted only if it names exactly one node that the pack allows for its
//! kind, in a scope file that is not a test.

use super::pack::Pack;
use super::sites::{site_at, site_check, Site, OK, OPERATOR_KINDS};
use super::source::{strip, universal_newlines, SourceSet};
use super::tree::{grammar, parse, walk, SgNode, SgRoot};
use super::Kind;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

pub const MALFORMED: &str = "malformed";
pub const TEST_FILE: &str = "test_file";
pub const OUTSIDE_SCOPE: &str = "outside_scope";
pub const ANCHOR_MISMATCH: &str = "anchor_mismatch";
pub const ANCHOR_AMBIGUOUS: &str = "anchor_ambiguous";
pub const DUPLICATE: &str = "duplicate";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Decision {
    pub accepted: bool,
    pub reason: &'static str,
    /// The anchor matched only with the statement terminator appended.
    pub anchor_completed: bool,
}

impl Decision {
    /// The reason, marked when the anchor had to be completed.
    pub fn outcome(&self) -> String {
        let mark = if self.anchor_completed {
            "+anchor_completed"
        } else {
            ""
        };
        format!("{}{mark}", self.reason)
    }
}

/// The fields of an annotation that has the contract's shape.
struct Fields<'a> {
    file: String,
    start: usize,
    end: usize,
    kind: Kind,
    anchor: &'a str,
}

/// Whole numbers; true and false count as 1 and 0, as in the retained
/// receipts.
fn line(value: Option<&Value>) -> Option<usize> {
    let number = match value? {
        Value::Number(n) => n.as_u64()?,
        Value::Bool(b) => u64::from(*b),
        _ => return None,
    };
    usize::try_from(number).ok()
}

fn fields(annotation: &Value) -> Option<Fields<'_>> {
    let a = annotation.as_object()?;
    let name = a.get("kind")?.as_str()?;
    let kind = Kind::ALL.into_iter().find(|k| k.name() == name)?;
    let anchor = a.get("anchor")?.as_str()?;
    let (start, end) = (line(a.get("start_line"))?, line(a.get("end_line"))?);
    if strip(anchor).is_empty() || start < 1 || end < start {
        return None;
    }
    let file = a.get("file").and_then(Value::as_str).unwrap_or_default();
    Some(Fields {
        file: file.replace('\\', "/"),
        start,
        end,
        kind,
        anchor,
    })
}

/// Named nodes on exactly these lines whose text is the anchor.
fn anchor_nodes<'r>(root: &SgNode<'r>, f: &Fields<'_>, anchor: &str) -> Vec<SgNode<'r>> {
    walk(root)
        .into_iter()
        .filter(|n| {
            n.is_named()
                && n.start_pos().line() + 1 == f.start
                && n.end_pos().line() + 1 == f.end
                && universal_newlines(&n.text()) == anchor
        })
        .collect()
}

/// The nodes an anchor names, and whether the statement terminator had to
/// be appended to find them.
fn match_anchor<'r>(pack: &Pack, root: &SgNode<'r>, f: &Fields<'_>) -> (Vec<SgNode<'r>>, bool) {
    let nodes = anchor_nodes(root, f, f.anchor);
    let terminator = pack.terminator();
    if !nodes.is_empty() || terminator.is_empty() {
        return (nodes, false);
    }
    let completed = anchor_nodes(root, f, &format!("{}{terminator}", f.anchor));
    let found = !completed.is_empty();
    (completed, found)
}

/// Validates a list of annotations in order; a repeated site is a duplicate.
pub struct Validator<'a> {
    source: &'a SourceSet,
    pack: &'a Pack,
    roots: HashMap<String, SgRoot>,
    accepted: BTreeSet<(String, (usize, usize), Kind)>,
}

impl<'a> Validator<'a> {
    pub fn new(source: &'a SourceSet, pack: &'a Pack) -> Self {
        Validator {
            source,
            pack,
            roots: HashMap::new(),
            accepted: BTreeSet::new(),
        }
    }

    fn root(&mut self, file: &str) -> Result<&SgRoot, String> {
        if !self.roots.contains_key(file) {
            let lang = grammar(&self.pack.language)?;
            let root = parse(&self.source.text(file)?, lang)?;
            self.roots.insert(file.to_owned(), root);
        }
        Ok(&self.roots[file])
    }

    fn decide(&mut self, annotation: &Value) -> Result<(&'static str, bool), String> {
        let Some(f) = fields(annotation) else {
            return Ok((MALFORMED, false));
        };
        if self.source.is_test_file(&f.file) {
            return Ok((TEST_FILE, false));
        }
        if !self.source.files.contains(&f.file) {
            return Ok((OUTSIDE_SCOPE, false));
        }
        let pack = self.pack;
        let root = self.root(&f.file)?.root();
        let (matches, completed) = match_anchor(pack, &root, &f);
        let spans: BTreeSet<_> = matches
            .iter()
            .map(|n| n.range())
            .map(|r| (r.start, r.end))
            .collect();
        let span = match (spans.len(), spans.first()) {
            (1, Some(span)) => *span,
            (0, _) => return Ok((ANCHOR_MISMATCH, completed)),
            _ => return Ok((ANCHOR_AMBIGUOUS, completed)),
        };
        let (reason, _) = site_check(pack, &matches[0], f.kind);
        if reason != OK {
            return Ok((reason, completed));
        }
        if !self.accepted.insert((f.file, span, f.kind)) {
            return Ok((DUPLICATE, completed));
        }
        Ok((OK, completed))
    }

    /// The decision for the next annotation of the list.
    pub fn check(&mut self, annotation: &Value) -> Result<Decision, String> {
        let (reason, anchor_completed) = self.decide(annotation)?;
        Ok(Decision {
            accepted: reason == OK,
            reason,
            anchor_completed,
        })
    }
}

/// One decision per annotation, in order.
pub fn validate(
    source: &SourceSet,
    pack: &Pack,
    annotations: &[Value],
) -> Result<Vec<Decision>, String> {
    let mut validator = Validator::new(source, pack);
    annotations.iter().map(|a| validator.check(a)).collect()
}

/// The site an accepted annotation names, in the file as it is on disk.
/// None when the kind has no operators or the condition is excluded.
pub fn site_of(
    source: &SourceSet,
    pack: &Pack,
    annotation: &Value,
) -> Result<Option<Site>, String> {
    let Some(f) = fields(annotation).filter(|f| OPERATOR_KINDS.contains(&f.kind)) else {
        return Ok(None);
    };
    let root = parse(&source.raw(&f.file)?, grammar(&pack.language)?)?;
    let (nodes, _) = match_anchor(pack, &root.root(), &f);
    Ok(nodes
        .first()
        .and_then(|node| site_at(pack, &f.file, f.kind, node)))
}
