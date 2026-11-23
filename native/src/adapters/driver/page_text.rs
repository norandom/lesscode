//! The run page as text: served while a run goes on, and written once at
//! its end with the snapshot inside (`.kiro/specs/run-dashboard`, tasks 3.2
//! and 4.1; design, "One file").
//!
//! The page and its view library are files under `page/` of the crate and
//! are taken into the binary here. `bundled.json` names the library, where
//! it comes from and its hash.

use super::page_snapshot::Snapshot;
use serde_json::Value;
use std::path::Path;
use std::sync::LazyLock;

const TEMPLATE: &str = include_str!("../../../page/run.html");
const LIST: &str = include_str!("../../../page/list.html");
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

/// Where the data stands in a written page, and what ends it. A report
/// keeps the page of the version of the tool that wrote it; only these two
/// are the same in every version.
const DATA_BEGINS: &str = "const data = ";
const DATA_ENDS: &str = ";\n";

/// A written page in three parts: up to its data, the data, and the rest.
fn parts(page: &str) -> Result<[&str; 3], String> {
    let cut = page
        .split_once(DATA_BEGINS)
        .and_then(|(head, rest)| Some((head, rest.split_once(DATA_ENDS)?)));
    match cut {
        Some((head, (data, tail))) => Ok([head, data, tail]),
        None => Err("this is not a page lesscode wrote: it has no data inside".to_owned()),
    }
}

/// The run inside a written report.
pub fn data_of(page: &str) -> Result<Value, String> {
    let [_, data, _] = parts(page)?;
    match serde_json::from_str::<Value>(data) {
        Ok(Value::Object(run)) => Ok(Value::Object(run)),
        Ok(_) => Err("the page was written without a run inside".to_owned()),
        Err(error) => Err(format!("the data inside the page cannot be read: {error}")),
    }
}

/// The same report with other data inside; the page around it stays as it
/// was written.
pub fn with_data(page: &str, data: &Value) -> Result<String, String> {
    let [head, _, tail] = parts(page)?;
    let data = serde_json::to_string(data).map_err(|e| e.to_string())?;
    let data = inside(&data);
    Ok(format!("{head}{DATA_BEGINS}{data}{DATA_ENDS}{tail}"))
}

/// The page that lists several reports, with the list inside.
pub fn list(reports: &Value) -> Result<String, String> {
    let data = serde_json::to_string(reports).map_err(|e| e.to_string())?;
    let page = put(LIST, LIBRARY_PLACE, LIBRARY);
    Ok(put(&page, DATA, &inside(&data)))
}
