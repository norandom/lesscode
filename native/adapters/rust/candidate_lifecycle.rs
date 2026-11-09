//! Deconvoluted process lifecycle, execution supervisor, and pytest group orchestrator.
//!
//! Compact candidate replacement for `baseline_lifecycle.rs` (R52 prototype gate).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct LifecycleError(pub String);
impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for LifecycleError {}

/// Run cmd with deadline-bound tree kill semantics.
pub fn call(
    program: &str,
    args: &[&str],
    cwd: &Path,
    envs: &[(String, String)],
    deadline: Option<Instant>,
) -> i32 {
    let mut cmd = Command::new(program);
    cmd.args(args).current_dir(cwd).envs(envs.iter().cloned());
    let Ok(mut child) = cmd.spawn() else { return 127 };
    let poll = Duration::from_millis(50);
    loop {
        if let Ok(Some(st)) = child.try_wait() {
            return st.code().unwrap_or(-1);
        }
        if let Some(dl) = deadline {
            if Instant::now() >= dl {
                let _ = Command::new("taskkill").args(["/F", "/T", "/PID", &child.id().to_string()]).output();
                println!("proto: deadline exceeded (infrastructure, not a verdict)");
                return 124;
            }
            std::thread::sleep(poll.min(dl.saturating_duration_since(Instant::now())));
        } else {
            std::thread::sleep(poll);
        }
    }
}

/// Fail closed when module is not imported from `root`.
pub fn probe(
    venv: &Path,
    root: &Path,
    module: &str,
    envs: &[(String, String)],
) -> Result<(), LifecycleError> {
    let code = format!("import {module} as m; print(m.__file__)");
    let out = Command::new(venv).arg("-c").arg(&code).envs(envs.iter().cloned()).output()
        .map_err(|e| LifecycleError(format!("probe failed: {e}")))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let imported = PathBuf::from(if stdout.is_empty() { "/".into() } else { stdout.clone() });
    let is_prefix = match (imported.canonicalize(), root.canonicalize()) {
        (Ok(c), Ok(r)) => c.starts_with(r),
        _ => imported.starts_with(root),
    };
    if !is_prefix {
        return Err(LifecycleError(format!("INVALID scope not imported from {}: {stdout}{}", root.display(), String::from_utf8_lossy(&out.stderr))));
    }
    Ok(())
}

/// Serial per-group pytest execution with standard flags.
pub fn run_groups(
    venv: &Path,
    root: &Path,
    groups: &[(String, Vec<String>)],
    marks: &str,
    envs: &[(String, String)],
    deadline: Option<Instant>,
    extra_first: &[String],
) -> i32 {
    let base_args = ["-m", "pytest", "-q", "-c", "pyproject.toml", "--rootdir", ".", "-p", "no:cacheprovider", "-p", "e2_winshim"];
    let tail_args = ["-x", "--timeout=600", "-m", marks];
    for (cwd, args) in groups.iter().filter(|(_, a)| !a.is_empty()) {
        let mut cmd: Vec<&str> = base_args.to_vec();
        cmd.extend(extra_first.iter().map(String::as_str));
        cmd.extend_from_slice(&tail_args);
        cmd.extend(args.iter().map(String::as_str));
        let t0 = Instant::now();
        let code = call(venv.to_string_lossy().as_ref(), &cmd, &root.join(cwd), envs, deadline);
        println!("proto {cwd}: exit {code} in {:.1}s", t0.elapsed().as_secs_f64());
        if !matches!(code, 0 | 5) { return 1; }
    }
    0
}

/// Refusal evaluation rules.
pub fn verdict(
    arm_exit: i32,
    fresh: bool,
    level2_ops: i64,
    query_evidence: Option<&Path>,
) -> (i32, Vec<String>) {
    let mut reasons = Vec::new();
    if arm_exit != 0 { reasons.push(format!("arm exit {arm_exit}")); }
    if !fresh { reasons.push("replay not fresh or short of golden coverage".into()); }
    if level2_ops > 0 && !matches!(query_evidence, Some(p) if p.is_file()) {
        reasons.push(format!("{level2_ops} L2 differences without query-differential evidence"));
    }
    let code = if reasons.is_empty() { 0 } else if arm_exit == 2 { 2 } else { 1 };
    (code, reasons)
}

/// Arm environment setup with thread caps and pythonpath.
pub fn arm_env(
    root: &Path,
    paths: &[PathBuf],
    extra: &[(String, String)],
) -> Vec<(String, String)> {
    let pythonpath = std::iter::once(root.join("src"))
        .chain(paths.iter().cloned())
        .map(|p| p.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(";");
    let mut env = vec![
        ("OPENBLAS_NUM_THREADS".into(), "1".into()),
        ("OMP_NUM_THREADS".into(), "1".into()),
        ("MKL_NUM_THREADS".into(), "1".into()),
        ("PYTHONWARNINGS".into(), "ignore".into()),
        ("PYTHONDONTWRITEBYTECODE".into(), "1".into()),
        ("PYTHONPATH".into(), pythonpath),
    ];
    for (k, v) in extra {
        if let Some(e) = env.iter_mut().find(|(ek, _)| ek == k) { e.1 = v.clone(); }
        else { env.push((k.clone(), v.clone())); }
    }
    env
}

/// Build execution groups by directory with deselections.
pub fn build_groups(
    files: &[String],
    bad: &[String],
    packages: &[&str],
    replay_target: Option<&Path>,
) -> Vec<(String, Vec<String>)> {
    let root_files: Vec<String> = files.iter().filter(|f| f.starts_with("tests/")).cloned().collect();
    let mut groups = vec![(".".into(), root_files)];
    for p in packages {
        let prefix = format!("packages/facdrone-{p}/");
        let pkg_files = files.iter().filter_map(|f| f.strip_prefix(&prefix)).map(Into::into).collect();
        groups.push((format!("packages/facdrone-{p}"), pkg_files));
    }
    let mut res: Vec<(String, Vec<String>)> = groups.into_iter().filter_map(|(cwd, args)| {
        if args.is_empty() { return None; }
        let prefix = if cwd == "." { "" } else { &format!("{cwd}/") };
        let mut group_args = args;
        for n in bad.iter().filter_map(|b| b.strip_prefix(prefix)) {
            if cwd != "." || n.starts_with("tests/") {
                group_args.push(format!("--deselect={n}"));
            }
        }
        Some((cwd, group_args))
    }).collect();
    if let (Some(target), Some(first)) = (replay_target, res.first_mut()) {
        first.1.extend(["-p".into(), "facdrone_testing.fixtures".into(), target.to_string_lossy().into()]);
    }
    res
}
