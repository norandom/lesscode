use clap::{Parser, Subcommand};
use lesscode_core::{admission, compare::render_comparison, model::*};
use serde::de::DeserializeOwned;
use std::{
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
    process::ExitCode,
};

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
}
fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("invalid JSON in {}: {e}", path.display()))
}
fn linked(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}
// Walk before canonicalization so links/junctions cannot hide behind their targets.
// Missing suffixes are resolved against the canonical existing ancestor.
fn checked_path(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let mut resolved = PathBuf::new();
    for part in absolute.components() {
        match part {
            Component::ParentDir => {
                return Err(format!("path traversal is not allowed: {}", path.display()))
            }
            Component::Normal(name)
                if name == ".."
                    || name == "."
                    || name
                        .to_str()
                        .is_some_and(|s| s.contains('/') || s.contains(char::from(92))) =>
            {
                return Err(format!("nonportable path component: {}", path.display()))
            }
            Component::CurDir => continue,
            Component::Prefix(_) => {
                resolved.push(part.as_os_str());
                continue;
            }
            _ => resolved.push(part.as_os_str()),
        }
        match fs::symlink_metadata(&resolved) {
            Ok(metadata) => {
                if linked(&metadata) {
                    return Err(format!(
                        "symlink/reparse point is not allowed: {}",
                        resolved.display()
                    ));
                }
                resolved = fs::canonicalize(&resolved)
                    .map_err(|e| format!("{}: {e}", resolved.display()))?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("{}: {e}", resolved.display())),
        }
    }
    Ok(resolved)
}
fn fresh_output(out: &Path, roots: &[&Path]) -> Result<PathBuf, String> {
    let out = checked_path(out)?;
    for root in roots {
        let root = checked_path(root)?;
        if !root.is_dir() {
            return Err(format!(
                "source root is not a directory: {}",
                root.display()
            ));
        }
        if out.starts_with(&root) || root.starts_with(&out) {
            return Err(format!(
                "output must be outside source roots: {}",
                out.display()
            ));
        }
    }
    match fs::symlink_metadata(&out) {
        Ok(_) => Err(format!(
            "output already exists; choose a fresh path: {}",
            out.display()
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(out),
        Err(e) => Err(format!("{}: {e}", out.display())),
    }
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("cannot create {}: {e}", path.display()))?;
    file.write_all(bytes)
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}
fn create_parent(out: &Path) -> Result<(), String> {
    fs::create_dir_all(out.parent().ok_or("output needs a parent directory")?)
        .map_err(|e| format!("cannot create output parent: {e}"))
}
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
        Command::Admission {
            root,
            bindings,
            out,
        } => admission::cli_run(&root, &bindings, &out).map_err(|e| e.to_string()),
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
