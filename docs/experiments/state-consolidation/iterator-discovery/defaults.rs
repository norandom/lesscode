use super::{classes, field, tree, Index, Lines, Result, SgNode};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub(super) fn modifier(n: &SgNode<'_>, word: &str) -> bool {
    n.children()
        .any(|m| m.kind() == "modifiers" && m.text().split_whitespace().any(|w| w == word))
}

pub(super) fn empty_list(n: &SgNode<'_>, key: &str) -> bool {
    n.field(key).is_some_and(|list| {
        list.children()
            .all(|c| !c.is_named() || matches!(c.kind().as_ref(), "line_comment" | "block_comment"))
    })
}

pub(super) fn factories(d: &SgNode<'_>) -> Vec<(String, String)> {
    let Some(value) = d.field("value") else {
        return vec![];
    };
    if value.kind() != "object_creation_expression" {
        return vec![];
    }
    let Some(args) = value.field("arguments") else {
        return vec![];
    };
    args.children()
        .filter(|n| n.kind() == "method_invocation" && empty_list(n, "arguments"))
        .map(|n| (field(&n, "object"), field(&n, "name")))
        .collect()
}

// Decimal, nonnegative literals only. Refuse octal, expressions and integer
// literals whose type/range requires additional Java conversion rules.
fn decimal(s: &str) -> Option<i64> {
    let digits = s.strip_suffix(['l', 'L']).unwrap_or(s);
    if digits.is_empty()
        || !digits.bytes().all(|b| b.is_ascii_digit())
        || (digits.len() > 1 && digits.starts_with('0'))
    {
        return None;
    }
    let value: i64 = digits.parse().ok()?;
    (digits.len() != s.len() || value <= i64::from(i32::MAX)).then_some(value)
}

pub(super) struct Read {
    file: String,
    scope: String,
    target: String,
    line: usize,
    reference: String,
    member: String,
    ty: String,
    parameter_line: Option<usize>,
    subtract: Option<i64>,
}

pub(super) fn reads(n: &SgNode<'_>, file: &str) -> Vec<Read> {
    let Some(lhs) = n.field("left") else {
        return vec![];
    };
    if lhs.kind() != "field_access"
        || field(&lhs, "object") != "this"
        || !n.children().any(|c| c.kind() == "=")
    {
        return vec![];
    }
    let Some(rhs) = n.field("right") else {
        return vec![];
    };
    let Some(ctor) = tree::ancestors(n).into_iter().find(|p| {
        matches!(
            p.kind().as_ref(),
            "constructor_declaration" | "method_declaration"
        )
    }) else {
        return vec![];
    };
    if ctor.kind() != "constructor_declaration" {
        return vec![];
    }
    let Some(params) = ctor.field("parameters") else {
        return vec![];
    };
    let params: BTreeMap<_, _> = params
        .children()
        .filter(|p| p.kind() == "formal_parameter")
        .map(|p| {
            (
                field(&p, "name"),
                (
                    format!("{}{}", field(&p, "type"), field(&p, "dimensions")),
                    tree::position(&p).0,
                ),
            )
        })
        .collect();
    let scope = classes(n).into_iter().rev().collect::<Vec<_>>().join(".");
    let mut found = Vec::new();
    for access in tree::walk(&rhs)
        .into_iter()
        .filter(|c| c.kind() == "field_access")
    {
        let Some(receiver) = access.field("object") else {
            continue;
        };
        if receiver.kind() != "identifier" {
            continue;
        }
        let name = receiver.text();
        let (ty, parameter_line) = match params.get(name.as_ref()) {
            Some((ty, line)) => (ty.clone(), Some(*line)),
            None => (name.into_owned(), None),
        };
        let subtract = if rhs.range() == access.range() {
            Some(0)
        } else if rhs.kind() == "binary_expression"
            && field(&rhs, "operator") == "-"
            && rhs
                .field("left")
                .is_some_and(|l| l.range() == access.range())
        {
            decimal(&field(&rhs, "right"))
        } else {
            None
        };
        found.push(Read {
            file: file.into(),
            scope: scope.clone(),
            target: field(&lhs, "field"),
            line: tree::position(n).0,
            reference: access.text().into_owned(),
            member: field(&access, "field"),
            ty,
            parameter_line,
            subtract,
        });
    }
    found
}

impl Index {
    // This package's exported types use bare names. Resolve the nearest
    // enclosing class first, so CSVFormat.Builder and CSVParser.Builder differ.
    fn lexical_type(&self, scope: &str, ty: &str) -> Option<String> {
        let mut current = Some(scope);
        while let Some(path) = current {
            let nested = format!("{path}.{ty}");
            if self.types.contains(&nested) {
                return Some(nested);
            }
            current = path.rsplit_once('.').map(|(parent, _)| parent);
        }
        self.types.get(ty).cloned()
    }
    fn include(&self, lines: &mut Lines, file: &str, start: usize, end: usize) -> Result<()> {
        let source = self.sources.get(file).ok_or("unknown source file")?;
        let out = lines.entry(file.into()).or_default();
        for (i, s) in source
            .lines()
            .enumerate()
            .skip(start - 1)
            .take(end - start + 1)
        {
            let s = s.trim();
            if !s.is_empty() && !s.starts_with("//") {
                out.insert(i + 1, s.into());
            }
        }
        Ok(())
    }
}

pub(super) fn extend(index: &Index, lines: &mut Lines) -> Result<Vec<Value>> {
    // Freeze the seed set. Adding constructor signatures or factory bodies
    // must not silently add new data-flow roots during this pass.
    let seeds: Vec<_> = index
        .reads
        .iter()
        .filter(|r| lines.get(&r.file).is_some_and(|l| l.contains_key(&r.line)))
        .collect();
    let mut results = Vec::new();
    for read in seeds {
        let owner = index.lexical_type(&read.scope, &read.ty);
        let mut fact = json!({"target":format!("{}.{}",read.scope,read.target),"reference":read.reference,
            "assignment_file":read.file,"assignment_line":read.line,"parameter_line":read.parameter_line,
            "receiver_type":owner,"resolved":false});
        let candidate = index.fields.iter().find(|f| {
            Some(&f.scope) == owner.as_ref()
                && f.name == read.member
                && (read.parameter_line.is_some() || f.static_field)
        });
        if let Some(f) = candidate {
            if let Some(line) = read.parameter_line {
                index.include(lines, &read.file, line, line)?;
            }
            index.include(lines, &f.file, f.line, f.last_line)?;
            let default = if f.ty == "long" {
                match f.initializer.as_deref() {
                    Some(expr) => decimal(expr),
                    None => Some(0),
                }
            } else {
                None
            };
            let mut factories = Vec::new();
            for (ty, name) in &f.factories {
                if let Some(owner) = index.lexical_type(&f.scope, ty) {
                    for m in index.methods.iter().filter(|m| {
                        m.static_no_args
                            && &m.name == name
                            && m.owners.iter().rev().cloned().collect::<Vec<_>>().join(".") == owner
                    }) {
                        index.include(lines, &m.file, m.span.0, m.span.1)?;
                        factories.push(json!({"method":format!("{owner}.{name}"),"file":m.file,"line":m.span.0}));
                    }
                }
            }
            fact["resolved"] = json!(true);
            fact["declaration"] = json!({"owner":f.scope,"field":f.name,"file":f.file,"line":f.line,"type":f.ty,
                "initializer":f.initializer,"long_default":default,"factories":factories});
            fact["value_at_declaration_default"] = json!(default
                .zip(read.subtract)
                .and_then(|(v, k)| v.checked_sub(k)));
        }
        results.push(fact);
    }
    Ok(results)
}
