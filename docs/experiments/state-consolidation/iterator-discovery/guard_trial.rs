use super::{classes, defaults, field, public, save, sha256_hex, tree, Result, SgNode};
use lesscode_core::adapters::driver::process;
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path, time::Duration};

const PARSER: &str = "bfe10911dac9f3c54ae1b21a4848ede25944f87a09763f3c4fc50d7fbabf1e63";
const Z3: &str = "2bffeee10ba02a510d07be9c3fdf9c22f10ccc9724dc84201c5a40196f1a1e8f";
const BOUND: usize = 4;
const EVENTS: [&str; 3] = ["hasNext", "close", "next"];
const RESULTS: [&str; 6] = [
    "false",
    "true",
    "closed",
    "oak",
    "elm",
    "throws:NoSuchElementException",
];

fn syntax<'a>(n: &SgNode<'a>) -> Vec<SgNode<'a>> {
    n.children()
        .filter(|c| c.is_named() && !matches!(c.kind().as_ref(), "line_comment" | "block_comment"))
        .collect()
}

fn condition(n: &SgNode<'_>) -> Result<String> {
    match n.kind().as_ref() {
        "parenthesized_expression" => {
            let nodes = syntax(n);
            let [inner] = nodes.as_slice() else {
                return Err("invalid parentheses".into());
            };
            condition(inner)
        }
        "method_invocation"
            if n.field("object").is_none()
                && field(n, "name") == "isClosed"
                && defaults::empty_list(n, "arguments")
                && n.field("type_arguments").is_none() =>
        {
            Ok("closed".into())
        }
        "binary_expression" => {
            let left = n.field("left").ok_or("missing left operand")?;
            let right = n.field("right").ok_or("missing right operand")?;
            match field(n, "operator").as_str() {
                "&&" => Ok(format!(
                    "(and {} {})",
                    condition(&left)?,
                    condition(&right)?
                )),
                "==" if left.kind() == "identifier"
                    && left.text() == "current"
                    && right.kind() == "null_literal" =>
                {
                    Ok("(not pending)".into())
                }
                _ => Err("unsupported guard operator or operands".into()),
            }
        }
        _ => Err("unsupported guard expression".into()),
    }
}

struct Guard {
    smt: String,
    evidence: Value,
}

fn extract(path: &Path) -> Result<Guard> {
    let source = fs::read_to_string(path)?.replace("\r\n", "\n");
    let ast = tree::parse(&source, tree::grammar("java")?)?;
    let root = ast.root();
    if tree::first_error(&root).is_some() {
        return Err("invalid Java syntax".into());
    }
    let methods: Vec<_> = tree::walk(&root)
        .into_iter()
        .filter(|n| {
            n.kind() == "method_declaration"
                && field(n, "name") == "next"
                && public(n)
                && defaults::empty_list(n, "parameters")
                && classes(n) == ["CSVRecordIterator", "CSVParser"]
        })
        .collect();
    let [method] = methods.as_slice() else {
        return Err("expected one pinned iterator next method".into());
    };
    let body = method.field("body").ok_or("missing method body")?;
    let stmts = syntax(&body);
    let guard = stmts
        .first()
        .filter(|s| s.kind() == "if_statement")
        .ok_or("missing leading guard")?;
    let branch = guard.field("consequence").ok_or("missing consequence")?;
    let stmts = syntax(&branch);
    let [throw] = stmts.as_slice() else {
        return Err("guard must only throw".into());
    };
    if branch.kind() != "block"
        || throw.kind() != "throw_statement"
        || guard.field("alternative").is_some()
        || throw.text() != r#"throw new NoSuchElementException("CSVParser has been closed");"#
    {
        return Err("unsupported guard consequence".into());
    }
    let expr = guard.field("condition").ok_or("missing condition")?;
    let smt = condition(&expr)?;
    // The rest of this real parser supplies the fixed semantics used in the
    // experiment. Refuse to extrapolate them to a different implementation.
    let mut masked = source.clone();
    masked.replace_range(expr.range(), "(isClosed())");
    if sha256_hex(masked.as_bytes()) != PARSER {
        return Err("source outside the modeled condition changed".into());
    }
    Ok(Guard {
        evidence: json!({"file":path,"sha256_lf":sha256_hex(source.as_bytes()),"line":tree::position(&expr).0,
        "condition":expr.text(),"throw":throw.text(),"predicate":smt}),
        smt,
    })
}

// The surroundings are the R144 model. The two predicates are the only
// transition expressions lowered from the Java syntax in this experiment.
fn query(a: &Guard, b: &Guard, length: usize) -> String {
    let mut q = String::from("(set-logic QF_LIA)\n");
    for i in 0..length {
        q += &format!("(declare-const e{i} Int)\n(assert (and (<= 0 e{i}) (< e{i} 3)))\n");
    }
    let mut differences = vec!["false".to_owned()]; // identical initial observations
    for (arm, guard) in [("a", a), ("b", b)] {
        q += &format!(
            "(define-fun {arm}guard ((closed Bool) (pending Bool)) Bool {})\n",
            guard.smt
        );
        q += &format!("(define-fun {arm}c0 () Bool false)\n(define-fun {arm}p0 () Bool false)\n(define-fun {arm}n0 () Int 0)\n");
        for i in 0..length {
            let (c, p, n) = (
                format!("{arm}c{i}"),
                format!("{arm}p{i}"),
                format!("{arm}n{i}"),
            );
            let (deny, fetch, ret) = (
                format!("{arm}deny{i}"),
                format!("{arm}fetch{i}"),
                format!("{arm}r{i}"),
            );
            let j = i + 1;
            // e=0 looks ahead, e=1 closes, e=2 consumes. n counts successful reads.
            q += &format!("(define-fun {deny} () Bool ({arm}guard {c} {p}))\n");
            q += &format!("(define-fun {fetch} () Bool (and (not {p}) (< {n} 2) (or (and (= e{i} 0) (not {c})) (and (= e{i} 2) (not {deny})))))\n");
            q += &format!("(define-fun {arm}n{j} () Int (+ {n} (ite {fetch} 1 0)))\n");
            q += &format!("(define-fun {arm}c{j} () Bool (or {c} (= e{i} 1)))\n");
            q += &format!("(define-fun {arm}p{j} () Bool (ite (= e{i} 0) (or {p} {fetch}) (and {p} (or (= e{i} 1) {deny}))))\n");
            q += &format!("(define-fun {ret} () Int (ite (= e{i} 1) 2 (ite (= e{i} 0) (ite (and (not {c}) (or {p} {fetch})) 1 0) (ite (or {deny} (not (or {p} {fetch}))) 5 (+ 2 {arm}n{j})))))\n");
        }
    }
    for i in 0..length {
        differences.push(format!("(distinct ar{i} br{i})"));
        differences.push(format!("(distinct an{} bn{})", i + 1, i + 1));
    }
    format!("{q}(assert (or {}))\n(check-sat)\n", differences.join(" "))
}

fn solve(z3: &Path, out: &Path, name: &str, query: &str) -> Result<String> {
    let filename = format!("{name}.smt2");
    fs::write(out.join(&filename), query)?;
    let args = [z3.to_string_lossy().into_owned(), "-smt2".into(), filename];
    let run = process::run(&args, out, &BTreeMap::new(), Duration::from_secs(10));
    fs::write(out.join(format!("{name}.txt")), &run.output)?;
    if !run.passed() {
        return Err(format!("solver failed on {name}: {}", run.output).into());
    }
    Ok(run.output.trim().into())
}

fn bindings(answer: &str) -> Result<BTreeMap<String, usize>> {
    let (status, rows) = answer.split_once('\n').ok_or("missing model")?;
    if status.trim() != "sat" {
        return Err("no witness".into());
    }
    let mut rows = rows
        .trim()
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or("invalid model")?
        .trim();
    let mut values = BTreeMap::new();
    while !rows.is_empty() {
        let (pair, next) = rows
            .strip_prefix('(')
            .and_then(|s| s.split_once(')'))
            .ok_or("invalid pair")?;
        let fields: Vec<_> = pair.split_whitespace().collect();
        let [key, val] = fields.as_slice() else {
            return Err("invalid binding".into());
        };
        if values.insert(key.to_string(), val.parse()?).is_some() {
            return Err("duplicate binding".into());
        }
        rows = next.trim();
    }
    Ok(values)
}

fn witness(z3: &Path, out: &Path, q: &str, n: usize) -> Result<Value> {
    let mut keys = Vec::new();
    for i in 0..n {
        keys.extend([
            format!("e{i}"),
            format!("ar{i}"),
            format!("br{i}"),
            format!("an{}", i + 1),
            format!("bn{}", i + 1),
        ]);
    }
    let request = format!("{q}(get-value ({}))\n", keys.join(" "));
    let values = bindings(&solve(z3, out, "witness", &request)?)?;
    if values.len() != keys.len() || !keys.iter().all(|k| values.contains_key(k)) {
        return Err("wrong witness fields".into());
    }
    let mut trace = Vec::new();
    for i in 0..n {
        let event = EVENTS
            .get(values[&format!("e{i}")])
            .ok_or("unknown event")?;
        let observation = |arm: &str| -> Result<Value> {
            let result = RESULTS
                .get(values[&format!("{arm}r{i}")])
                .ok_or("unknown return")?;
            let count = values[&format!("{arm}n{}", i + 1)];
            if count > 2 {
                return Err("out of range record count".into());
            }
            Ok(json!({"value":result,"count":count}))
        };
        trace.push(json!({"event":event,"left":observation("a")?,"right":observation("b")?}));
    }
    if !trace.iter().any(|s| s["left"] != s["right"]) {
        return Err("witness lacks a public difference".into());
    }
    Ok(json!(trace))
}

pub(super) fn compare(left: &Path, right: &Path, z3: &Path, out: &Path) -> Result<()> {
    let (a, b) = (extract(left)?, extract(right)?);
    let z3 = z3.canonicalize()?;
    if sha256_hex(&fs::read(&z3)?) != Z3 {
        return Err("unexpected Z3 binary".into());
    }
    fs::create_dir(out)?;
    let out = out.canonicalize()?;
    let mut lengths = Vec::new();
    let mut first = None;
    for n in 0..=BOUND {
        let q = query(&a, &b, n);
        let status = solve(&z3, &out, &format!("length-{n}"), &q)?;
        match status.as_str() {
            "sat" if first.is_none() => first = Some(witness(&z3, &out, &q, n)?),
            "sat" | "unsat" => (),
            _ => return Err(format!("inconclusive solver response: {status}").into()),
        }
        lengths.push(json!({"length":n,"status":status}));
    }
    save(
        &out.join("result.json"),
        &json!({"left":a.evidence,"right":b.evidence,"bound":BOUND,
        "events":EVENTS,"z3_sha256":Z3,"lengths":lengths,"witness":first}),
    )
}
