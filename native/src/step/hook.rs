//! The pre-push hook of a repository that takes steps: the complexity
//! budgets, then the step guard, then the spec audit where the pinned
//! auditor is installed. One process reads the pushed refs once and stops
//! the push by its exit code.

use super::guard::{judge, pushed};
use crate::adapters::driver::process;
use crate::cli_deconvolute::cli_hook::{pre_push_budgets, PrePushArgs, PASSED};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Arguments of `step install-hook`.
#[derive(clap::Args, Clone, Debug)]
pub struct InstallHookArgs {
    /// The repository.
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,
    /// Replace a pre-push hook that is there already.
    #[arg(long)]
    pub force: bool,
}

/// Arguments of `step pre-push`, as git gives them to the hook.
#[derive(clap::Args, Clone, Debug)]
pub struct StepPrePushArgs {
    /// The name of the remote.
    pub remote: Option<String>,
    /// The location of the remote.
    pub url: Option<String>,
    /// The repository.
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,
}

pub const HOOK: &str = "#!/bin/sh\n# lesscode-core pre-push hook: complexity budgets, then the step guard.\nexec lesscode-core step pre-push \"$@\"\n";

/// The spec auditor installed by the repository's pinned installer, if any.
fn auditor(repo: &Path) -> Option<PathBuf> {
    let names = ["tools/bin/kiro-cli", "tools/bin/kiro-cli.exe"];
    names.iter().map(|n| repo.join(n)).find(|p| p.is_file())
}

/// Exit 0 when the push may go ahead, 1 when a check refuses it.
pub fn run_pre_push_hook(args: StepPrePushArgs) -> Result<u8, String> {
    let mut input = String::new();
    let read = std::io::stdin().read_to_string(&mut input);
    read.map_err(|e| format!("standard input: {e}"))?;
    let checked = pre_push_budgets(PrePushArgs {
        remote: args.remote,
        url: args.url,
        local_ref: None,
        local_sha: None,
        config: None,
        root: None,
        stdin_lines: Some(input.clone()),
        skip_budgets: false,
    })?;
    let (refused, mut specs) = judge(&args.repo, &pushed(&input))?;
    if refused {
        return Ok(1);
    }
    specs.sort();
    specs.dedup();
    // Without the auditor installed, no spec is audited.
    let cli = auditor(&args.repo);
    for (cli, spec) in cli
        .iter()
        .flat_map(|cli| specs.iter().map(move |s| (cli, s)))
    {
        let cli = cli.to_string_lossy().into_owned();
        let command = [cli, "check".into(), "--feature".into(), spec.clone()];
        let limit = Duration::from_secs(300);
        let done = process::run(&command, &args.repo, &BTreeMap::new(), limit);
        if done.exit_code != Some(0) {
            println!("spec {spec}: the audit does not pass\n{}", done.output);
            return Ok(1);
        }
    }
    if checked {
        println!("{PASSED}");
    }
    Ok(0)
}

#[cfg(unix)]
fn mark_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn mark_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Exit 0 when the hook was written, 1 when another hook is in its place.
pub fn run_install_hook(args: InstallHookArgs) -> Result<u8, String> {
    let hooks = args.repo.join(".git").join("hooks");
    if !args.repo.join(".git").is_dir() {
        return Err(format!(
            "{} has no .git directory; run this in the main working tree",
            args.repo.display()
        ));
    }
    let path = hooks.join("pre-push");
    if path.exists() && !args.force {
        println!("{} exists; use --force to replace it", path.display());
        return Ok(1);
    }
    let failed = |e: std::io::Error| format!("{}: {e}", path.display());
    std::fs::create_dir_all(&hooks).map_err(failed)?;
    std::fs::write(&path, HOOK).map_err(failed)?;
    mark_executable(&path).map_err(failed)?;
    println!("Installed {}", path.display());
    Ok(0)
}
