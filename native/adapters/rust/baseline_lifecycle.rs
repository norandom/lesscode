//! Process lifecycle, execution supervisor, and pytest group orchestrator.
//!
//! Extracted verbatim from `native/src/recording.rs` for isolated whole-responsibility
//! deconvolution evaluation (R52 prototype gate).

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

/// Run cmd; past the deadline kill its whole tree. Returns the arm exit code, 124 on deadline.
pub fn call(
    program: &str,
    args: &[&str],
    cwd: &Path,
    envs: &[(String, String)],
    deadline: Option<Instant>,
) -> i32 {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .envs(envs.iter().cloned());
    let Ok(mut child) = command.spawn() else {
        return 127;
    };
    let poll = Duration::from_millis(50);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.code().unwrap_or(-1),
            Ok(None) => {}
            Err(_) => return -1,
        }
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID"])
                    .arg(child.id().to_string())
                    .output();
                println!("proto: deadline exceeded (infrastructure, not a verdict)");
                return 124;
            }
            std::thread::sleep(poll.min(deadline.saturating_duration_since(Instant::now())));
        } else {
            std::thread::sleep(poll);
        }
    }
}

/// Fail closed when the scope module is not imported from `root`.
pub fn probe(
    venv: &Path,
    root: &Path,
    module: &str,
    envs: &[(String, String)],
) -> Result<(), LifecycleError> {
    let code = format!("import {module} as m; print(m.__file__)");
    let output = Command::new(venv)
        .arg("-c")
        .arg(&code)
        .envs(envs.iter().cloned())
        .output()
        .map_err(|e| LifecycleError(format!("probe failed: {e}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let imported_from = PathBuf::from(if stdout.is_empty() {
        "/".to_string()
    } else {
        stdout.clone()
    });
    if !same_file_prefix(&imported_from, root) {
        return Err(LifecycleError(format!(
            "INVALID scope not imported from {}: {stdout}{stderr}",
            root.display()
        )));
    }
    Ok(())
}

fn same_file_prefix(candidate: &Path, root: &Path) -> bool {
    let (c, r) = (candidate.canonicalize().ok(), root.canonicalize().ok());
    match (c, r) {
        (Some(c), Some(r)) => c.starts_with(r),
        _ => candidate.starts_with(root),
    }
}

/// Serial per-group pytest with the frozen flags; exit 0 only if every group passes.
pub fn run_groups(
    venv: &Path,
    root: &Path,
    groups: &[(String, Vec<String>)],
    marks: &str,
    envs: &[(String, String)],
    deadline: Option<Instant>,
    extra_first: &[String],
) -> i32 {
    for (cwd, args) in groups {
        if args.is_empty() {
            continue;
        }
        let mut cmd: Vec<String> = [
            "-m",
            "pytest",
            "-q",
            "-c",
            "pyproject.toml",
            "--rootdir",
            ".",
            "-p",
            "no:cacheprovider",
            "-p",
            "e2_winshim",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        cmd.extend(extra_first.iter().cloned());
        cmd.extend(
            ["-x", "--timeout=600", "-m", marks]
                .iter()
                .map(|s| s.to_string()),
        );
        cmd.extend(args.iter().cloned());
        let arg_refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
        let t0 = Instant::now();
        let code = call(
            venv.to_string_lossy().as_ref(),
            &arg_refs,
            &root.join(cwd),
            envs,
            deadline,
        );
        println!(
            "proto {cwd}: exit {code} in {:.1}s",
            t0.elapsed().as_secs_f64()
        );
        if !matches!(code, 0 | 5) {
            return 1;
        }
    }
    0
}

/// Frozen acceptance refusal rules: the arm must pass.
pub fn verdict(
    arm_exit: i32,
    fresh: bool,
    level2_ops: i64,
    query_evidence: Option<&Path>,
) -> (i32, Vec<String>) {
    let mut reasons = if arm_exit == 0 {
        Vec::new()
    } else {
        vec![format!("arm exit {arm_exit}")]
    };
    if !fresh {
        reasons.push("replay not fresh or short of golden coverage".into());
    }
    if level2_ops > 0 && !matches!(query_evidence, Some(p) if p.is_file()) {
        reasons.push(format!(
            "{level2_ops} L2 differences without query-differential evidence"
        ));
    }
    if reasons.is_empty() {
        return (0, reasons);
    }
    (if arm_exit == 2 { 2 } else { 1 }, reasons)
}

/// The arm environment: thread caps, warning silencers, module path.
pub fn arm_env(
    root: &Path,
    paths: &[PathBuf],
    extra: &[(String, String)],
) -> Vec<(String, String)> {
    let mut pythonpath = root.join("src").to_string_lossy().into_owned();
    for p in paths {
        pythonpath.push(';');
        pythonpath.push_str(&p.to_string_lossy());
    }
    let mut env: Vec<(String, String)> = [
        ("OPENBLAS_NUM_THREADS", "1"),
        ("OMP_NUM_THREADS", "1"),
        ("MKL_NUM_THREADS", "1"),
        ("PYTHONWARNINGS", "ignore"),
        ("PYTHONDONTWRITEBYTECODE", "1"),
        ("PYTHONPATH", pythonpath.as_str()),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    for (k, v) in extra {
        if let Some(entry) = env.iter_mut().find(|(ek, _)| ek == k) {
            entry.1 = v.clone();
        } else {
            env.push((k.clone(), v.clone()));
        }
    }
    env
}

/// (cwd relative to root, pytest args): root-suite seed files, then satellite packages.
pub fn build_groups(
    files: &[String],
    bad: &[String],
    packages: &[&str],
    replay_target: Option<&Path>,
) -> Vec<(String, Vec<String>)> {
    let pick = |prefix: &str, xs: &[String]| -> Vec<String> {
        xs.iter()
            .filter_map(|x| x.strip_prefix(prefix))
            .map(|x| x.to_string())
            .collect()
    };
    let mut out: Vec<(String, Vec<String>)> = vec![(
        ".".into(),
        pick(
            "",
            &files
                .iter()
                .filter(|f| f.starts_with("tests/"))
                .cloned()
                .collect::<Vec<_>>(),
        ),
    )];
    for p in packages {
        let prefix = format!("packages/facdrone-{p}/");
        out.push((format!("packages/facdrone-{p}"), pick(&prefix, files)));
    }
    out = out
        .into_iter()
        .map(|(cwd, args)| {
            if args.is_empty() {
                (cwd, Vec::new())
            } else {
                let prefix = if cwd == "." { "" } else { &format!("{cwd}/") };
                let mut args = args;
                for n in pick(prefix, bad) {
                    if cwd != "." || n.starts_with("tests/") {
                        args.push(format!("--deselect={n}"));
                    }
                }
                (cwd, args)
            }
        })
        .collect();
    if let Some(target) = replay_target {
        if let Some(first) = out.first_mut() {
            first.1.extend([
                "-p".to_string(),
                "facdrone_testing.fixtures".to_string(),
                target.to_string_lossy().into_owned(),
            ]);
        }
    }
    out.into_iter()
        .filter(|(_, args)| !args.is_empty())
        .collect()
}
