//! Analysis of one source file: its sites, the state-write sites, the
//! function boundaries and the constructs the pack declares as gaps.

use super::pack::{has, Gap, Pack};
use super::sites::{sites_of, Site, OPERATOR_KINDS};
use super::tree::{ancestors, first_error, grammar, parse, walk, SgNode};
use super::Kind;
use crate::adapter_traits::CoverageGap;

pub struct Analysis {
    /// Every site of every kind, in order.
    pub sites: Vec<Site>,
    /// `label:line` of each state-write site.
    pub write_sites: Vec<String>,
    /// `label:start-end` of each function boundary.
    pub boundaries: Vec<String>,
    pub gaps: Vec<CoverageGap>,
}

fn is_gap(gap: &Gap, node: &SgNode<'_>) -> bool {
    let kind = node.kind();
    if has(&gap.nodes, &kind) {
        return true;
    }
    let text = node.text();
    gap.nodes_with_prefix
        .get(kind.as_ref())
        .is_some_and(|prefixes| prefixes.iter().any(|p| text.starts_with(p.as_str())))
}

/// Analyse `text`; `label` is how the file is named in the results. A text
/// that does not parse is an error that says where parsing stopped.
pub fn analyse(pack: &Pack, label: &str, text: &str) -> Result<Analysis, String> {
    let root = parse(text, grammar(&pack.language)?)?;
    if let Some((line, column)) = first_error(&root.root()) {
        return Err(format!(
            "{label}:{line}:{column}: the source does not parse"
        ));
    }
    let mut kinds = OPERATOR_KINDS.to_vec();
    kinds.push(Kind::PureShellBoundary);
    let sites = sites_of(pack, label, text, &kinds)?;
    let of_kind = |kind: Kind| sites.iter().filter(move |s| s.kind == kind);
    let write_sites = of_kind(Kind::StateWrite)
        .map(|s| format!("{label}:{}", s.start_line))
        .collect();
    let boundaries = of_kind(Kind::PureShellBoundary)
        .map(|s| format!("{label}:{}-{}", s.start_line, s.end_line))
        .collect();
    let mut gaps = Vec::new();
    for node in walk(&root.root()) {
        let hidden = |n: &SgNode<'_>| pack.is_hidden(&n.kind());
        if hidden(&node) || ancestors(&node).iter().any(hidden) {
            continue;
        }
        for gap in pack.gaps.iter().filter(|gap| is_gap(gap, &node)) {
            gaps.push(CoverageGap {
                reason: gap.reason.clone(),
                location: Some(format!("{label}:{}", node.start_pos().line() + 1)),
            });
        }
    }
    gaps.dedup();
    Ok(Analysis {
        sites,
        write_sites,
        boundaries,
        gaps,
    })
}
