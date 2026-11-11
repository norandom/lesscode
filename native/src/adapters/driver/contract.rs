//! The adapter traits of `adapter_traits.rs`, answered by the driver: frame
//! analysis, and the six control kinds mapped onto pack-driven operators.

use super::analysis::{analyse, outside_frame};
use super::operators::mutants_of_sites;
use super::registry::embedded;
use super::sites::file_sites;
use crate::adapter_traits::{
    AnalysisRequest, AnalysisResult, CoverageGap, MutantOutcome, MutationBatch, MutationKind,
    MutationRequest, MutationResult, MutationSpec,
};
use std::fs;

/// The operator names that stand for a control kind, as prefixes.
/// `TruncatedStream` has none: it described a fault in what a process
/// prints, which no edit of a source file reproduces.
pub fn operators_of(kind: &MutationKind) -> &'static [&'static str] {
    match kind {
        MutationKind::InvertedCondition => &["negate"],
        MutationKind::BoundaryOffByOne => &["swap:"],
        MutationKind::InvertedErrorOrder => &["swap_next"],
        MutationKind::MissingFailureState => &["drop:", "rewrite:"],
        MutationKind::SilentNone => &["drop_statement", "literal:"],
        MutationKind::TruncatedStream => &[],
    }
}

fn read(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Cannot read {path}: {e}"))
}

/// Frame analysis of the request's files with the shipped pack of `language`.
/// A file that cannot be read or parsed is a gap, never a silent pass.
pub fn analyze(language: &str, request: &AnalysisRequest) -> AnalysisResult {
    let mut result = AnalysisResult {
        permitted_writes: request.frame_spec.allowed_write_sites.clone(),
        ..AnalysisResult::default()
    };
    for path in &request.source_paths {
        let pack = embedded(language).map_err(|e| e.to_string());
        let analysis = pack.and_then(|pack| analyse(pack, path, &read(path)?));
        match analysis {
            Ok(analysis) => {
                let outside = outside_frame(&analysis.write_sites, &request.frame_spec);
                result.violations.extend(outside);
                result.write_sites.extend(analysis.write_sites);
                result.unverified_gaps.extend(analysis.gaps);
            }
            Err(reason) => result.unverified_gaps.push(CoverageGap {
                reason,
                location: Some(path.clone()),
            }),
        }
    }
    result.is_valid = result.violations.is_empty() && result.unverified_gaps.is_empty();
    result
}

/// The lines and operator names of every valid mutant of a file.
fn operators_in(language: &str, path: &str) -> Result<Vec<(usize, usize, String)>, String> {
    let pack = embedded(language).map_err(|e| e.to_string())?;
    let text = read(path)?;
    let sites = file_sites(pack, path, &text)?;
    let all: Vec<_> = sites.iter().collect();
    let mut found = Vec::new();
    for (site, of_site) in sites.iter().zip(mutants_of_sites(pack, &text, &all)?) {
        for mutant in of_site.valid {
            found.push((site.start_line, site.end_line, mutant.operator));
        }
    }
    Ok(found)
}

fn outcome(
    spec: &MutationSpec,
    found: &Result<Vec<(usize, usize, String)>, String>,
) -> MutantOutcome {
    let found = match found {
        Ok(found) => found,
        Err(reason) => {
            return MutantOutcome::InfrastructureFailure {
                reason: reason.clone(),
            }
        }
    };
    let prefixes = operators_of(&spec.kind);
    if prefixes.is_empty() {
        return MutantOutcome::Invalid {
            reason: format!("no pack-driven operator stands for {:?}", spec.kind),
        };
    }
    // A location that ends in a line number selects that line; otherwise
    // the whole file is searched.
    let line = spec
        .location
        .rsplit(':')
        .next()
        .and_then(|l| l.parse().ok());
    let at_line = |(start, end, _): &&(usize, usize, String)| {
        line.is_none_or(|line: usize| (*start..=*end).contains(&line))
    };
    let of_kind = |(_, _, operator): &&(usize, usize, String)| {
        prefixes.iter().any(|prefix| operator.starts_with(prefix))
    };
    match found.iter().filter(at_line).filter(of_kind).count() {
        0 => MutantOutcome::Invalid {
            reason: format!("no mutant for {:?} at {}", spec.kind, spec.location),
        },
        mutants => MutantOutcome::NotRun { mutants },
    }
}

/// For each control, whether the driver has a mutant of the mapped operator
/// at the named place. No test is run here, so nothing is ever reported as
/// killed; `deconvolute run` is what runs a project's tests.
pub fn generate(language: &str, request: &MutationRequest) -> MutationBatch {
    let found = operators_in(language, &request.target_path);
    let results: Vec<_> = request
        .controls
        .iter()
        .map(|spec| MutationResult {
            spec: spec.clone(),
            outcome: outcome(spec, &found),
        })
        .collect();
    MutationBatch {
        denominator: results.len(),
        results,
        all_killed: false,
        killed_count: 0,
    }
}
