//! The run page as text: served while a run goes on, and written once at
//! its end with the snapshot inside (`.kiro/specs/run-dashboard`, tasks 3.2
//! and 4.1; design, "One file").
//!
//! The page and its view library are files under `page/` of the crate and
//! are taken into the binary here. `bundled.json` names the library, where
//! it comes from and its hash.

use super::page_snapshot::Snapshot;
use std::path::Path;
use std::sync::LazyLock;

const TEMPLATE: &str = include_str!("../../../page/run.html");
const LIBRARY: &str = include_str!("../../../page/htm-preact-standalone.umd.js");

const LIBRARY_PLACE: &str = "/*LIB*/";
const EVENTS: &str = "/*EVENTS*/null";
const DATA: &str = "/*DATA*/null";

/// `text` with `value` where `marker` first stands.
fn put(text: &str, marker: &str, value: &str) -> String {
    let (before, after) = text.split_once(marker).unwrap_or((text, ""));
    format!("{before}{value}{after}")
}

/// The page with its library, and the two places for a run still open.
fn page() -> &'static str {
    static PAGE: LazyLock<String> = LazyLock::new(|| put(TEMPLATE, LIBRARY_PLACE, LIBRARY));
    &PAGE
}

/// Text that can stand inside a script of the page without ending it.
fn inside(text: &str) -> String {
    text.replace('<', "\\u003c")
}

/// The page that follows the stream at `events`.
pub fn live(events: &str) -> String {
    let address = serde_json::to_string(events).unwrap_or_default();
    put(page(), EVENTS, &inside(&address))
}

/// The page with a snapshot inside: the report of a run.
pub fn report(snapshot: &Snapshot) -> Result<String, String> {
    let data = serde_json::to_string(snapshot).map_err(|e| e.to_string())?;
    Ok(put(page(), DATA, &inside(&data)))
}

/// Writes the report of a run to `place`.
pub fn write_report(place: &Path, snapshot: &Snapshot) -> Result<(), String> {
    std::fs::write(place, report(snapshot)?).map_err(|e| format!("{}: {e}", place.display()))
}
