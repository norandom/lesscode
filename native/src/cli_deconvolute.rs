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
use crate::adapters::driver::measure::{measure, Size};
use crate::adapters::driver::registry::embedded;
use crate::adapters::driver::run_command::{run_run, RunArgs};

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
    /// Run the project's own tests against every mutant, in copies only.
    Run(RunArgs),
    /// Keep the sites that enough annotation lists agree on.
    MergeAnnotations(crate::adapters::driver::merge::MergeArgs),
    /// Git pre-push hook; enforces complexity budgets only.
    PrePush(cli_hook::PrePushArgs),
    /// Install a pre-push hook that runs the complexity budgets.
    InstallHook(cli_hook::InstallHookArgs),
}

#[derive(Serialize)]
pub struct DensityReceipt {
    pub receipt: &'static str,
    pub lang: String,
    pub baseline: String,
    pub candidate: String,
    /// Syntax measures of both files; the verdict rests on tokens and nodes.
    pub baseline_size: Size,
    pub candidate_size: Size,
    pub token_reduction_pct: f64,
    pub tokens_removed: usize,
    pub node_reduction_pct: f64,
    /// For information only: lines depend on layout.
    pub line_reduction_pct: f64,
    pub thresholds: DensityThresholds,
    pub passed: bool,
    pub verdict: &'static str,
}

/// What `pass` needs. The percentages are those of the earlier line and
/// statement rule; 400 tokens stand for its 50 lines.
#[derive(Serialize)]
pub struct DensityThresholds {
    pub min_token_reduction_pct: f64,
    pub min_tokens_removed: usize,
    pub min_node_reduction_pct: f64,
}

const THRESHOLDS: DensityThresholds = DensityThresholds {
    min_token_reduction_pct: 20.0,
    min_tokens_removed: 400,
    min_node_reduction_pct: 10.0,
};

fn reduction_pct(before: usize, after: usize) -> f64 {
    match before {
        0 => 0.0,
        _ => before.saturating_sub(after) as f64 / before as f64 * 100.0,
    }
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
        DeconvoluteCommand::Run(args) => run_run(args),
        DeconvoluteCommand::MergeAnnotations(args) => {
            crate::adapters::driver::merge::run_merge(args)
        }
        DeconvoluteCommand::PrePush(args) => cli_hook::run_pre_push(args),
        DeconvoluteCommand::InstallHook(args) => cli_hook::run_install_hook(args),
    }
}

/// `deconvolute density`: compare two files by syntax, so that layout
/// cannot change the verdict. Exit 0 pass, 1 marginal, 3 when a file cannot
/// be read or does not parse.
fn run_density(
    lang: &str,
    baseline_path: &Path,
    candidate_path: &Path,
    out: Option<&Path>,
) -> Result<u8, String> {
    let lang = normalize_lang(lang)?;
    let pack = embedded(lang).map_err(|e| e.to_string())?;
    let size = |path: &Path| measure(pack, &path.to_string_lossy(), &read_text(path)?);
    let (before, after) = (size(baseline_path)?, size(candidate_path)?);
    let token_reduction_pct = reduction_pct(before.tokens, after.tokens);
    let tokens_removed = before.tokens.saturating_sub(after.tokens);
    let node_reduction_pct = reduction_pct(before.nodes, after.nodes);
    let passed = token_reduction_pct >= THRESHOLDS.min_token_reduction_pct
        && tokens_removed >= THRESHOLDS.min_tokens_removed
        && node_reduction_pct >= THRESHOLDS.min_node_reduction_pct;

    let receipt = DensityReceipt {
        receipt: "deconvolution-density-v2",
        lang: lang.into(),
        baseline: baseline_path.to_string_lossy().to_string(),
        candidate: candidate_path.to_string_lossy().to_string(),
        baseline_size: before,
        candidate_size: after,
        token_reduction_pct,
        tokens_removed,
        node_reduction_pct,
        line_reduction_pct: reduction_pct(before.lines, after.lines),
        thresholds: THRESHOLDS,
        passed,
        verdict: if passed { "pass" } else { "marginal" },
    };
    emit_json(&receipt, out)?;
    Ok(if passed { 0 } else { 1 })
}
