//! Research tooling, not part of the product: check a list of annotations
//! with the driver's validator and print, for each, whether it is accepted,
//! why not, and the site it names. Also prints the eligible sites.
//!
//!     cargo run --example annotation_sites -- --lang L --source-root R --file F [--test-path T] --annotations A
//!
//! Used by the Slice B annotation-layer baseline (R82) to compare what
//! repeated model runs select.

use clap::Parser;
use lesscode_core::adapters::driver::annotations::{site_of, validate};
use lesscode_core::adapters::driver::registry::embedded;
use lesscode_core::adapters::driver::sites::{eligible_sites, Site};
use lesscode_core::adapters::driver::source::SourceSet;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    lang: String,
    #[arg(long)]
    source_root: PathBuf,
    #[arg(long = "file", required = true)]
    files: Vec<String>,
    #[arg(long = "test-path")]
    test_paths: Vec<String>,
    #[arg(long)]
    annotations: PathBuf,
}

fn key(site: &Site) -> String {
    let (line, column, end_line, end_column) = site.span();
    let kind = site.kind.name();
    format!("{}:{line}:{column}-{end_line}:{end_column}:{kind}", site.file)
}

fn run(args: Args) -> Result<Value, String> {
    let pack = embedded(&args.lang).map_err(|e| e.to_string())?;
    let source = SourceSet {
        root: args.source_root,
        files: args.files,
        test_paths: args.test_paths,
    };
    let bytes = std::fs::read(&args.annotations).map_err(|e| e.to_string())?;
    let annotations: Vec<Value> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let decisions = validate(&source, pack, &annotations)?;
    let mut rows = Vec::new();
    for (annotation, decision) in annotations.iter().zip(&decisions) {
        let site = match decision.accepted {
            true => site_of(&source, pack, annotation)?.map(|s| key(&s)),
            false => None,
        };
        rows.push(json!({"accepted": decision.accepted, "outcome": decision.outcome(),
                         "kind": annotation["kind"], "site": site}));
    }
    let eligible: Vec<String> = eligible_sites(&source, pack)?.iter().map(key).collect();
    Ok(json!({"annotations": rows, "eligible": eligible}))
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(3)
        }
    }
}
