//! Lower just the two pinned row filters. All other source shapes are unsupported.
use lesscode_core::adapters::driver::tree::{self, SgNode};
use serde::Serialize;
use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub type Case = [u8; 4];
pub const SYMBOLS: [&str; 4] = ["old_digest", "old_revision", "new_digest", "new_revision"];

fn unique<'a>(items: impl Iterator<Item = SgNode<'a>>, label: &str) -> Result<SgNode<'a>> {
    let mut items = items;
    let first = items.next().ok_or_else(|| format!("missing {label}"))?;
    if items.next().is_some() {
        return Err(format!("ambiguous {label}").into());
    }
    Ok(first)
}

fn field(node: &SgNode<'_>, key: &str) -> Option<String> {
    node.field(key).map(|n| n.text().into_owned())
}

fn tuple(source: &str) -> Result<[usize; 2]> {
    let inner = source
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or("expected tuple")?;
    let member = |s: &str| match s.trim() {
        "r.get(\"baseline_digest\")" => Ok(0),
        "r.get(\"rules_revision\")" => Ok(1),
        "newest.get(\"baseline_digest\")" => Ok(2),
        "newest.get(\"rules_revision\")" => Ok(3),
        other => Err(format!("unsupported row access {other}")),
    };
    let (a, b) = inner.split_once(',').ok_or("expected two tuple members")?;
    Ok([member(a)?, member(b)?])
}

#[derive(Serialize)]
pub struct Guard {
    function: String,
    line: usize,
    comparison: String,
    basis_expression: String,
    row: [usize; 2],
    basis: [usize; 2],
    keep_when_equal: bool,
}

impl Guard {
    pub fn keep(&self, case: Case) -> bool {
        let equal = self
            .row
            .iter()
            .zip(self.basis)
            .all(|(&r, b)| case[r] == case[b]);
        equal == self.keep_when_equal
    }

    pub fn expression(&self) -> String {
        let [a, b] = self.row.map(|i| SYMBOLS[i]);
        let [c, d] = self.basis.map(|i| SYMBOLS[i]);
        let equal = format!("(and (= {a} {c}) (= {b} {d}))");
        if self.keep_when_equal {
            equal
        } else {
            format!("(not {equal})")
        }
    }
}

pub fn lower(source: &str, baseline: bool) -> Result<Guard> {
    let ast = tree::parse(source, tree::grammar("python")?)?;
    let root = ast.root();
    if tree::first_error(&root).is_some() {
        return Err("invalid Python syntax".into());
    }
    let function = |name: &str| {
        unique(
            root.children().filter(|n| {
                n.kind() == "function_definition" && field(n, "name").as_deref() == Some(name)
            }),
            name,
        )
    };
    let entry = function("signal_standing")?;
    let scope = if baseline {
        unique(
            tree::walk(&entry).into_iter().filter(|n| {
                n.kind() == "call" && n.text() == "_latest_and_not_run(_latest_basis(readings))"
            }),
            "baseline call chain",
        )?;
        function("_latest_basis")?
    } else {
        entry
    };
    let nodes = tree::walk(&scope);
    let assignment = |name: &str| -> Result<String> {
        let n = unique(
            nodes
                .iter()
                .filter(|n| n.kind() == "assignment" && field(n, "left").as_deref() == Some(name))
                .cloned(),
            name,
        )?;
        field(&n, "right").ok_or_else(|| "assignment lacks a value".into())
    };
    let newest = assignment("newest")?;
    let expected = if baseline {
        "readings[-1]"
    } else {
        "readings[-1] if readings else None"
    };
    if newest != expected {
        return Err("unsupported choice of newest row".into());
    }
    let basis_expression = assignment("basis")?;
    let basis_tuple = if baseline {
        &basis_expression
    } else {
        basis_expression
            .strip_suffix(" if newest else None")
            .ok_or("unsupported basis assignment")?
    };
    let basis = tuple(basis_tuple)?;
    if basis.iter().any(|&i| i < 2) {
        return Err("basis refers to unbound row".into());
    }

    let cmp = unique(
        nodes
            .iter()
            .filter(|n| n.kind() == "comparison_operator" && n.text().ends_with(" basis"))
            .cloned(),
        "basis comparison",
    )?;
    let comparison = cmp.text().into_owned();
    let (lhs, equals) = if let Some(lhs) = comparison.strip_suffix(" == basis") {
        (lhs, true)
    } else {
        (
            comparison
                .strip_suffix(" != basis")
                .ok_or("unsupported basis comparison")?,
            false,
        )
    };
    let row = tuple(lhs)?;
    if row.iter().zip(basis).any(|(&r, b)| r % 2 != b % 2) {
        return Err("comparison across distinct string families".into());
    }
    let parent = cmp.parent().ok_or("missing filter context")?;
    if baseline {
        let list = parent.parent().ok_or("missing comprehension")?;
        if parent.kind() != "if_clause"
            || list.text() != format!("[r for r in readings if {comparison}]")
        {
            return Err("unsupported inclusion filter".into());
        }
    } else {
        let loop_node = parent
            .parent()
            .and_then(|n| n.parent())
            .ok_or("missing loop")?;
        if parent.kind() != "if_statement"
            || field(&parent, "consequence").as_deref() != Some("continue")
            || field(&parent, "alternative").is_some()
            || loop_node.kind() != "for_statement"
            || field(&loop_node, "left").as_deref() != Some("r")
            || field(&loop_node, "right").as_deref() != Some("readings")
        {
            return Err("unsupported exclusion filter".into());
        }
    }
    Ok(Guard {
        function: field(&scope, "name").ok_or("function lacks name")?,
        line: tree::position(&cmp).0,
        comparison,
        basis_expression,
        row,
        basis,
        keep_when_equal: if baseline { equals } else { !equals },
    })
}
