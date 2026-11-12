//! Git pre-push hook and installer porcelain. The hook enforces complexity budgets only.
//!
//! - Bypasses branch deletions (`(delete)` or zero-sha) and `LESSCODE_SKIP_HOOK=1`.
//! - Otherwise runs the complexity budget check against `complexity-budgets.json`.
//! - Treats every branch name the same; no other check runs.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::budgets::{self, BudgetsConfig};

/// Command-line arguments for `lesscode-core deconvolute pre-push`.
#[derive(clap::Args, Clone, Debug, Default)]
pub struct PrePushArgs {
    /// Remote name (passed by git as $1).
    pub remote: Option<String>,
    /// Remote URL (passed by git as $2).
    pub url: Option<String>,
    /// Local ref being pushed (e.g. refs/heads/my-branch).
    #[arg(long)]
    pub local_ref: Option<String>,
    /// Local commit sha.
    #[arg(long)]
    pub local_sha: Option<String>,
    /// Path to complexity-budgets.json (optional override).
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Source root directory (optional override).
    #[arg(long)]
    pub root: Option<PathBuf>,
    /// Stdin lines provided directly (testing bypass).
    #[arg(long)]
    pub stdin_lines: Option<String>,
    /// Skip complexity budget checks.
    #[arg(long)]
    pub skip_budgets: bool,
}

/// Command-line arguments for `lesscode-core deconvolute install-hook`.
#[derive(clap::Args, Clone, Debug, Default)]
pub struct InstallHookArgs {
    /// Target git directory (defaults to .git).
    #[arg(long)]
    pub git_dir: Option<PathBuf>,
    /// Overwrite existing pre-push hook if present.
    #[arg(long, short)]
    pub force: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PushRef {
    pub local_ref: String,
    pub local_sha: String,
    pub remote_ref: String,
    pub remote_sha: String,
}

pub fn parse_ref_lines(input: &str) -> Vec<PushRef> {
    input
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                Some(PushRef {
                    local_ref: parts[0].to_string(),
                    local_sha: parts[1].to_string(),
                    remote_ref: parts[2].to_string(),
                    remote_sha: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn is_branch_deletion(local_ref: &str, local_sha: &str) -> bool {
    local_ref == "(delete)"
        || local_sha == "0000000000000000000000000000000000000000"
        || local_sha.chars().all(|c| c == '0')
}

fn locate_budgets(config: Option<&Path>, root: Option<&Path>) -> Option<(PathBuf, PathBuf)> {
    if let (Some(c), Some(r)) = (config, root) {
        return Some((c.to_path_buf(), r.to_path_buf()));
    }
    let candidates = [
        ("native/complexity-budgets.json", "native/src"),
        ("complexity-budgets.json", "src"),
    ];
    for (c, r) in candidates {
        let cp = PathBuf::from(c);
        let rp = PathBuf::from(r);
        if cp.exists() && rp.exists() {
            return Some((cp, rp));
        }
    }
    None
}

fn check_budgets_gate(config_path: &Path, root_path: &Path) -> Result<(), String> {
    let bytes =
        fs::read(config_path).map_err(|e| format!("Cannot read {}: {e}", config_path.display()))?;
    let config: BudgetsConfig =
        serde_json::from_slice(&bytes).map_err(|e| format!("Invalid budgets config: {e}"))?;

    let (reports, unlisted, exit_code) = budgets::check(root_path, &config)?;
    if exit_code != 0 {
        eprintln!("[lesscode-hook] ERROR: Complexity budgets violated:");
        for un in unlisted {
            eprintln!("  - unlisted file: {un}");
        }
        for rep in reports
            .iter()
            .filter(|r| r.status() == "exceeded" || r.status() == "missing")
        {
            eprintln!("  - {}: status={}", rep.path, rep.status());
        }
        return Err("Complexity budget ceilings exceeded".into());
    }
    println!("[lesscode-hook] Complexity budgets verified (0 violations).");
    Ok(())
}

fn check_ref_target(r: &PushRef, args: &PrePushArgs) -> Result<(), String> {
    if is_branch_deletion(&r.local_ref, &r.local_sha) {
        println!(
            "[lesscode-hook] Branch deletion detected ({}); skipping.",
            r.local_ref
        );
        return Ok(());
    }

    if !args.skip_budgets {
        if let Some((cfg, src)) = locate_budgets(args.config.as_deref(), args.root.as_deref()) {
            check_budgets_gate(&cfg, &src)?;
        }
    }
    Ok(())
}

/// The line that closes a push no check refused.
pub const PASSED: &str = "[lesscode-hook] Pre-push verification passed.";

pub fn run_pre_push(args: PrePushArgs) -> Result<u8, String> {
    if pre_push_budgets(args)? {
        println!("{PASSED}");
    }
    Ok(0)
}

/// The budgets of every pushed ref. True when refs were checked and none was
/// refused; the caller says "passed" once its own checks agree.
pub fn pre_push_budgets(args: PrePushArgs) -> Result<bool, String> {
    if std::env::var("LESSCODE_SKIP_HOOK").ok().as_deref() == Some("1") {
        println!("[lesscode-hook] Hook skipped via LESSCODE_SKIP_HOOK=1");
        return Ok(false);
    }

    let input = if let Some(lines) = &args.stdin_lines {
        lines.clone()
    } else if let (Some(lr), Some(ls)) = (&args.local_ref, &args.local_sha) {
        format!("{lr} {ls} {lr} 0000000000000000000000000000000000000000\n")
    } else {
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        buf
    };

    let refs = parse_ref_lines(&input);
    if refs.is_empty() {
        println!(
            "[lesscode-hook] No refs provided on stdin; running standalone budget verification."
        );
        if !args.skip_budgets {
            if let Some((cfg, src)) = locate_budgets(args.config.as_deref(), args.root.as_deref()) {
                check_budgets_gate(&cfg, &src)?;
            }
        }
        return Ok(false);
    }

    for r in &refs {
        check_ref_target(r, &args)?;
    }
    Ok(true)
}

pub fn run_install_hook(args: InstallHookArgs) -> Result<u8, String> {
    let git_dir = args.git_dir.unwrap_or_else(|| PathBuf::from(".git"));
    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)
        .map_err(|e| format!("Cannot create hooks directory {}: {e}", hooks_dir.display()))?;

    let hook_path = hooks_dir.join("pre-push");
    if hook_path.exists() && !args.force {
        return Err(format!(
            "Hook already exists at {}. Pass --force to overwrite.",
            hook_path.display()
        ));
    }

    let hook_content = "#!/bin/sh\n# lesscode-core pre-push hook\nexec lesscode-core deconvolute pre-push \"$@\"\n";
    fs::write(&hook_path, hook_content)
        .map_err(|e| format!("Cannot write hook {}: {e}", hook_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).map_err(|e| e.to_string())?;
    }

    let ps1_path = hooks_dir.join("pre-push.ps1");
    let ps1_content = "# lesscode-core pre-push hook for PowerShell\n& lesscode-core deconvolute pre-push $args\n";
    let _ = fs::write(&ps1_path, ps1_content);

    println!(
        "[lesscode-hook] Successfully installed pre-push hook at {}",
        hook_path.display()
    );
    Ok(0)
}
