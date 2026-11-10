//! Command handlers of the driver. An `Err` is bad input (exit code 3).

use super::admission::admit;
use super::analysis::analyse;
use super::controls::{replay, seed_mutants, ControlFile};
use super::operators::{classify, mutants, Change};
use super::pack::Pack;
use super::registry::embedded;
use super::sites::{file_sites, Site};
use crate::adapter_traits::{CoverageGap, FrameSpec};
use crate::specimen_runner::sha256_hex_bytes;
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
    if let (Some(controls), Some(root)) = (&args.controls, &args.source_root) {
        let file = ControlFile::read(controls)?;
        match Pack::parse(&text) {
            Ok(pack) => {
                let source = file.source(root);
                result.add("controls", replay(&pack, &source, &file.cases));
                result.add("seed_mutants", seed_mutants(&pack, &source, &file.cases));
            }
            Err(_) => {
                for name in ["controls", "seed_mutants"] {
                    result.not_run(name, "the pack does not have the pack shape");
                }
            }
        }
    }
    emit_json(&result, args.out.as_deref())?;
    for line in result.feedback() {
        eprintln!("{line}");
    }
    Ok(if result.admitted { 0 } else { 1 })
}

fn read_source(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    String::from_utf8(bytes).map_err(|e| format!("{} is not UTF-8: {e}", path.display()))
}

#[derive(Serialize)]
struct AnalysisReceipt<'a> {
    receipt: &'static str,
    lang: &'a str,
    source: &'a str,
    pack_sha256: &'a str,
    sites: Vec<Site>,
    write_sites: Vec<String>,
    function_boundaries: Vec<String>,
    gaps: Vec<CoverageGap>,
    violations: Vec<String>,
    passed: bool,
}

/// `deconvolute analyze`: exit 0 when nothing was found, 1 on a gap or a
/// write outside the frame specification.
pub fn run_analyze(
    lang: &str,
    source: &Path,
    frame_spec: Option<&Path>,
    out: Option<&Path>,
) -> Result<u8, String> {
    let pack = embedded(lang).map_err(|e| e.to_string())?;
    let label = source.to_string_lossy();
    let analysis = analyse(pack, &label, &read_source(source)?)?;
    let mut violations = Vec::new();
    if let Some(spec_path) = frame_spec {
        let bytes = fs::read(spec_path)
            .map_err(|e| format!("Cannot read frame spec {}: {e}", spec_path.display()))?;
        let spec: FrameSpec =
            serde_json::from_slice(&bytes).map_err(|e| format!("Invalid frame spec JSON: {e}"))?;
        let allowed = |site: &String| {
            let file_only = site
                .rsplit_once(':')
                .map_or(site.as_str(), |(file, _)| file);
            let permits = |a: &String| file_only.ends_with(a.as_str()) || site.contains(a.as_str());
            spec.allowed_write_sites.iter().any(permits)
        };
        violations = analysis
            .write_sites
            .iter()
            .filter(|site| !allowed(site))
            .cloned()
            .collect();
    }
    let passed = analysis.gaps.is_empty() && violations.is_empty();
    let receipt = AnalysisReceipt {
        receipt: "deconvolution-analysis-v2",
        lang,
        source: &label,
        pack_sha256: &pack.digest,
        sites: analysis.sites,
        write_sites: analysis.write_sites,
        function_boundaries: analysis.boundaries,
        gaps: analysis.gaps,
        violations,
        passed,
    };
    emit_json(&receipt, out)?;
    Ok(if passed { 0 } else { 1 })
}

#[derive(Serialize)]
struct MutantRecord {
    file: String,
    site: Site,
    operator: String,
    sha256: String,
    #[serde(skip)]
    text: String,
}

#[derive(Serialize)]
struct MutationReceipt<'a> {
    receipt: &'static str,
    lang: &'a str,
    source: &'a str,
    out_dir: String,
    pack_sha256: &'a str,
    sites: usize,
    sites_without_operator: Vec<Site>,
    invalid_mutants: usize,
    mutants_available: usize,
    mutants_generated: usize,
    seed: Option<u64>,
    mutants: Vec<MutantRecord>,
    passed: bool,
}

/// A small generator defined here, so that a seed selects the same mutants
/// on every platform and in every release (xorshift64*).
fn next_random(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// Keep `limit` of the items, chosen by `seed`, in their original order.
fn select<T>(items: Vec<T>, limit: usize, seed: u64) -> Vec<T> {
    let mut order: Vec<usize> = (0..items.len()).collect();
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15 | 1;
    for i in (1..order.len()).rev() {
        order.swap(i, (next_random(&mut state) % (i as u64 + 1)) as usize);
    }
    let mut keep = order[..limit.min(order.len())].to_vec();
    keep.sort_unstable();
    let mut items: Vec<Option<T>> = items.into_iter().map(Some).collect();
    keep.into_iter().filter_map(|i| items[i].take()).collect()
}

/// Every valid mutant of a text, the sites without an operator, the number
/// of sites and the number of mutants that were not a real change.
fn all_mutants(
    pack: &Pack,
    name: &str,
    text: &str,
) -> Result<(Vec<MutantRecord>, Vec<Site>, usize, usize), String> {
    let sites = file_sites(pack, name, text)?;
    let (mut found, mut bare, mut invalid) = (Vec::new(), Vec::new(), 0);
    for site in &sites {
        let of_site = mutants(pack, text, site)?;
        if of_site.is_empty() {
            bare.push(site.clone());
        }
        for mutant in of_site {
            if classify(pack, text, &mutant.text)? != Change::Changed {
                invalid += 1;
                continue;
            }
            found.push(MutantRecord {
                file: String::new(),
                site: site.clone(),
                sha256: sha256_hex_bytes(mutant.text.as_bytes()),
                operator: mutant.operator,
                text: mutant.text,
            });
        }
    }
    Ok((found, bare, sites.len(), invalid))
}

/// `deconvolute mutate`: mutant files go to `out_dir` only; exit 0 with
/// mutants, 1 with none.
pub fn run_mutate(
    lang: &str,
    source: &Path,
    out_dir: &Path,
    out: Option<&Path>,
    selection: Option<(usize, u64)>,
) -> Result<u8, String> {
    let pack = embedded(lang).map_err(|e| e.to_string())?;
    let label = source.to_string_lossy();
    let text = read_source(source)?;
    let name = source.file_name().map(|n| n.to_string_lossy().into_owned());
    let name = name.ok_or_else(|| format!("{label} is not a file path"))?;
    // Report a file that does not parse before anything is written.
    analyse(pack, &label, &text)?;
    let (mut found, bare, sites, invalid) = all_mutants(pack, &name, &text)?;
    let available = found.len();
    if let Some((limit, seed)) = selection {
        found = select(found, limit, seed);
    }
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("Cannot create out-dir {}: {e}", out_dir.display()))?;
    let extension = source.extension().map(|e| e.to_string_lossy().into_owned());
    for (index, mutant) in found.iter_mut().enumerate() {
        let label: String = mutant.operator.chars().take(40).collect();
        let label = label.replace(|c: char| !c.is_ascii_alphanumeric(), "_");
        mutant.file = format!("mutant_{index:04}_{label}");
        if let Some(extension) = &extension {
            mutant.file = format!("{}.{extension}", mutant.file);
        }
        let path = out_dir.join(&mutant.file);
        fs::write(&path, &mutant.text)
            .map_err(|e| format!("Cannot write mutant {}: {e}", path.display()))?;
    }
    let receipt = MutationReceipt {
        receipt: "deconvolution-mutation-v2",
        lang,
        source: &label,
        out_dir: out_dir.to_string_lossy().into_owned(),
        pack_sha256: &pack.digest,
        sites,
        sites_without_operator: bare,
        invalid_mutants: invalid,
        mutants_available: available,
        mutants_generated: found.len(),
        seed: selection.map(|(_, seed)| seed),
        passed: !found.is_empty(),
        mutants: found,
    };
    emit_json(&receipt, out)?;
    if !receipt.passed {
        eprintln!("No mutation operator matched {label}; 0 mutants were generated.");
        return Ok(1);
    }
    Ok(0)
}
