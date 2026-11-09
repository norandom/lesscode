use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn run_cli(args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_lesscode-core"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("execute lesscode-core")
}

#[test]
fn pre_push_skips_when_env_flag_set() {
    let output = run_cli(
        &["deconvolute", "pre-push"],
        &[("LESSCODE_SKIP_HOOK", "1")],
    );
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hook skipped via LESSCODE_SKIP_HOOK=1"));
}

#[test]
fn pre_push_skips_on_branch_deletion() {
    let stdin_lines = "refs/heads/refactor/old 0000000000000000000000000000000000000000 refs/heads/refactor/old 1111111111111111111111111111111111111111\n";
    let output = run_cli(
        &[
            "deconvolute",
            "pre-push",
            "--stdin-lines",
            stdin_lines,
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Branch deletion detected"));
}

#[test]
fn pre_push_passes_clean_codebase_budgets() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cfg = manifest_dir.join("complexity-budgets.json");
    let src = manifest_dir.join("src");
    let stdin_lines = "refs/heads/feature/sample aabbccdd refs/heads/feature/sample eeff0011\n";

    let output = run_cli(
        &[
            "deconvolute",
            "pre-push",
            "--config",
            cfg.to_str().unwrap(),
            "--root",
            src.to_str().unwrap(),
            "--stdin-lines",
            stdin_lines,
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Complexity budgets verified (0 violations)"));
    assert!(stdout.contains("Pre-push verification passed"));
}

#[test]
fn pre_push_rejects_violated_budgets() {
    let temp = std::env::temp_dir().join(format!("lc-hook-viol-{}", std::process::id()));
    let _ = fs::create_dir_all(&temp);
    let cfg = temp.join("budgets.json");
    let src = temp.join("src");
    let _ = fs::create_dir_all(&src);
    fs::write(src.join("unlisted.rs"), "pub fn unlisted() {}\n").unwrap();
    fs::write(&cfg, r#"{"files": []}"#).unwrap();

    let output = run_cli(
        &[
            "deconvolute",
            "pre-push",
            "--config",
            cfg.to_str().unwrap(),
            "--root",
            src.to_str().unwrap(),
            "--stdin-lines",
            "refs/heads/main 1234 refs/heads/main 5678\n",
        ],
        &[],
    );
    let _ = fs::remove_dir_all(&temp);

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Complexity budgets violated") || stderr.contains("Complexity budget ceilings exceeded"));
}

#[test]
fn install_hook_creates_portable_git_hook_script() {
    let temp = std::env::temp_dir().join(format!("lc-hook-install-{}", std::process::id()));
    let _ = fs::create_dir_all(&temp);
    let git_dir = temp.join(".git");

    let output = run_cli(
        &[
            "deconvolute",
            "install-hook",
            "--git-dir",
            git_dir.to_str().unwrap(),
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(0));

    let hook_path = git_dir.join("hooks").join("pre-push");
    assert!(hook_path.exists());
    let hook_text = fs::read_to_string(&hook_path).unwrap();
    assert!(hook_text.contains("lesscode-core deconvolute pre-push"));

    let ps1_path = git_dir.join("hooks").join("pre-push.ps1");
    assert!(ps1_path.exists());

    // Installing again without --force should fail
    let output_err = run_cli(
        &[
            "deconvolute",
            "install-hook",
            "--git-dir",
            git_dir.to_str().unwrap(),
        ],
        &[],
    );
    assert_ne!(output_err.status.code(), Some(0));

    // Installing with --force should succeed
    let output_force = run_cli(
        &[
            "deconvolute",
            "install-hook",
            "--git-dir",
            git_dir.to_str().unwrap(),
            "--force",
        ],
        &[],
    );
    assert_eq!(output_force.status.code(), Some(0));

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn refactor_branch_detection_triggers_refactoring_path() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cfg = manifest_dir.join("complexity-budgets.json");
    let src = manifest_dir.join("src");
    let stdin_lines = "refs/heads/refactor/deconvolute-cache 1122 refs/heads/refactor/deconvolute-cache 3344\n";

    let output = run_cli(
        &[
            "deconvolute",
            "pre-push",
            "--config",
            cfg.to_str().unwrap(),
            "--root",
            src.to_str().unwrap(),
            "--stdin-lines",
            stdin_lines,
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Detected major refactoring branch"));
}
