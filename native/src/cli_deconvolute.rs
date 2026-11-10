//! Deconvolution CLI subcommands for static analysis, mutation, and density gating.
//!
//! Governed by:
//! - REQ-DOG-2.1 through REQ-DOG-2.5, REQ-DOG-6.1 through REQ-DOG-6.3
//! - `.kiro/specs/dogfood-self-application/design.md`

use clap::Subcommand;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::adapters::driver::cli::{self as driver_cli, emit_json};

#[path = "cli_hook.rs"]
pub mod cli_hook;

#[derive(Subcommand, Debug, Clone)]
pub enum DeconvoluteCommand {
    /// List the sites, state writes, function boundaries and declared gaps of a file.
    Analyze {
        #[arg(long)]
        lang: String,
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        frame_spec: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Write the mutants the language pack gives for a file.
    Mutate {
        #[arg(long)]
        lang: String,
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        /// Write at most this many mutants, chosen by --seed.
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long, default_value_t = 0)]
        seed: u64,
    },
    /// Evaluate mechanical density reduction thresholds.
    Density {
        #[arg(long)]
        lang: String,
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        candidate: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Check a language pack before it is used.
    AdmitPack(driver_cli::AdmitPackArgs),
    /// Git pre-push hook; enforces complexity budgets only.
    PrePush(cli_hook::PrePushArgs),
    /// Install standard Git pre-push hook script in .git/hooks.
    InstallHook(cli_hook::InstallHookArgs),
}

#[derive(Serialize)]
pub struct DensityReceipt {
    pub receipt: &'static str,
    pub lang: String,
    pub baseline: String,
    pub candidate: String,
    pub baseline_lines: usize,
    pub candidate_lines: usize,
    pub line_reduction_pct: f64,
    pub baseline_stmts: usize,
    pub candidate_stmts: usize,
    pub stmt_reduction_pct: f64,
    pub passed: bool,
    pub verdict: &'static str,
}

fn read_text(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))
}

fn normalize_lang(lang: &str) -> Result<&'static str, String> {
    match lang.to_lowercase().as_str() {
        "rust" | "rs" => Ok("rust"),
        "java" => Ok("java"),
        "typescript" | "ts" => Ok("typescript"),
        "python" | "py" => Ok("python"),
        _ => Err(format!(
            "Unsupported language '{lang}'. Supported: rust, java, typescript, python"
        )),
    }
}

pub fn run(cmd: DeconvoluteCommand) -> Result<u8, String> {
    match cmd {
        DeconvoluteCommand::Analyze {
            lang,
            source,
            frame_spec,
            out,
        } => driver_cli::run_analyze(
            normalize_lang(&lang)?,
            &source,
            frame_spec.as_deref(),
            out.as_deref(),
        ),
        DeconvoluteCommand::Mutate {
            lang,
            source,
            out_dir,
            out,
            limit,
            seed,
        } => driver_cli::run_mutate(
            normalize_lang(&lang)?,
            &source,
            &out_dir,
            out.as_deref(),
            limit.map(|limit| (limit, seed)),
        ),
        DeconvoluteCommand::Density {
            lang,
            baseline,
            candidate,
            out,
        } => run_density(&lang, &baseline, &candidate, out.as_deref()),
        DeconvoluteCommand::AdmitPack(args) => driver_cli::run_admit_pack(args),
        DeconvoluteCommand::PrePush(args) => cli_hook::run_pre_push(args),
        DeconvoluteCommand::InstallHook(args) => cli_hook::run_install_hook(args),
    }
}

fn count_metrics(lang: &str, text: &str) -> (usize, usize) {
    match lang {
        "rust" => (
            crate::adapters::rust::metrics::count_canonical_lines(text),
            crate::adapters::rust::metrics::count_ast_statements(text),
        ),
        "java" => (
            crate::adapters::java::metrics::count_canonical_lines(text),
            crate::adapters::java::metrics::count_ast_statements(text),
        ),
        "typescript" => (
            crate::adapters::typescript::metrics::count_canonical_lines(text),
            crate::adapters::typescript::metrics::count_ast_statements(text),
        ),
        _ => (
            crate::density::count_canonical_lines(text),
            crate::density::count_canonical_lines(text),
        ),
    }
}

fn run_density(
    lang: &str,
    baseline_path: &Path,
    candidate_path: &Path,
    out: Option<&Path>,
) -> Result<u8, String> {
    let lang = normalize_lang(lang)?;
    let base_text = read_text(baseline_path)?;
    let cand_text = read_text(candidate_path)?;

    let (base_lines, base_stmts) = count_metrics(lang, &base_text);
    let (cand_lines, cand_stmts) = count_metrics(lang, &cand_text);

    let delta_lines = base_lines.saturating_sub(cand_lines);
    let line_reduction_pct = if base_lines > 0 {
        (delta_lines as f64 / base_lines as f64) * 100.0
    } else {
        0.0
    };

    let delta_stmts = base_stmts.saturating_sub(cand_stmts);
    let stmt_reduction_pct = if base_stmts > 0 {
        (delta_stmts as f64 / base_stmts as f64) * 100.0
    } else {
        0.0
    };

    let passed = line_reduction_pct >= 20.0
        && delta_lines >= 50
        && stmt_reduction_pct >= 10.0
        && delta_stmts >= 10;

    let receipt = DensityReceipt {
        receipt: "deconvolution-density-v1",
        lang: lang.into(),
        baseline: baseline_path.to_string_lossy().to_string(),
        candidate: candidate_path.to_string_lossy().to_string(),
        baseline_lines: base_lines,
        candidate_lines: cand_lines,
        line_reduction_pct,
        baseline_stmts: base_stmts,
        candidate_stmts: cand_stmts,
        stmt_reduction_pct,
        passed,
        verdict: if passed { "pass" } else { "marginal" },
    };
    emit_json(&receipt, out)?;
    Ok(if passed { 0 } else { 1 })
}
