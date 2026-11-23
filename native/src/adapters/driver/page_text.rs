//! The run page as text: served while a run goes on, and written once at
//! its end with the snapshot inside (`.kiro/specs/run-dashboard`, task
//! 3.2; design, "One file").
//!
//! The page below is a plain stand-in that shows the snapshot as it is.
//! Task 4.1 puts the page of the design in its place; the two markers stay.

use super::page_snapshot::Snapshot;
use std::path::Path;

const EVENTS: &str = "/*EVENTS*/null";
const DATA: &str = "/*DATA*/null";

const PAGE: &str = r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>lesscode run</title>
<style>
:root { color-scheme: light dark; }
body { font: 14px/1.5 system-ui, sans-serif; margin: 16px; }
pre { white-space: pre-wrap; overflow-wrap: anywhere; }
</style>
</head>
<body>
<h1>lesscode run</h1>
<p id="state">Waiting for the run.</p>
<pre id="run"></pre>
<script>
const events = /*EVENTS*/null;
const data = /*DATA*/null;
const state = document.getElementById("state");
let ended = false;
function show(run) {
  ended = run.ended === true;
  state.textContent = ended ? "The run has ended: " + (run.outcome || "") : "The run goes on.";
  document.getElementById("run").textContent = JSON.stringify(run, null, 1);
}
if (data) show(data);
if (events) {
  const stream = new EventSource(events);
  stream.onmessage = (message) => { try { show(JSON.parse(message.data)); } catch (_) {} };
  stream.onerror = () => {
    stream.close();
    if (!ended) state.textContent = "The tool has stopped before the run ended.";
  };
}
</script>
</body>
</html>
"#;

/// Text that can stand inside a script of the page without ending it.
fn inside(text: &str) -> String {
    text.replace('<', "\\u003c")
}

/// The page with `value` where `marker` stands.
fn with(marker: &str, value: &str) -> String {
    let (before, after) = PAGE.split_once(marker).unwrap_or((PAGE, ""));
    format!("{before}{}{after}", inside(value))
}

/// The page that follows the stream at `events`.
pub fn live(events: &str) -> String {
    with(EVENTS, &serde_json::to_string(events).unwrap_or_default())
}

/// The page with a snapshot inside: the report of a run.
pub fn report(snapshot: &Snapshot) -> Result<String, String> {
    let data = serde_json::to_string(snapshot).map_err(|e| e.to_string())?;
    Ok(with(DATA, &data))
}

/// Writes the report of a run to `place`.
pub fn write_report(place: &Path, snapshot: &Snapshot) -> Result<(), String> {
    std::fs::write(place, report(snapshot)?).map_err(|e| format!("{}: {e}", place.display()))
}
