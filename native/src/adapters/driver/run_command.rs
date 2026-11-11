//! `deconvolute run`: mutate the configured files and run the project's own
//! tests against every mutant, in copies only.

use super::annotations::{site_of, validate, Decision};
use super::config::RunConfig;
use super::functions::select;
use super::operators::mutants_of_sites;
use super::pack::Pack;
use super::registry::embedded;
use super::runner::{run_batch, BatchOutput, Job, MutantResult, ERROR};
use super::sites::{eligible_sites, Site};
use super::source::SourceSet;
use super::store::Store;
use super::workspace::check_separate;
use crate::specimen_runner::sha256_hex_bytes;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Arguments of `deconvolute run`.
#[derive(clap::Args, Clone, Debug)]
pub struct RunArgs {
    /// The run configuration (files, commands, limits).
    #[arg(long)]
    pub config: PathBuf,
    /// Where the working copies and reusable outcomes are kept; outside the source root.
    #[arg(long)]
    pub work: PathBuf,
    /// Copies to run at once; the configuration's value when absent.
    #[arg(long)]
    pub workers: Option<usize>,
    /// Mutate only the sites of these annotations that pass validation.
    #[arg(long)]
    pub annotations: Option<PathBuf>,
    /// Run every mutant again, even one with a stored outcome.
    #[arg(long)]
    pub no_reuse: bool,
    /// Where to write the run receipt.
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Serialize)]
struct RunReceipt<'a> {
    receipt: &'static str,
    driver: &'static str,
    language: &'a str,
    pack_sha256: &'a str,
    config_sha256: String,
    source_files_sha256: BTreeMap<String, String>,
    sites: usize,
    /// Sites outside the functions the configuration names; absent when it names none.
    #[serde(skip_serializing_if = "Option::is_none")]
    sites_not_selected: Option<usize>,
    sites_without_operator: Vec<Site>,
    invalid_before_run: usize,
    annotation_decisions: Option<Vec<Decision>>,
    counts_by_class: BTreeMap<String, usize>,
    counts_by_kind: BTreeMap<String, BTreeMap<String, usize>>,
    incomplete: bool,
    #[serde(flatten)]
    batch: BatchOutput,
}

/// The sites to mutate: every eligible site, or the accepted annotations.
fn chosen_sites(
    args: &RunArgs,
    source: &SourceSet,
    pack: &Pack,
) -> Result<(Vec<Site>, Option<Vec<Decision>>), String> {
    let Some(path) = &args.annotations else {
        return Ok((eligible_sites(source, pack)?, None));
    };
    let bytes = std::fs::read(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    let annotations: Vec<serde_json::Value> = serde_json::from_slice(&bytes)
        .map_err(|e| format!("{} is not a list of annotations: {e}", path.display()))?;
    let decisions = validate(source, pack, &annotations)?;
    let mut sites = Vec::new();
    for (annotation, decision) in annotations.iter().zip(&decisions) {
        if decision.accepted {
            sites.extend(site_of(source, pack, annotation)?);
        }
    }
    Ok((sites, Some(decisions)))
}

/// The mutants to run, with what was found on the way.
struct Planned {
    jobs: Vec<Job>,
    /// Sites for which the pack offers no operator.
    bare: Vec<Site>,
    /// Mutants that were offered and do not parse.
    invalid: usize,
    /// The text of each file that has a site.
    texts: BTreeMap<String, String>,
}

fn planned(
    source: &SourceSet,
    sites: &[Site],
    pack: &Pack,
    identity: &str,
) -> Result<Planned, String> {
    let mut texts: BTreeMap<String, String> = BTreeMap::new();
    let (mut jobs, mut bare, mut invalid) = (Vec::new(), Vec::new(), 0);
    for file in &source.files {
        let of_file: Vec<&Site> = sites.iter().filter(|s| s.file == *file).collect();
        if of_file.is_empty() {
            continue;
        }
        let text = source.raw(file)?;
        let original = sha256_hex_bytes(text.as_bytes());
        for (site, of_site) in of_file.iter().zip(mutants_of_sites(pack, &text, &of_file)?) {
            if of_site.offered == 0 {
                bare.push((*site).clone());
            }
            invalid += of_site.offered - of_site.valid.len();
            for mutant in of_site.valid {
                let sha256 = sha256_hex_bytes(mutant.text.as_bytes());
                let key = Store::key(&original, &sha256, identity);
                jobs.push(Job {
                    number: jobs.len() + 1,
                    site: (*site).clone(),
                    operator: mutant.operator,
                    text: mutant.text,
                    sha256,
                    key,
                });
            }
        }
        texts.insert(file.clone(), text);
    }
    Ok(Planned {
        jobs,
        bare,
        invalid,
        texts,
    })
}

/// Exit 0 when every mutant has a class, 2 when errors remain; a void run
/// or bad input is exit 3.
pub fn run_run(args: RunArgs) -> Result<u8, String> {
    let config = RunConfig::read(&args.config)?;
    let pack = embedded(&config.language).map_err(|e| e.to_string())?;
    check_separate(&config.source_root, &args.work)?;
    let source = SourceSet {
        root: config.source_root.clone(),
        files: config.files.clone(),
        test_paths: config.test_paths.clone(),
    };
    let (sites, decisions) = chosen_sites(&args, &source, pack)?;
    let (sites, sites_not_selected) = select(sites, &config.functions, &source, pack)?;
    let identity = config.identity();
    let Planned {
        jobs,
        bare,
        invalid,
        texts,
    } = planned(&source, &sites, pack, &identity)?;
    let store = match args.no_reuse {
        true => None,
        false => Some(Store::open(&args.work.join("outcomes"))?),
    };
    let run_id = args.out.to_string_lossy().into_owned();
    let workers = config.parallel(args.workers);
    super::interrupt::install();
    let batch = run_batch(&config, &args.work, jobs, store.as_ref(), workers, &run_id);
    let (counts_by_class, counts_by_kind) = counts(&batch.mutants);
    let incomplete = counts_by_class.contains_key(ERROR);
    let void = batch.void.clone();
    let receipt = RunReceipt {
        receipt: "deconvolution-run-v1",
        driver: env!("CARGO_PKG_VERSION"),
        language: &config.language,
        pack_sha256: &pack.digest,
        config_sha256: sha256_hex_bytes(identity.as_bytes()),
        source_files_sha256: texts
            .iter()
            .map(|(file, text)| (file.clone(), sha256_hex_bytes(text.as_bytes())))
            .collect(),
        sites: sites.len(),
        sites_not_selected,
        sites_without_operator: bare,
        invalid_before_run: invalid,
        annotation_decisions: decisions,
        counts_by_class,
        counts_by_kind,
        incomplete,
        batch,
    };
    super::cli::emit_json(&receipt, Some(&args.out))?;
    if let Some(why) = void {
        return Err(format!("the run is void: {why}"));
    }
    Ok(if incomplete { 2 } else { 0 })
}

/// Counts of results by class, and by site kind and class.
fn counts(
    mutants: &[MutantResult],
) -> (
    BTreeMap<String, usize>,
    BTreeMap<String, BTreeMap<String, usize>>,
) {
    let mut by_class: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_kind: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for mutant in mutants {
        *by_class.entry(mutant.class.clone()).or_default() += 1;
        let kind = by_kind
            .entry(mutant.site.kind.name().to_owned())
            .or_default();
        *kind.entry(mutant.class.clone()).or_default() += 1;
    }
    (by_class, by_kind)
}
