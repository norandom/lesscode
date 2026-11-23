//! `step reports`: one page that lists several reports
//! (`.kiro/specs/run-dashboard`, task 7.1; requirement 8).
//!
//! Reads the run inside each report, writes a copy of each report and the
//! list into a new directory, and changes nothing else. With
//! `--without-names-of` the copies of that project's reports, and so the
//! list, have numbers where the names of files and functions stood. Nothing
//! is written unless every report could be read.

use super::plan_page::now;
use crate::adapters::driver::page_list::{list, project_of, row};
use crate::adapters::driver::page_names::leave_out;
use crate::adapters::driver::page_snapshot::{stamp, SCHEMA};
use crate::adapters::driver::page_text;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(clap::Args)]
pub struct ReportsArgs {
    /// The reports to list: pages that a run wrote with --page.
    #[arg(required = true, value_name = "REPORT")]
    pub reports: Vec<PathBuf>,
    /// A new directory for the list (index.html) and a copy of each report.
    #[arg(long, value_name = "DIRECTORY")]
    pub out: PathBuf,
    /// Leave the names of files and functions of this project out of the list and of its reports; a project is named as its repository's directory is. Repeat for several projects.
    #[arg(long = "without-names-of", value_name = "PROJECT")]
    pub without_names_of: Vec<String>,
}

/// A report as it goes into the list's directory, and its row of the list.
struct Listed {
    name: String,
    page: String,
    project: String,
    row: Value,
}

fn said(place: &Path, error: impl std::fmt::Display) -> String {
    format!("{}: {error}", place.display())
}

/// Reads one report; leaves the names out where its project is named.
fn listed(place: &Path, at: usize, without: &[String]) -> Result<Listed, String> {
    let page = fs::read_to_string(place).map_err(|e| said(place, e))?;
    let mut run = page_text::data_of(&page).map_err(|e| said(place, e))?;
    if run.get("schema") != Some(&Value::from(SCHEMA)) {
        let found = run.get("schema").cloned().unwrap_or_default();
        let why = format!("its data has version {found}; this tool reads version {SCHEMA}");
        return Err(said(place, why));
    }
    let project = project_of(&run).unwrap_or_else(|| "not known".to_owned());
    let left_out = without.contains(&project);
    let page = if left_out {
        leave_out(&mut run);
        page_text::with_data(&page, &run).map_err(|e| said(place, e))?
    } else {
        page
    };
    let name = format!("report-{:02}.html", at + 1);
    let row = row(&run, &name, &project, left_out);
    Ok(Listed {
        name,
        page,
        project,
        row,
    })
}

/// The directory of the list must be new or empty: nothing is overwritten.
fn fresh(out: &Path) -> Result<(), String> {
    let taken = match fs::read_dir(out) {
        Ok(mut entries) => entries.next().is_some(),
        Err(_) => out.exists(),
    };
    match taken {
        true => Err(said(out, "is not a new or empty directory; name another")),
        false => Ok(()),
    }
}

/// A project named with the option must be the project of a report:
/// a name written wrongly would otherwise leave every name in.
fn all_named(reports: &[Listed], without: &[String]) -> Result<(), String> {
    let has = |project: &String| reports.iter().any(|r| &r.project == project);
    let Some(missing) = without.iter().find(|project| !has(project)) else {
        return Ok(());
    };
    let mut projects: Vec<&str> = reports.iter().map(|r| r.project.as_str()).collect();
    projects.sort_unstable();
    projects.dedup();
    Err(format!(
        "no report is of a project named {missing}; the reports are of: {}",
        projects.join(", ")
    ))
}

pub fn run_reports(args: ReportsArgs) -> Result<u8, String> {
    fresh(&args.out)?;
    let without = &args.without_names_of;
    let places = args.reports.iter().enumerate();
    let reports = places
        .map(|(at, place)| listed(place, at, without))
        .collect::<Result<Vec<Listed>, String>>()?;
    all_named(&reports, without)?;
    let rows = reports.iter().map(|r| r.row.clone()).collect();
    let index = list(env!("CARGO_PKG_VERSION"), stamp(now()), rows);
    let index = page_text::list(&index)?;

    fs::create_dir_all(&args.out).map_err(|e| said(&args.out, e))?;
    for report in &reports {
        let place = args.out.join(&report.name);
        fs::write(&place, &report.page).map_err(|e| said(&place, e))?;
    }
    let place = args.out.join("index.html");
    fs::write(&place, index).map_err(|e| said(&place, e))?;
    println!(
        "The list of {} reports is at {}",
        reports.len(),
        place.display()
    );
    Ok(0)
}
