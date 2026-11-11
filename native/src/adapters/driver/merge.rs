//! `deconvolute merge-annotations`: keep the sites that enough annotation
//! lists agree on. A selection made by one model call varies from call to
//! call; the sites several calls name vary less (R85). Each list is checked
//! by the validator first, so only accepted annotations count. No model is
//! contacted.

use super::annotations::{site_of, validate};
use super::config::RunConfig;
use super::registry::embedded;
use super::source::SourceSet;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Arguments of `deconvolute merge-annotations`.
#[derive(clap::Args, Clone, Debug)]
pub struct MergeArgs {
    /// The run configuration that names the language, source root and files.
    #[arg(long)]
    pub config: PathBuf,
    /// Keep a site that at least this many lists name.
    #[arg(long, default_value_t = 2)]
    pub min: usize,
    /// Where to write the merged list, usable with `run --annotations`.
    #[arg(long)]
    pub out: PathBuf,
    /// The annotation lists to merge.
    #[arg(required = true)]
    pub lists: Vec<PathBuf>,
}

#[derive(Serialize)]
struct ListReport {
    list: String,
    annotations: usize,
    accepted: usize,
    sites: usize,
}

#[derive(Serialize)]
struct MergeReceipt {
    receipt: &'static str,
    min: usize,
    lists: Vec<ListReport>,
    /// Sites named by at least one list, and by at least `min` lists.
    sites_named: usize,
    sites_kept: usize,
    out: String,
}

type SiteKey = (String, (usize, usize, usize, usize), &'static str);

/// Exit 0 when at least one site is kept, 1 when none is.
pub fn run_merge(args: MergeArgs) -> Result<u8, String> {
    if args.min == 0 || args.min > args.lists.len() {
        return Err(format!(
            "--min {} does not fit {} lists",
            args.min,
            args.lists.len()
        ));
    }
    let config = RunConfig::read(&args.config)?;
    let pack = embedded(&config.language).map_err(|e| e.to_string())?;
    let source = SourceSet {
        root: config.source_root.clone(),
        files: config.files.clone(),
        test_paths: config.test_paths.clone(),
    };
    // Per site: how many lists name it, and the first annotation that did.
    let mut named: BTreeMap<SiteKey, (usize, Value)> = BTreeMap::new();
    let mut lists = Vec::new();
    for path in &args.lists {
        let bytes =
            std::fs::read(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        let annotations: Vec<Value> = serde_json::from_slice(&bytes)
            .map_err(|e| format!("{} is not a list of annotations: {e}", path.display()))?;
        let decisions = validate(&source, pack, &annotations)?;
        let mut of_list: BTreeMap<SiteKey, &Value> = BTreeMap::new();
        for (annotation, decision) in annotations.iter().zip(&decisions) {
            if !decision.accepted {
                continue;
            }
            if let Some(site) = site_of(&source, pack, annotation)? {
                let key = (site.file.clone(), site.span(), site.kind.name());
                of_list.entry(key).or_insert(annotation);
            }
        }
        lists.push(ListReport {
            list: path.to_string_lossy().into_owned(),
            annotations: annotations.len(),
            accepted: decisions.iter().filter(|d| d.accepted).count(),
            sites: of_list.len(),
        });
        for (key, annotation) in of_list {
            let entry = named.entry(key).or_insert_with(|| (0, annotation.clone()));
            entry.0 += 1;
        }
    }
    let kept: Vec<&Value> = named
        .values()
        .filter(|(count, _)| *count >= args.min)
        .map(|(_, annotation)| annotation)
        .collect();
    let text = serde_json::to_vec_pretty(&kept).map_err(|e| e.to_string())?;
    std::fs::write(&args.out, text)
        .map_err(|e| format!("Cannot write {}: {e}", args.out.display()))?;
    let receipt = MergeReceipt {
        receipt: "deconvolution-annotation-merge-v1",
        min: args.min,
        lists,
        sites_named: named.len(),
        sites_kept: kept.len(),
        out: args.out.to_string_lossy().into_owned(),
    };
    super::cli::emit_json(&receipt, None)?;
    Ok(if kept.is_empty() { 1 } else { 0 })
}
