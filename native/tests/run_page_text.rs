//! The run page inside the binary, with its view library
//! (`.kiro/specs/run-dashboard`, task 4.1). The page is looked at as text;
//! no browser is started here.

use lesscode_core::dashboard::log::RunLog;
use lesscode_core::dashboard::text::{live, report};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// The name of the SVG format. It stands in the library as a name for
/// elements and is not fetched.
const SVG: &str = "http://www.w3.org/2000/svg";

fn kept(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn library() -> String {
    kept("page/htm-preact-standalone.umd.js")
}

/// The report of a run that has only begun, and the page as it is served:
/// the two pages the tool gives out.
fn pages() -> [String; 2] {
    let begun = RunLog::new("step plan", "2.1.0", &[], 1_795_443_600_000).snapshot();
    [report(&begun).unwrap(), live("/secret/events")]
}

#[test]
fn the_page_holds_its_style_its_code_and_the_library() {
    let library = library();
    for page in pages() {
        assert!(page.starts_with("<!doctype html>"));
        assert!(page.contains("<style>") && page.contains("</style>"));
        assert!(page.contains(&library), "the library is not in the page");
        // The page's own code, which draws with the library.
        assert!(page.contains("window.htmPreact"));
        assert!(page.contains("new EventSource(events)"));
        // Nothing is left to be filled, and nothing is fetched from a file.
        assert!(!page.contains("/*LIB*/"));
        for fetched in ["<link", " src=", ".src ", "@import", "url("] {
            assert!(!page.contains(fetched), "the page has {fetched}");
        }
    }
}

#[test]
fn the_page_names_no_address_on_another_machine() {
    for page in pages() {
        let rest = page.replace(SVG, "");
        let found = rest.find("://").map(|at| {
            let from = at.saturating_sub(10);
            rest.get(from..(at + 40).min(rest.len())).unwrap_or("")
        });
        assert_eq!(found, None);
    }
}

#[test]
fn the_page_is_under_150_kib_without_data() {
    let [_, served] = pages();
    assert!(served.len() < 150 * 1024, "{} bytes", served.len());
    // The library is the larger part that is not ours; it must be in the count.
    assert!(served.len() > library().len());
}

#[test]
fn the_markers_of_the_page_are_filled_and_only_they() {
    let [written, served] = pages();
    assert!(served.contains("const events = \"/secret/events\";\n"));
    assert!(served.contains("const data = /*DATA*/null;\n"));
    assert!(written.contains("const events = /*EVENTS*/null;\n"));
    assert!(written.contains("const data = {\"schema\":1,"));
    // A text of the run cannot end the script it stands in.
    let mut log = RunLog::new("</script><b>", "2.1.0", &[], 1_795_443_600_000);
    log.beat(1_795_443_601_000);
    let page = report(&log.snapshot()).unwrap();
    assert_eq!(
        page.matches("</script>").count(),
        written.matches("</script>").count()
    );
}

#[test]
fn the_library_is_the_one_the_notices_name() {
    let hash: String = Sha256::digest(library().as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    for file in ["bundled.json", "THIRD_PARTY_NOTICES.txt", "LICENSES.md"] {
        assert!(
            kept(file).contains(&hash),
            "{file} does not name the library with the hash {hash}: \
             write the hash to bundled.json and run scripts/license-inventory.ps1 -Crate native"
        );
    }
}
