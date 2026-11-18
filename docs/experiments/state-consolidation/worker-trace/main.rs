mod forecast;
mod instrument;
mod predict;

use lesscode_core::receipt::sha256_hex;
use serde_json::json;
use std::{
    env,
    error::Error,
    ffi::OsString,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const PHASES: [&str; 4] = ["warm", "unchanged", "changed", "fresh"];
fn ensure(condition: bool, error: &str) -> Result<()> {
    if !condition {
        return Err(error.to_owned().into());
    }
    Ok(())
}
fn save(path: &Path, data: &impl serde::Serialize) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(data)?;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    Ok(f.write_all(&bytes)?)
}
fn configured(var: &str) -> Result<PathBuf> {
    Ok(env::var_os(var)
        .ok_or_else(|| format!("missing {var}"))?
        .into())
}
fn option<'a>(args: &'a [OsString], flag: &str) -> Result<&'a Path> {
    let mut found = args.windows(2).filter(|a| a[0] == flag);
    let pair = found.next().ok_or_else(|| format!("missing {flag}"))?;
    ensure(found.next().is_none(), "duplicate option")?;
    Ok(Path::new(&pair[1]))
}

// Forward the original command and exit status. The prediction reader only
// accepts driver events. Child observations are written in a separate tree.
fn driver(args: &[OsString]) -> Result<i32> {
    let phase = option(args, "--out")?
        .file_stem()
        .and_then(|p| p.to_str())
        .ok_or("missing phase")?;
    ensure(PHASES.contains(&phase), "unexpected phase")?;
    let events = configured("LC_WORKER_LOGS")?.join(format!("{phase}.jsonl"));
    let children = configured("LC_WORKER_OBSERVATIONS")?.join(format!("{phase}.jsonl"));
    ensure(
        !events.exists() && !children.exists(),
        "phase already recorded",
    )?;
    let status = Command::new(configured("LC_WORKER_DRIVER")?)
        .args(args)
        .env("LC_WORKER_TRACE_FILE", events)
        .env("LC_WORKER_CHILD_FILE", children)
        .status()?;
    status.code().ok_or_else(|| "driver terminated".into())
}
fn child(args: &[OsString]) -> Result<i32> {
    let file = option(args, "--expect-from")?.canonicalize()?;
    let cwd = env::current_dir()?.canonicalize()?;
    let hash = sha256_hex(&fs::read(&file)?);
    let status = Command::new(configured("LC_WORKER_TOY")?)
        .args(args)
        .status()?;
    let code = status.code().ok_or("test command terminated")?;
    let mut sink = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(configured("LC_WORKER_CHILD_FILE")?)?;
    writeln!(
        sink,
        "{}",
        json!({"cwd":cwd,"file":file,"hash":hash,"exit":code})
    )?;
    Ok(code)
}
fn run() -> Result<i32> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let usage = "worker-trace overlay NATIVE NEW_OVERLAY | predict SITES DRIVER_EVENTS RELATIVE_TEST NEW_REPORT | forecast NATIVE EARLY_INPUT NEW_REQUEST";
    let (mode, rest) = args.split_first().ok_or(usage)?;
    match (mode.to_str(), rest) {
        (Some("overlay"), [native, out]) => instrument::overlay(native.as_ref(), out.as_ref())?,
        (Some("predict"), [sites, log, test, out]) => {
            predict::run(sites.as_ref(), log.as_ref(), test.as_ref(), out.as_ref())?
        }
        (Some("forecast"), [native, early, out]) => {
            forecast::request(native.as_ref(), early.as_ref(), out.as_ref())?
        }
        (Some("deconvolute"), _) => return driver(&args),
        (Some("--file"), _) => return child(&args),
        _ => return Err(usage.into()),
    }
    Ok(0)
}
fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1)
        }
    }
}
