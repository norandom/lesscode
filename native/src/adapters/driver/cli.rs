//! Command handlers of the driver. An `Err` is bad input (exit code 3).

use super::admission::admit;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Write a receipt to `out`, or print it when no path is given.
pub fn emit_json<T: Serialize>(value: &T, out: Option<&Path>) -> Result<(), String> {
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

/// Arguments of `deconvolute admit-pack`.
#[derive(clap::Args, Clone, Debug)]
pub struct AdmitPackArgs {
    /// The pack file to check.
    #[arg(long)]
    pub pack: PathBuf,
    /// The grammar the pack is written for.
    #[arg(long)]
    pub lang: String,
    /// Recorded controls the pack must reproduce (needs --source-root).
    #[arg(long, requires = "source_root")]
    pub controls: Option<PathBuf>,
    /// The source tree the controls refer to.
    #[arg(long, requires = "controls")]
    pub source_root: Option<PathBuf>,
    /// Where to write the admission receipt; printed when absent.
    #[arg(long)]
    pub out: Option<PathBuf>,
}

/// Exit 0 when the pack is admitted, 1 when it is rejected.
pub fn run_admit_pack(args: AdmitPackArgs) -> Result<u8, String> {
    let bytes = fs::read(&args.pack)
        .map_err(|e| format!("Cannot read the pack {}: {e}", args.pack.display()))?;
    let text = String::from_utf8(bytes)
        .map_err(|e| format!("The pack {} is not UTF-8: {e}", args.pack.display()))?;
    if let (Some(controls), Some(root)) = (&args.controls, &args.source_root) {
        if !controls.is_file() || !root.is_dir() {
            return Err(format!(
                "Controls {} or source root {} not found",
                controls.display(),
                root.display()
            ));
        }
    }
    let mut result = admit(&text, &args.lang).map_err(|e| e.to_string())?;
    if args.controls.is_some() {
        result.not_run("controls", "control replay is not available in this build");
    }
    emit_json(&result, args.out.as_deref())?;
    for line in result.feedback() {
        eprintln!("{line}");
    }
    Ok(if result.admitted { 0 } else { 1 })
}
