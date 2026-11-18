use super::{ensure, save, Result, PHASES};
use lesscode_core::receipt::sha256_hex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Deserialize)]
struct Site {
    id: String,
    operation: String,
}
#[derive(Deserialize)]
struct Inventory {
    sites: Vec<Site>,
}
#[derive(Deserialize)]
struct Event {
    sequence: usize,
    site: String,
    op: String,
    data: Value,
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key]
        .as_str()
        .ok_or_else(|| format!("missing {key}").into())
}
fn hash(v: &Value, key: &str) -> Result<String> {
    let s = text(v, key)?;
    ensure(
        s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid digest",
    )?;
    Ok(s.into())
}
fn path(v: &Value, key: &str) -> Result<PathBuf> {
    let p = PathBuf::from(text(v, key)?);
    ensure(p.is_absolute(), "expected absolute trace path")?;
    Ok(p)
}

#[derive(Default)]
struct Phase {
    project: Option<(PathBuf, String)>,
    read_site: String,
    digest: Option<String>,
    identity: Option<String>,
    keys: BTreeSet<String>,
    lookups: BTreeMap<String, bool>,
    workspace: Option<String>,
    pending_mutant: Option<PathBuf>,
    commands: Vec<Value>,
    sites: BTreeSet<String>,
}
#[derive(Serialize)]
struct Copied {
    phase: String,
    sequence: usize,
    read_site: String,
    copy_site: String,
    source: PathBuf,
    hash: String,
}
impl Phase {
    fn project(&self) -> Result<&(PathBuf, String)> {
        self.project
            .as_ref()
            .ok_or_else(|| "no project test read".into())
    }
    fn observe(
        &mut self,
        e: Event,
        relative: &Path,
        inventory: &BTreeMap<String, String>,
        copies: &mut BTreeMap<PathBuf, Copied>,
        phase: &str,
    ) -> Result<()> {
        ensure(
            inventory.get(&e.site) == Some(&e.op),
            "event does not match source site",
        )?;
        self.sites.insert(e.site.clone());
        let v = &e.data;
        match e.op.as_str() {
            "read_test" => {
                self.read_site = e.site;
                let p = path(v, "path")?;
                ensure(
                    p.ends_with(relative) && self.project.replace((p, hash(v, "hash")?)).is_none(),
                    "expected exactly one project test",
                )?;
            }
            "test_digest" => {
                let relative = relative
                    .to_str()
                    .ok_or("invalid test path")?
                    .replace('\\', "/");
                let expected = sha256_hex(format!("{relative}\t{}", self.project()?.1).as_bytes());
                ensure(
                    hash(v, "hash")? == expected && self.digest.replace(expected).is_none(),
                    "aggregate does not match observed path and bytes",
                )?;
            }
            "identity" => {
                ensure(
                    self.digest.is_some() && self.identity.replace(hash(v, "hash")?).is_none(),
                    "unexpected configuration identity",
                )?;
            }
            "key" => {
                ensure(
                    self.identity.as_deref() == Some(text(v, "identity")?)
                        && self.keys.insert(hash(v, "hash")?),
                    "outcome key has unknown inputs or is duplicated",
                )?;
            }
            "lookup" => {
                let key = hash(v, "hash")?;
                let hit = v["hit"].as_bool().ok_or("missing lookup verdict")?;
                ensure(
                    self.keys.contains(&key) && self.lookups.insert(key, hit).is_none(),
                    "cache lookup does not match planned key",
                )?;
            }
            "open" | "create" => {
                ensure(
                    v["ok"] == true && self.workspace.replace(e.op).is_none(),
                    "expected one successful worker acquisition",
                )?;
            }
            "copy" => {
                let from = path(v, "from")?;
                let to = path(v, "to")?;
                if to.ends_with(relative) {
                    let (project, bytes) = self.project()?;
                    ensure(
                        &from == project,
                        "untracked source copied over watched test",
                    )?;
                    copies.insert(
                        to,
                        Copied {
                            phase: phase.into(),
                            sequence: e.sequence,
                            read_site: self.read_site.clone(),
                            copy_site: e.site,
                            source: from,
                            hash: bytes.clone(),
                        },
                    );
                }
            }
            "mutant_tree" => {
                ensure(
                    self.pending_mutant.replace(path(v, "cwd")?).is_none(),
                    "overlapping mutant runs",
                )?;
            }
            "command_dir" => {
                let cwd = path(v, "cwd")?;
                let argv = v["argv"].as_array().ok_or("missing command arguments")?;
                let mut expectations = argv.windows(2).filter(|p| p[0] == "--expect-from");
                let arg = expectations
                    .next()
                    .and_then(|p| p[1].as_str())
                    .ok_or("unexpected test command")?;
                ensure(
                    Path::new(arg) == relative && expectations.next().is_none(),
                    "test argument does not match watched path",
                )?;
                let file = cwd.join(relative);
                let expected = copies
                    .get(&file)
                    .ok_or("no observed copy into child-visible test path")?;
                let mutant = self.pending_mutant.take();
                ensure(
                    self.workspace.is_some() && mutant.as_ref().is_none_or(|p| *p == cwd),
                    "missing workspace or mutant cwd mismatch",
                )?;
                self.commands.push(json!({"cwd":cwd,"file":file,"predicted_hash":expected.hash,"last_copy":expected,"current":expected.hash==self.project()?.1,"mutant":mutant.is_some(),"site":e.site}));
            }
            _ => return Err("unsupported event".into()),
        }
        Ok(())
    }
    fn finish(&self) -> Result<Value> {
        ensure(
            self.digest.is_some()
                && self.identity.is_some()
                && !self.keys.is_empty()
                && self.lookups.len() == self.keys.len(),
            "incomplete identity or lookup trace",
        )?;
        let misses = self.lookups.values().filter(|hit| !**hit).count();
        let mutants = self.commands.iter().filter(|r| r["mutant"] == true).count();
        ensure(
            self.workspace.is_some()
                && self.pending_mutant.is_none()
                && self.commands.len() == mutants + 2
                && mutants == misses,
            "incomplete worker execution for one worker and one test step",
        )?;
        Ok(
            json!({"project":self.project()?.0,"project_hash":self.project()?.1,"test_identity":self.digest,"configuration_with_tests":self.identity,
            "worker":self.workspace,"outcome_key_hits":self.lookups,"cache_hits":self.keys.len()-misses,"mutant_executions":mutants,
            "commands":self.commands,"executed_sites":self.sites}),
        )
    }
}
pub fn run(sites: &Path, log: &Path, relative: &Path, out: &Path) -> Result<()> {
    ensure(
        relative
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
            && relative.file_name().is_some(),
        "expected a relative test file",
    )?;
    let inv: Inventory = serde_json::from_slice(&fs::read(sites)?)?;
    let mut index = BTreeMap::new();
    for s in inv.sites {
        ensure(
            index.insert(s.id, s.operation).is_none(),
            "duplicate source site",
        )?;
    }
    let mut copies = BTreeMap::new();
    let mut result = BTreeMap::new();
    for name in PHASES {
        let mut phase = Phase::default();
        for (i, line) in fs::read_to_string(log.join(format!("{name}.jsonl")))?
            .lines()
            .enumerate()
        {
            let event: Event = serde_json::from_str(line)?;
            ensure(event.sequence == i + 1, "missing or reordered driver event")?;
            phase.observe(event, relative, &index, &mut copies, name)?;
        }
        result.insert(name, phase.finish()?);
    }
    save(out, &result)
}
