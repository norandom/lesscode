//! The receipt of the budgets check (`rust-syntax-units-v2`).

use std::path::Path;

use crate::budgets::FileReport;
use crate::receipt::J;

fn n(v: usize) -> J {
    J::int(v as i64)
}
/// The ceiling in the unit that applies, null in the other.
fn ceiling(applies: bool, value: usize) -> J {
    if applies {
        n(value)
    } else {
        J::Null
    }
}
fn file_j(r: &FileReport) -> J {
    J::obj(vec![
        ("path", J::str(r.path.clone())),
        ("present", J::Bool(r.present)),
        ("total_lines", n(r.total_lines)),
        ("production_lines", n(r.production_lines)),
        ("test_lines", n(r.test_lines)),
        ("logical_units", n(r.logical_units)),
        ("longest_fn_lines", n(r.longest_fn_lines)),
        ("longest_fn_name", J::str(r.longest_fn_name.clone())),
        ("production_tokens", n(r.production_tokens)),
        ("test_tokens", n(r.test_tokens)),
        ("longest_fn_tokens", n(r.longest_fn_tokens)),
        (
            "longest_fn_tokens_name",
            J::str(r.longest_fn_tokens_name.clone()),
        ),
        ("max_tokens", ceiling(!r.in_lines[0], r.limit[0])),
        ("max_physical_lines", ceiling(r.in_lines[0], r.limit[0])),
        ("max_logical_units", n(r.limit[1])),
        ("max_fn_tokens", ceiling(!r.in_lines[1], r.limit[2])),
        ("max_fn_lines", ceiling(r.in_lines[1], r.limit[2])),
        ("owner", J::str(r.owner.clone())),
        ("allocation", J::str(r.allocation.clone())),
        (
            "disposition",
            r.disposition.clone().map(J::str).unwrap_or(J::Null),
        ),
        ("status", J::str(r.status())),
    ])
}

pub(crate) fn receipt(
    root: &Path,
    config: &Path,
    reports: &[FileReport],
    unlisted: &[String],
) -> J {
    let s = |status: &str| n(reports.iter().filter(|r| r.status() == status).count());
    J::obj(vec![
        ("receipt", J::str("budgets")),
        ("schema", J::str("rust-syntax-units-v2")),
        ("root", J::str(root.to_string_lossy().replace('\\', "/"))),
        (
            "config",
            J::str(config.to_string_lossy().replace('\\', "/")),
        ),
        ("files", J::Arr(reports.iter().map(file_j).collect())),
        (
            "unlisted",
            J::Arr(unlisted.iter().map(|p| J::str(p.clone())).collect()),
        ),
        (
            "summary",
            J::obj(vec![
                ("total", n(reports.len() + unlisted.len())),
                ("within_budget", s("within_budget")),
                ("disposed", s("disposed")),
                ("exceeded", s("exceeded")),
                ("unlisted", n(unlisted.len())),
                ("missing", s("missing")),
            ]),
        ),
    ])
}
