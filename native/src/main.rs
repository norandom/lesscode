use clap::{Parser, Subcommand};
use lesscode_core::{
    admission, budgets, cli_deconvolute, compare::render_comparison, model::*, regenerator, step,
    surface,
};
use std::{fs, path::PathBuf, process::ExitCode};

mod cli_paths;
use cli_paths::{create_parent, fresh_output, read_json, write_new};

#[derive(Parser)]
#[command(
    version,
    about = "Read-only capability-compression preparation and comparison",
    after_help = "Exit codes: 0 prepared/accepted; 1 rejected/not-demonstrated; 2 incomplete; 3 invalid input, baseline-ineligible or IO failure. Reports require fresh output outside source roots. Checks are supplied, never executed."
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Freeze the baseline and reviewed manifest in a new external JSON file.
    Prepare {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        out: PathBuf,
        /// Explicit external artifact directories; repeat for multiple roots.
        #[arg(long)]
        artifact_root: Vec<PathBuf>,
    },
    /// Compare supplied roots and evidence; write result.json and report.md.
    Compare {
        #[arg(long)]
        frozen: PathBuf,
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        candidate: PathBuf,
        #[arg(long)]
        evidence: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        artifact_root: Vec<PathBuf>,
    },
    /// Verify artifact bytes against sidecar digests; write an admission receipt.
    Admission {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        bindings: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Verify native size budgets; write a budgets receipt.
    Budgets {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Contract-B surface derivation and candidate verification (task 8.2).
    Surface {
        #[command(subcommand)]
        cmd: surface::SurfaceCommand,
    },
    /// Two-tier regenerator controller (task 8.5); exit codes 0/2/3/4/5/6.
    Regenerate {
        #[command(subcommand)]
        cmd: regenerator::RegenerateCommand,
    },
    /// Deconvolution commands: analyze, mutate, density.
    Deconvolute {
        #[command(subcommand)]
        cmd: cli_deconvolute::DeconvoluteCommand,
    },
    /// Refactoring in checked steps: plan.
    Step {
        #[command(subcommand)]
        cmd: step::StepCommand,
    },
}
#[rustfmt::skip]
fn run(args: Args) -> Result<u8, String> {
    match args.command {
        Command::Prepare {
            manifest,
            baseline,
            out,
            artifact_root,
        } => {
            let out = fresh_output(&out, &[&baseline])?;
            let manifest: Manifest = read_json(&manifest)?;
            let frozen = lesscode_core::prepare(manifest, &baseline, &artifact_root)
                .map_err(|e| e.to_string())?;
            let bytes = serde_json::to_vec_pretty(&frozen).map_err(|e| e.to_string())?;
            fresh_output(&out, &[&baseline])?;
            create_parent(&out)?;
            write_new(&out, &bytes)?;
            println!("Prepared {}", out.display());
            Ok(0)
        }
        Command::Compare {
            frozen,
            baseline,
            candidate,
            evidence,
            out,
            artifact_root,
        } => {
            let out = fresh_output(&out, &[&baseline, &candidate])?;
            let trial: FrozenTrial = read_json(&frozen)?;
            let evidence: EvidenceBundle = read_json(&evidence)?;
            let result =
                lesscode_core::compare(&trial, &baseline, &candidate, &evidence, &artifact_root)
                    .map_err(|e| e.to_string())?;
            let report =
                render_comparison(&result, &trial, &evidence).map_err(|e| e.to_string())?;
            fresh_output(&out, &[&baseline, &candidate])?;
            create_parent(&out)?;
            fs::create_dir(&out).map_err(|e| format!("cannot create {}: {e}", out.display()))?;
            write_new(&out.join("result.json"), report.json.as_bytes())?;
            write_new(&out.join("report.md"), report.markdown.as_bytes())?;
            println!("{:?}: {}", result.status, out.display());
            for reason in &result.reasons {
                eprintln!("{}: {}", reason.subject, reason.explanation);
            }
            Ok(match result.status {
                ComparisonStatus::Accepted => 0,
                ComparisonStatus::RejectedBehavior
                | ComparisonStatus::RejectedTarget
                | ComparisonStatus::NotDemonstrated => 1,
                ComparisonStatus::Incomplete => 2,
                ComparisonStatus::Invalid | ComparisonStatus::BaselineIneligible => 3,
            })
        }
        Command::Admission { root, bindings, out } => admission::cli_run(&root, &bindings, &out),
        Command::Budgets { config, root, out } => budgets::cli_run(&root, &config, &out),
        Command::Surface { cmd } => surface::cli_run(cmd).map_err(|e| e.to_string()),
        Command::Regenerate { cmd } => regenerator::cli_run(cmd),
        Command::Deconvolute { cmd } => cli_deconvolute::run(cmd),
        Command::Step { cmd } => step::run(cmd),
    }
}
fn main() -> ExitCode {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(error) => {
            let code = if error.use_stderr() { 3 } else { 0 };
            let _ = error.print();
            return ExitCode::from(code);
        }
    };
    match run(args) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(3)
        }
    }
}
