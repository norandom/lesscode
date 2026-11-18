mod solver;
mod syntax;

use lesscode_core::receipt::sha256_hex;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fs,
    path::Path,
};
use syntax::{Binding, Expr, Module, Ref, Site};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Package = BTreeMap<String, Module>;
fn check(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.to_owned().into())
    }
}

#[derive(Serialize)]
struct StateRead {
    value: String,
    // Ordered from the use at the handler to the originating string literal.
    sites: Vec<Site>,
    table: Option<(String, String)>,
}
fn resolve(pkg: &Package, module: &str, r: &Ref) -> Result<StateRead> {
    let (name, slot) = match &r.expr {
        Expr::Literal(value) => {
            return Ok(StateRead {
                value: value.clone(),
                sites: vec![r.site.clone()],
                table: None,
            })
        }
        Expr::Name(name) => (name, None),
        Expr::Slot(name, key, i) => (name, Some((key, *i))),
    };
    let mut sites = vec![r.site.clone()];
    let owner = if let Some((owner, site)) = pkg[module].imports.get(name) {
        sites.push(site.clone());
        owner.as_str()
    } else {
        module
    };
    let (next, table) = match (pkg[owner].bindings.get(name), slot) {
        (Some(Binding::Scalar(v)), None) => (v, None),
        (Some(Binding::Table(entries)), Some((key, i))) => (
            &entries.get(key).ok_or("unresolved table key")?[i],
            Some((owner.to_string(), name.clone())),
        ),
        _ => return Err(format!("unresolved state value {owner}.{name}").into()),
    };
    let mut found = resolve(pkg, owner, next)?;
    sites.append(&mut found.sites);
    Ok(StateRead {
        value: found.value,
        sites,
        table,
    })
}
fn validate_imports(pkg: &Package) -> Result<()> {
    let mut loaded = BTreeSet::new();
    while loaded.len() < pkg.len() {
        let before = loaded.len();
        for (module, m) in pkg {
            for (symbol, (owner, _)) in &m.imports {
                let target = pkg
                    .get(owner)
                    .ok_or_else(|| format!("missing module {owner}"))?;
                check(
                    target.classes.contains_key(symbol) || target.bindings.contains_key(symbol),
                    "unresolved import",
                )?;
            }
            if m.imports.values().all(|(owner, _)| loaded.contains(owner)) {
                loaded.insert(module.clone());
            }
        }
        check(before < loaded.len(), "cyclic imports")?;
    }
    Ok(())
}
#[derive(Serialize)]
struct Event {
    owner: String,
    inherited_via: Site,
    class: Site,
    method: Site,
    guard: Site,
    from: StateRead,
    to: StateRead,
    write: Site,
    return_false: Site,
    return_true: Site,
}
#[derive(Serialize)]
struct Machine {
    sources: BTreeMap<String, String>,
    entity: String,
    field: String,
    declaration: Site,
    initial: StateRead,
    initial_write: Site,
    states: BTreeSet<String>,
    events: BTreeMap<String, Event>,
    rule_location: String,
}
impl Machine {
    fn step(&self, state: &str, event: &str) -> (String, bool) {
        let e = &self.events[event];
        if state == e.from.value {
            (e.to.value.clone(), true)
        } else {
            (state.into(), false)
        }
    }
}
fn discover(dir: &Path) -> Result<Machine> {
    let mut pkg = Package::new();
    let mut sources = BTreeMap::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("py") {
            continue;
        }
        check(
            entry.file_type()?.is_file(),
            "expected a regular source file",
        )?;
        let stem = p
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("invalid module filename")?
            .to_owned();
        let file = format!("{stem}.py");
        let source = fs::read_to_string(&p)?;
        let m = syntax::module(&file, &source).map_err(|e| format!("{file}: {e}"))?;
        sources.insert(file, sha256_hex(source.as_bytes()));
        pkg.insert(stem, m);
    }
    validate_imports(&pkg)?;
    let mut roots = pkg
        .iter()
        .flat_map(|(module, m)| m.classes.iter().map(move |(name, c)| (module, name, c)))
        .filter(|(_, _, c)| !c.bases.is_empty());
    let (module, name, root) = roots.next().ok_or("no composed entity")?;
    check(roots.next().is_none(), "ambiguous composed entity")?;
    let [init] = root.methods.as_slice() else {
        return Err("expected a single initializer on entity".into());
    };
    check(init.name == "__init__", "missing initializer")?;
    let initial = resolve(&pkg, module, &init.to)?;
    let mut events = BTreeMap::new();
    let mut states = BTreeSet::from([initial.value.clone()]);
    let mut bases = BTreeSet::new();
    let mut tables = BTreeSet::new();
    let mut rule_files = BTreeSet::new();
    for base in &root.bases {
        check(bases.insert(base), "duplicate base")?;
        let (owner, import) = &pkg[module].imports[base];
        let class = pkg[owner].classes.get(base).ok_or("base is not a class")?;
        check(
            class.bases.is_empty() && !class.methods.is_empty(),
            "unsupported inherited class",
        )?;
        for m in &class.methods {
            check(
                m.field == init.field && m.name != init.field,
                "field ownership mismatch",
            )?;
            let (guard, ref_from) = m.guard.as_ref().ok_or("unsupported inherited method")?;
            let from = resolve(&pkg, owner, ref_from)?;
            let to = resolve(&pkg, owner, &m.to)?;
            for s in [&from, &to] {
                check(
                    s.table.is_some() || s.sites.len() == 1,
                    "indirect rule outside a transition table",
                )?;
                states.insert(s.value.clone());
                tables.insert(s.table.clone());
            }
            rule_files.insert(guard.file.clone());
            let e = Event {
                owner: format!("{owner}.{base}"),
                inherited_via: import.clone(),
                class: class.declaration.clone(),
                method: m.declaration.clone(),
                guard: guard.clone(),
                from,
                to,
                write: m.write.clone(),
                return_false: m.refusal.clone().ok_or("missing refusal")?,
                return_true: m.success.clone().ok_or("missing success")?,
            };
            check(
                events.insert(m.name.clone(), e).is_none(),
                "conflicting event definitions",
            )?;
        }
    }
    check(
        pkg.values().map(|m| m.classes.len()).sum::<usize>() == bases.len() + 1,
        "unaccounted class",
    )?;
    let rule_location = match tables.into_iter().collect::<Vec<_>>().as_slice() {
        [Some((owner, table))] => format!("table {owner}.{table}"),
        [None] if rule_files.len() > 1 => "distributed literal guards".into(),
        _ => return Err("mixed or unsupported rule ownership".into()),
    };
    Ok(Machine {
        sources,
        entity: format!("{module}.{name}"),
        field: init.field.clone(),
        declaration: root.declaration.clone(),
        initial,
        initial_write: init.write.clone(),
        states,
        events,
        rule_location,
    })
}

fn execute() -> Result<bool> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [package] => {
            let m = discover(Path::new(package))?;
            println!("{}", serde_json::to_string_pretty(&m)?);
            Ok(false)
        }
        [left, right, z3, out] => solver::compare(
            &discover(Path::new(left))?,
            &discover(Path::new(right))?,
            Path::new(z3),
            Path::new(out),
        ),
        _ => Err(
            "usage: discover-machine PACKAGE | LEFT_PACKAGE RIGHT_PACKAGE Z3 NEW_EVIDENCE_DIR"
                .into(),
        ),
    }
}
fn main() {
    match execute() {
        Ok(false) => (),
        Ok(true) => std::process::exit(2),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1)
        }
    }
}
