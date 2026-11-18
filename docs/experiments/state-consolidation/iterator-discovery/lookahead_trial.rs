//! Lower the whole `hasNext()` look-ahead and the `next()` closure guard of
//! the pinned Commons CSV iterator, then compare two source versions with Z3.
use super::guard_trial::{condition, depths, method, next_guard, parse_java, syntax, PARSER};
use super::{sha256_hex, tree, Result, SgNode};
use serde_json::{json, Value};
use std::path::Path;

/// The original body of `hasNext()`, restored when masking a lowered copy.
const HAS_NEXT: &str = "{
            if (isClosed()) {
                return false;
            }
            if (current == null) {
                current = getNextRecord();
            }
            return current != null;
        }";
const FILL: &str = "current = getNextRecord();";

/// Lowered predicates over `closed` and `pending`: the closure guard and the
/// fill condition of `hasNext()`, its return value, and the guard of `next()`.
struct Arm {
    closure: String,
    fill: String,
    returns: String,
    next: String,
    evidence: Value,
}

fn single<'a>(block: &SgNode<'a>) -> Option<SgNode<'a>> {
    match syntax(block).as_slice() {
        [statement] if block.kind() == "block" => Some(statement.clone()),
        _ => None,
    }
}

fn extract(path: &Path) -> Result<Arm> {
    let (source, ast) = parse_java(path)?;
    let root = ast.root();
    let has_next = method(&root, "hasNext")?;
    let body = has_next.field("body").ok_or("missing method body")?;
    let statements = syntax(&body);
    let [guard, fill, ret] = statements.as_slice() else {
        return Err("unsupported hasNext body".into());
    };
    if guard.kind() != "if_statement" || guard.field("alternative").is_some() {
        return Err("unsupported hasNext body".into());
    }
    let closed = guard.field("consequence").and_then(|b| single(&b));
    if closed.map(|s| s.text().into_owned()).as_deref() != Some("return false;") {
        return Err("unsupported closure return".into());
    }
    let closure = condition(&guard.field("condition").ok_or("missing condition")?)?;
    let (fill_test, assign) = match fill.kind().as_ref() {
        "if_statement" if fill.field("alternative").is_none() => {
            let test = fill.field("condition").ok_or("missing condition")?;
            let assign = fill.field("consequence").and_then(|b| single(&b));
            (condition(&test)?, assign.ok_or("unsupported fill")?)
        }
        "expression_statement" => ("true".to_owned(), fill.clone()),
        _ => return Err("unsupported hasNext body".into()),
    };
    if assign.kind() != "expression_statement" || assign.text() != FILL {
        return Err("unsupported fill".into());
    }
    let values = syntax(ret);
    let [value] = values.as_slice() else {
        return Err("unsupported return".into());
    };
    if ret.kind() != "return_statement" {
        return Err("unsupported hasNext body".into());
    }
    let returns = condition(value).map_err(|e| format!("unsupported return: {e}"))?;
    let (next_test, next, _) = next_guard(&root)?;
    // Everything outside the two lowered regions is the R144 model's source.
    let (lowered, guarded) = (body.range(), next_test.range());
    if guarded.start < lowered.end {
        return Err("unexpected method order".into());
    }
    let mut masked = source.clone();
    masked.replace_range(guarded, "(isClosed())");
    masked.replace_range(lowered, HAS_NEXT);
    if sha256_hex(masked.as_bytes()) != PARSER {
        return Err("source outside the modeled regions changed".into());
    }
    let evidence = json!({"file":path,"sha256_lf":sha256_hex(source.as_bytes()),
        "has_next_line":tree::position(&has_next).0,"has_next_body":body.text(),
        "next_guard_line":tree::position(&next_test).0,"next_guard":next_test.text(),
        "predicates":{"closure":closure,"fill":fill_test,"returns":returns,"next":next}});
    Ok(Arm {
        closure,
        fill: fill_test,
        returns,
        next,
        evidence,
    })
}

// e=0 is hasNext, e=1 close, e=2 next. n counts successful reads of the
// two-record input; the pending record, when present, is always the last read.
// Return codes index guard_trial::RESULTS.
fn query(a: &Arm, b: &Arm, length: usize) -> String {
    let mut q = String::from("(set-logic QF_LIA)\n");
    for i in 0..length {
        q += &format!("(declare-const e{i} Int)\n(assert (and (<= 0 e{i}) (< e{i} 3)))\n");
    }
    for (arm, m) in [("a", a), ("b", b)] {
        for (name, body) in [
            ("hg", &m.closure),
            ("hf", &m.fill),
            ("hr", &m.returns),
            ("ng", &m.next),
        ] {
            q += &format!("(define-fun {arm}{name} ((closed Bool) (pending Bool)) Bool {body})\n");
        }
        q += &format!("(define-fun {arm}c0 () Bool false)\n(define-fun {arm}p0 () Bool false)\n(define-fun {arm}n0 () Int 0)\n");
        for i in 0..length {
            let j = i + 1;
            let (c, p, n, e) = (
                format!("{arm}c{i}"),
                format!("{arm}p{i}"),
                format!("{arm}n{i}"),
                format!("e{i}"),
            );
            let (hdeny, fill, ndeny, take, read) = (
                format!("{arm}hdeny{i}"),
                format!("{arm}fill{i}"),
                format!("{arm}ndeny{i}"),
                format!("{arm}take{i}"),
                format!("{arm}read{i}"),
            );
            q += &format!("(define-fun {hdeny} () Bool ({arm}hg {c} {p}))\n");
            q += &format!(
                "(define-fun {fill} () Bool (and (= {e} 0) (not {hdeny}) ({arm}hf {c} {p})))\n"
            );
            q += &format!("(define-fun {ndeny} () Bool ({arm}ng {c} {p}))\n");
            q += &format!("(define-fun {take} () Bool (and (= {e} 2) (not {ndeny}) (not {p})))\n");
            q += &format!("(define-fun {read} () Bool (and (< {n} 2) (or {fill} {take})))\n");
            q += &format!("(define-fun {arm}n{j} () Int (+ {n} (ite {read} 1 0)))\n");
            q += &format!("(define-fun {arm}c{j} () Bool (or {c} (= {e} 1)))\n");
            q += &format!("(define-fun {arm}p{j} () Bool (ite {fill} {read} (ite (= {e} 2) (and {p} {ndeny}) {p})))\n");
            q += &format!("(define-fun {arm}r{i} () Int (ite (= {e} 1) 2 (ite (= {e} 0) (ite {hdeny} 0 (ite ({arm}hr {c} {arm}p{j}) 1 0)) (ite {ndeny} 5 (ite {p} (+ 2 {n}) (ite {read} (+ 2 {arm}n{j}) 5))))))\n");
        }
    }
    let mut differences = vec!["false".to_owned()]; // identical initial observations
    for i in 0..length {
        differences.push(format!("(distinct ar{i} br{i})"));
        differences.push(format!("(distinct an{} bn{})", i + 1, i + 1));
    }
    format!("{q}(assert (or {}))\n(check-sat)\n", differences.join(" "))
}

pub(super) fn compare(left: &Path, right: &Path, z3: &Path, out: &Path) -> Result<()> {
    let (a, b) = (extract(left)?, extract(right)?);
    depths(z3, out, a.evidence.clone(), b.evidence.clone(), |n| {
        query(&a, &b, n)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lower(expr: &str) -> Result<String> {
        let java = format!("class A {{ boolean f() {{ return {expr}; }} }}");
        let ast = tree::parse(&java, tree::grammar("java")?)?;
        let root = ast.root();
        let ret = tree::walk(&root)
            .into_iter()
            .find(|n| n.kind() == "return_statement")
            .ok_or("no return")?;
        let value = syntax(&ret).into_iter().next().ok_or("no value")?;
        condition(&value)
    }

    #[test]
    fn comparisons_with_null_lower_in_either_order() {
        assert_eq!(lower("current == null").unwrap(), "(not pending)");
        assert_eq!(lower("(null == current)").unwrap(), "(not pending)");
        assert_eq!(lower("current != null").unwrap(), "pending");
        assert_eq!(lower("null != current").unwrap(), "pending");
        assert_eq!(
            lower("isClosed() && current == null").unwrap(),
            "(and closed (not pending))"
        );
        assert!(lower("current != other").is_err());
        assert!(lower("current != null || isClosed()").is_err());
        assert!(lower("null == null").is_err());
    }
}
