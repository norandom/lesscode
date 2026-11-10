//! Deconvolution CLI subcommands for static analysis, mutation, and density gating.
//!
//! Governed by:
//! - REQ-DOG-2.1 through REQ-DOG-2.5, REQ-DOG-6.1 through REQ-DOG-6.3
//! - `.kiro/specs/dogfood-self-application/design.md`

use clap::Subcommand;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::adapter_traits::{CoverageGap, FrameSpec, MutationKind, MutationSpec};
use crate::adapters::java::JavaAdapter;
use crate::adapters::rust::RustAdapter;
use crate::adapters::typescript::TypeScriptAdapter;

#[path = "cli_hook.rs"]
pub mod cli_hook;

#[derive(Subcommand, Debug, Clone)]
pub enum DeconvoluteCommand {
    /// Static AST effect scanning and dynamic gap validation.
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
    /// Generate mandatory adverse mutation variants.
    Mutate {
        #[arg(long)]
        lang: String,
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
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
    /// Git pre-push hook; enforces complexity budgets only.
    PrePush(cli_hook::PrePushArgs),
    /// Install standard Git pre-push hook script in .git/hooks.
    InstallHook(cli_hook::InstallHookArgs),
}

#[derive(Serialize)]
pub struct AnalysisReceipt {
    pub receipt: &'static str,
    pub lang: String,
    pub source: String,
    pub write_sites: Vec<String>,
    pub gaps: Vec<CoverageGap>,
    pub violations: Vec<String>,
    pub passed: bool,
}

#[derive(Serialize)]
pub struct MutationReceipt {
    pub receipt: &'static str,
    pub lang: String,
    pub source: String,
    pub out_dir: String,
    pub mutants_generated: usize,
    pub passed: bool,
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

fn emit_json<T: Serialize>(value: &T, out: Option<&Path>) -> Result<(), String> {
    let json_bytes =
        serde_json::to_vec_pretty(value).map_err(|e| format!("Serialization error: {e}"))?;
    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(path, &json_bytes)
            .map_err(|e| format!("Cannot write output {}: {e}", path.display()))?;
    } else {
        let text = String::from_utf8_lossy(&json_bytes);
        println!("{text}");
    }
    Ok(())
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

/// Python has no analyzer or mutant generator: produce nothing and exit 2 (incomplete).
fn python_not_implemented(what: &str) -> Result<u8, String> {
    eprintln!("Python {what} is not implemented in this release; no result was produced.");
    Ok(2)
}

pub fn run(cmd: DeconvoluteCommand) -> Result<u8, String> {
    match cmd {
        DeconvoluteCommand::Analyze {
            lang,
            source,
            frame_spec,
            out,
        } => run_analyze(&lang, &source, frame_spec.as_deref(), out.as_deref()),
        DeconvoluteCommand::Mutate {
            lang,
            source,
            out_dir,
            out,
        } => run_mutate(&lang, &source, &out_dir, out.as_deref()),
        DeconvoluteCommand::Density {
            lang,
            baseline,
            candidate,
            out,
        } => run_density(&lang, &baseline, &candidate, out.as_deref()),
        DeconvoluteCommand::PrePush(args) => cli_hook::run_pre_push(args),
        DeconvoluteCommand::InstallHook(args) => cli_hook::run_install_hook(args),
    }
}

fn run_analyze(
    lang: &str,
    source_path: &Path,
    frame_spec_path: Option<&Path>,
    out: Option<&Path>,
) -> Result<u8, String> {
    let lang = normalize_lang(lang)?;
    if lang == "python" {
        return python_not_implemented("analysis");
    }
    let source_text = read_text(source_path)?;
    let path_str = source_path.to_string_lossy().to_string();

    let (write_sites, gaps) = match lang {
        "rust" => RustAdapter::new()
            .analyzer
            .scan_source(&path_str, &source_text),
        "java" => JavaAdapter::new()
            .analyzer
            .scan_source(&path_str, &source_text),
        "typescript" => TypeScriptAdapter::new()
            .analyzer
            .scan_source(&path_str, &source_text),
        _ => unreachable!(),
    };

    let mut violations = Vec::new();
    if let Some(spec_path) = frame_spec_path {
        let spec_bytes = fs::read(spec_path)
            .map_err(|e| format!("Cannot read frame spec {}: {e}", spec_path.display()))?;
        let spec: FrameSpec = serde_json::from_slice(&spec_bytes)
            .map_err(|e| format!("Invalid frame spec JSON: {e}"))?;

        for site in &write_sites {
            let file_only = site.split(':').next().unwrap_or(site);
            let allowed = spec
                .allowed_write_sites
                .iter()
                .any(|a| file_only.ends_with(a) || site.contains(a));
            if !allowed {
                violations.push(site.clone());
            }
        }
    }

    let passed = gaps.is_empty() && violations.is_empty();
    let receipt = AnalysisReceipt {
        receipt: "deconvolution-analysis-v1",
        lang: lang.into(),
        source: path_str,
        write_sites,
        gaps,
        violations,
        passed,
    };
    emit_json(&receipt, out)?;
    Ok(if passed { 0 } else { 1 })
}

fn mutation_specs(loc: &str) -> [MutationSpec; 4] {
    [
        (
            MutationKind::MissingFailureState,
            "Missing failure invariant",
        ),
        (MutationKind::InvertedCondition, "Inverted condition"),
        (
            MutationKind::InvertedErrorOrder,
            "Inverted error precedence",
        ),
        (MutationKind::SilentNone, "Silent dropped state/env"),
    ]
    .map(|(kind, desc)| MutationSpec {
        id: format!("{kind:?}"),
        kind,
        location: loc.to_string(),
        description: desc.into(),
    })
}

fn generate_mutants(lang: &str, src: &str, specs: &[MutationSpec]) -> Vec<(MutationKind, String)> {
    let mut mutants = Vec::new();
    match lang {
        "rust" => {
            let gen = crate::adapters::rust::RustMutantGenerator::new();
            for s in specs {
                if let Ok(m) = gen.apply_mutation(src, s) {
                    mutants.push((s.kind.clone(), m));
                }
            }
        }
        "java" => {
            let gen = crate::adapters::java::JavaMutantGenerator::new();
            for s in specs {
                if let Ok(m) = gen.apply_mutation(src, s) {
                    mutants.push((s.kind.clone(), m));
                }
            }
        }
        "typescript" => {
            let gen = crate::adapters::typescript::mutation::TypeScriptMutantGenerator::new();
            for s in specs {
                if let Ok(m) = gen.apply_mutation(src, s) {
                    mutants.push((s.kind.clone(), m));
                }
            }
        }
        _ => {}
    }
    mutants
}

fn write_mutants(
    out_dir: &Path,
    ext: &str,
    mutants: &[(MutationKind, String)],
) -> Result<(), String> {
    for (idx, (kind, src)) in mutants.iter().enumerate() {
        let path = out_dir.join(format!("mutant_{idx}_{kind:?}.{ext}"));
        fs::write(&path, src)
            .map_err(|e| format!("Cannot write mutant {}: {e}", path.display()))?;
    }
    Ok(())
}

fn run_mutate(
    lang: &str,
    source_path: &Path,
    out_dir: &Path,
    out: Option<&Path>,
) -> Result<u8, String> {
    let lang = normalize_lang(lang)?;
    if lang == "python" {
        return python_not_implemented("mutation");
    }
    let source_text = read_text(source_path)?;
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("Cannot create out-dir {}: {e}", out_dir.display()))?;

    let path_str = source_path.to_string_lossy().to_string();
    let specs = mutation_specs(&path_str);
    let mutants = generate_mutants(lang, &source_text, &specs);
    let ext = match lang {
        "rust" => "rs",
        "java" => "java",
        "typescript" => "ts",
        _ => unreachable!(),
    };
    write_mutants(out_dir, ext, &mutants)?;

    let receipt = MutationReceipt {
        receipt: "deconvolution-mutation-v1",
        lang: lang.into(),
        source: path_str,
        out_dir: out_dir.to_string_lossy().to_string(),
        mutants_generated: mutants.len(),
        passed: !mutants.is_empty(),
    };
    emit_json(&receipt, out)?;
    if !receipt.passed {
        eprintln!(
            "No mutation operator matched {}; 0 mutants were generated.",
            receipt.source
        );
        return Ok(1);
    }
    Ok(0)
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
