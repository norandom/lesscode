//! Recognize a closed Python subset. Unknown statements invalidate the report.
use super::{check, Result};
use lesscode_core::adapters::driver::tree::{self, SgNode};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Site {
    pub file: String,
    pub line: usize,
    pub text: String,
}
fn site(file: &str, n: &SgNode<'_>) -> Site {
    Site {
        file: file.into(),
        line: tree::position(n).0,
        text: n.text().into_owned(),
    }
}
fn header(file: &str, n: &SgNode<'_>) -> Site {
    Site {
        file: file.into(),
        line: tree::position(n).0,
        text: n.text().lines().next().unwrap_or_default().into(),
    }
}
fn nodes<'a>(n: &SgNode<'a>) -> Vec<SgNode<'a>> {
    tree::named_children(n, &["comment".into()])
}
fn field<'a>(n: &SgNode<'a>, key: &str) -> Result<SgNode<'a>> {
    n.field(key)
        .ok_or_else(|| format!("expected {key} in {}", n.text()).into())
}
fn identifier(n: &SgNode<'_>) -> Result<String> {
    check(n.kind() == "identifier", "expected an unqualified name")?;
    Ok(n.text().into_owned())
}
fn string(n: &SgNode<'_>) -> Result<String> {
    let text = n.text();
    let first = text.chars().next().ok_or("empty expression")?;
    check(
        n.kind() == "string" && matches!(first, '\'' | '"'),
        "expected a plain string literal",
    )?;
    let s = text
        .strip_prefix(first)
        .and_then(|s| s.strip_suffix(first))
        .ok_or("invalid literal")?;
    check(
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
        "unsupported string contents",
    )?;
    Ok(s.into())
}

#[derive(Clone)]
pub enum Expr {
    Literal(String),
    Name(String),
    Slot(String, String, usize),
}
fn expr(n: &SgNode<'_>) -> Result<Expr> {
    match n.kind().as_ref() {
        "string" => Ok(Expr::Literal(string(n)?)),
        "identifier" => Ok(Expr::Name(identifier(n)?)),
        "subscript" => {
            let inner = field(n, "value")?;
            check(
                inner.kind() == "subscript" && nodes(n).len() == 2 && nodes(&inner).len() == 2,
                "expected a table entry and tuple slot",
            )?;
            let name = identifier(&field(&inner, "value")?)?;
            let key = string(&field(&inner, "subscript")?)?;
            let sub = field(n, "subscript")?;
            let slot = match sub.text().as_ref() {
                "0" => 0,
                "1" => 1,
                _ => return Err("expected tuple slot 0 or 1".into()),
            };
            Ok(Expr::Slot(name, key, slot))
        }
        _ => Err(format!("unsupported state expression {}", n.text()).into()),
    }
}
#[derive(Clone)]
pub struct Ref {
    pub expr: Expr,
    pub site: Site,
}
fn reference(file: &str, n: &SgNode<'_>) -> Result<Ref> {
    Ok(Ref {
        expr: expr(n)?,
        site: site(file, n),
    })
}
pub enum Binding {
    Scalar(Ref),
    Table(BTreeMap<String, [Ref; 2]>),
}
pub struct Method {
    pub name: String,
    pub declaration: Site,
    pub field: String,
    pub write: Site,
    pub to: Ref,
    pub guard: Option<(Site, Ref)>,
    pub refusal: Option<Site>,
    pub success: Option<Site>,
}
pub struct Class {
    pub declaration: Site,
    pub bases: Vec<String>,
    pub methods: Vec<Method>,
}
#[derive(Default)]
pub struct Module {
    pub imports: BTreeMap<String, (String, Site)>,
    pub bindings: BTreeMap<String, Binding>,
    pub classes: BTreeMap<String, Class>,
}
fn assign<'a>(statement: &SgNode<'a>) -> Result<(SgNode<'a>, SgNode<'a>)> {
    let parts = nodes(statement);
    let [a] = parts.as_slice() else {
        return Err("expected one assignment".into());
    };
    check(
        statement.kind() == "expression_statement"
            && a.kind() == "assignment"
            && nodes(a).len() == 2,
        "unsupported assignment",
    )?;
    Ok((field(a, "left")?, field(a, "right")?))
}
fn attr(n: &SgNode<'_>, receiver: &str) -> Result<String> {
    check(
        n.kind() == "attribute" && identifier(&field(n, "object")?)? == receiver,
        "expected receiver field",
    )?;
    let name = identifier(&field(n, "attribute")?)?;
    check(
        !name.starts_with("__"),
        "special or name-mangled receiver field",
    )?;
    Ok(name)
}
fn method(file: &str, n: &SgNode<'_>) -> Result<Method> {
    check(
        n.kind() == "function_definition"
            && n.text().starts_with("def ")
            && n.field("return_type").is_none(),
        "unsupported method",
    )?;
    let params = nodes(&field(n, "parameters")?);
    let [arg] = params.as_slice() else {
        return Err("expected receiver only".into());
    };
    let receiver = identifier(arg)?;
    let read_state = |node: &SgNode<'_>| -> Result<Ref> {
        let r = reference(file, node)?;
        check(
            !matches!(&r.expr, Expr::Name(n) | Expr::Slot(n, _, _) if n == &receiver),
            "receiver shadows the module binding",
        )?;
        Ok(r)
    };
    let name = identifier(&field(n, "name")?)?;
    let statements = nodes(&field(n, "body")?);
    let (write, guard, refusal, success) = match statements.as_slice() {
        [w] if name == "__init__" => (w, None, None, None),
        [g, w, s] if !name.starts_with("__") => {
            check(
                g.kind() == "if_statement"
                    && nodes(g).len() == 2
                    && s.kind() == "return_statement"
                    && s.text() == "return True",
                "unsupported handler effects or acceptance",
            )?;
            let c = field(g, "condition")?;
            let pair = nodes(&c);
            let [lhs, rhs] = pair.as_slice() else {
                return Err("expected binary comparison".into());
            };
            check(
                c.kind() == "comparison_operator" && c.children().any(|op| op.kind() == "!="),
                "expected inequality guard",
            )?;
            let denied = nodes(&field(g, "consequence")?);
            let [r] = denied.as_slice() else {
                return Err("expected single refusal".into());
            };
            check(
                r.kind() == "return_statement" && r.text() == "return False",
                "unsupported refusal",
            )?;
            (
                w,
                Some((attr(lhs, &receiver)?, site(file, &c), read_state(rhs)?)),
                Some(site(file, r)),
                Some(site(file, s)),
            )
        }
        _ => return Err(format!("unsupported method body {name}").into()),
    };
    let (lhs, rhs) = assign(write)?;
    let key = attr(&lhs, &receiver)?;
    if let Some((ref guarded, _, _)) = guard {
        check(guarded == &key, "guard and write use different fields")?;
    }
    Ok(Method {
        name,
        declaration: header(file, n),
        field: key,
        write: site(file, write),
        to: read_state(&rhs)?,
        guard: guard.map(|(_, s, r)| (s, r)),
        refusal,
        success,
    })
}
fn literal_or_prior(r: &Ref, module: &Module) -> Result<()> {
    match &r.expr {
        Expr::Literal(_) => Ok(()),
        Expr::Name(n) if matches!(module.bindings.get(n), Some(Binding::Scalar(_))) => Ok(()),
        _ => Err("table or constant depends on unsupported or later binding".into()),
    }
}
fn binding(file: &str, rhs: &SgNode<'_>, module: &Module) -> Result<Binding> {
    if rhs.kind() != "dictionary" {
        let value = reference(file, rhs)?;
        literal_or_prior(&value, module)?;
        return Ok(Binding::Scalar(value));
    }
    let mut entries = BTreeMap::new();
    for p in nodes(rhs) {
        let key = string(&field(&p, "key")?)?;
        let value = field(&p, "value")?;
        check(value.kind() == "tuple", "expected transition tuple")?;
        let tuple = nodes(&value);
        let [a, b] = tuple.as_slice() else {
            return Err("expected a state pair".into());
        };
        let pair = [reference(file, a)?, reference(file, b)?];
        for r in &pair {
            literal_or_prior(r, module)?;
        }
        check(entries.insert(key, pair).is_none(), "duplicate table key")?;
    }
    Ok(Binding::Table(entries))
}
pub fn module(file: &str, source: &str) -> Result<Module> {
    let ast = tree::parse(source, tree::grammar("python")?)?;
    let root = ast.root();
    check(tree::first_error(&root).is_none(), "invalid Python syntax")?;
    let mut m = Module::default();
    let mut defined = std::collections::BTreeSet::new();
    for n in nodes(&root) {
        let name = match n.kind().as_ref() {
            "import_from_statement" => {
                let s = n.text();
                let (source, symbol) = s
                    .strip_prefix("from .")
                    .and_then(|s| s.split_once(" import "))
                    .ok_or("expected sibling import")?;
                check(
                    [source, symbol].iter().all(|s| {
                        !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
                    }),
                    "unsupported import",
                )?;
                m.imports
                    .insert(symbol.into(), (source.into(), site(file, &n)));
                symbol.to_owned()
            }
            "expression_statement" => {
                let (lhs, rhs) = assign(&n)?;
                let key = identifier(&lhs)?;
                let b = binding(file, &rhs, &m)?;
                m.bindings.insert(key.clone(), b);
                key
            }
            "class_definition" => {
                let key = identifier(&field(&n, "name")?)?;
                let bases = n
                    .field("superclasses")
                    .map(|b| nodes(&b).iter().map(identifier).collect::<Result<Vec<_>>>())
                    .transpose()?
                    .unwrap_or_default();
                check(
                    bases.iter().all(|b| m.imports.contains_key(b)),
                    "base must be imported before class definition",
                )?;
                let body = field(&n, "body")?;
                let methods = nodes(&body)
                    .iter()
                    .map(|m| method(file, m))
                    .collect::<Result<_>>()?;
                let declaration = header(file, &n);
                m.classes.insert(
                    key.clone(),
                    Class {
                        declaration,
                        bases,
                        methods,
                    },
                );
                key
            }
            other => return Err(format!("unsupported module statement {other}").into()),
        };
        check(defined.insert(name), "duplicate module binding")?;
    }
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_extra_handler_effects_and_a_guard_on_another_receiver() {
        let source = "class M:\n    def act(obj):\n        if obj.mode != 'ready':\n            return False\n        obj.mode = 'done'\n        return True\n";
        assert!(module("m.py", source).is_ok());
        for changed in [
            source.replace("return True", "notify()\n        return True"),
            source.replace("if obj.mode", "if other.mode"),
            source.replace("if obj.mode", "if obj.other"),
            source.replace("'ready'", "obj['event'][0]"),
            source.replace("return False", "return True"),
        ] {
            assert!(module("m.py", &changed).is_err());
        }
    }

    #[test]
    fn rejects_table_rebinding_and_forward_references() {
        assert!(module("t.py", "A='ready'\nT={'event': (A, 'done')}\n").is_ok());
        assert!(module("t.py", "T={'event': (A, 'done')}\nA='ready'\n").is_err());
        assert!(module("t.py", "A='ready'\nA='other'\n").is_err());
    }
}
