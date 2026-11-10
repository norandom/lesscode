//! Research tooling, not part of the product: list every mutant a pack file
//! gives for some source files, as JSON, so that two packs can be compared.
//!
//!     cargo run --example pack_mutants -- --pack P --source-root R --file F [--file F] [--test-path T]
//!
//! Used by the Slice B drift baseline (R66) to compare a model-written pack
//! with the shipped pack by site, operator and hash of the mutated text.

use clap::Parser;
use lesscode_core::adapters::driver::operators::mutants;
use lesscode_core::adapters::driver::pack::Pack;
use lesscode_core::adapters::driver::sites::eligible_sites;
use lesscode_core::adapters::driver::source::SourceSet;
use lesscode_core::specimen_runner::sha256_hex_bytes;
use serde_json::json;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    pack: PathBuf,
    #[arg(long)]
    source_root: PathBuf,
    #[arg(long = "file", required = true)]
    files: Vec<String>,
    #[arg(long = "test-path")]
    test_paths: Vec<String>,
}

fn run(args: Args) -> Result<serde_json::Value, String> {
    let text = std::fs::read_to_string(&args.pack).map_err(|e| e.to_string())?;
    let pack = Pack::parse(&text).map_err(|e| e.to_string())?;
    let source = SourceSet {
        root: args.source_root,
        files: args.files,
        test_paths: args.test_paths,
    };
    let sites = eligible_sites(&source, &pack)?;
    let mut found = Vec::new();
    for site in &sites {
        let text = source.raw(&site.file)?;
        for mutant in mutants(&pack, &text, site)? {
            found.push(json!({
                "site": site,
                "operator": mutant.operator,
                "sha256": sha256_hex_bytes(mutant.text.as_bytes()),
            }));
        }
    }
    Ok(json!({"pack_sha256": pack.digest, "sites": sites.len(), "mutants": found}))
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(result) => {
            println!("{result}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("pack_mutants: {error}");
            ExitCode::from(3)
        }
    }
}
