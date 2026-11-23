//! Asking for the run page with `--page` and `--open`
//! (`.kiro/specs/run-dashboard`, task 3.2). The browser is a stand-in that
//! notes the address it was given and reads the page from it.

use lesscode_core::adapters::driver::page_serve::LOCAL;
use lesscode_core::step::plan::{run_plan, PlanArgs};
use lesscode_core::step::plan_show::PageArgs;
use serde_json::{json, Value};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

const APP: &str = "def step(state, a):\n    if a > 3:\n        raise ValueError(\"big\")\n    state.total = a\n    return a\n";

fn toy() -> String {
    let test_binary = std::env::current_exe().unwrap();
    let profile = test_binary.parent().unwrap().parent().unwrap();
    let name = format!("toy_check{}", std::env::consts::EXE_SUFFIX);
    let path = profile.join("examples").join(name);
    assert!(path.is_file(), "not run: {} was not built", path.display());
    path.to_string_lossy().into_owned()
}

/// A toy project whose test passes, and its run configuration.
fn project(name: &str) -> (PathBuf, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("run_page_options_{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("project/src")).unwrap();
    fs::write(dir.join("project/src/app.py"), APP).unwrap();
    let toy = toy();
    let test = [&toy, "--file", "src/app.py", "--expect", "if a > 3:"];
    let config = json!({
        "language": "python", "source_root": "project", "files": ["src/app.py"], "test": test,
        "timeout_floor_s": 10, "timeout_ceiling_s": 60, "workers": 2,
    });
    fs::write(dir.join("run.json"), config.to_string()).unwrap();
    let config = dir.join("run.json");
    (dir, config)
}

fn arguments(dir: &Path, config: &Path, show: PageArgs) -> PlanArgs {
    PlanArgs {
        config: config.to_path_buf(),
        work: dir.join("work"),
        frame_spec: None,
        workers: None,
        out: dir.join("plan.json"),
        show,
        watch: None,
    }
}

/// What the stand-in browser was given, and what it read there.
static OPENED: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

fn browser(address: &str) -> Result<(), String> {
    let rest = address.strip_prefix("http://").unwrap();
    let (host, path) = rest.split_once('/').unwrap();
    let mut stream = TcpStream::connect(host).map_err(|e| e.to_string())?;
    write!(stream, "GET /{path} HTTP/1.1\r\nHost: {host}\r\n\r\n").unwrap();
    let mut page = String::new();
    let _ = stream.read_to_string(&mut page);
    OPENED.lock().unwrap().push((address.to_owned(), page));
    Ok(())
}

fn no_browser(_address: &str) -> Result<(), String> {
    OPENED.lock().unwrap().push((String::new(), String::new()));
    Err("there is none".into())
}

/// The snapshot inside a written report.
fn inside(report: &Path) -> Value {
    let text = fs::read_to_string(report).unwrap();
    let (_, rest) = text.split_once("const data = ").unwrap();
    let (data, _) = rest.split_once(";\n").unwrap();
    serde_json::from_str(data).unwrap()
}

#[test]
fn without_either_option_nothing_is_written_or_opened_and_the_plan_is_the_same() {
    let (dir, config) = project("neither");
    let show = PageArgs {
        opener: Some(browser),
        ..PageArgs::default()
    };
    assert_eq!(run_plan(arguments(&dir, &config, show)), Ok(0));
    let plain = fs::read(dir.join("plan.json")).unwrap();
    let mut left: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    left.sort();
    assert_eq!(left, ["plan.json", "project", "run.json", "work"]);
    assert!(OPENED.lock().unwrap().is_empty());
    // The same run with the page: the plan is the same, byte for byte.
    let show = PageArgs {
        page: Some(Some(dir.join("report.html"))),
        ..PageArgs::default()
    };
    assert_eq!(run_plan(arguments(&dir, &config, show)), Ok(0));
    assert_eq!(fs::read(dir.join("plan.json")).unwrap(), plain);
}

#[test]
fn page_writes_the_report_at_the_named_place_and_opens_nothing() {
    let (dir, config) = project("page");
    let place = dir.join("report.html");
    let show = PageArgs {
        page: Some(Some(place.clone())),
        opener: Some(browser),
        ..PageArgs::default()
    };
    assert_eq!(run_plan(arguments(&dir, &config, show)), Ok(0));
    assert!(
        OPENED.lock().unwrap().is_empty(),
        "never opened unless asked"
    );
    let shown = inside(&place);
    assert_eq!(shown["ended"], true);
    assert_eq!(shown["exit_code"], 0);
    assert_eq!(shown["command"], "step plan");
    let plan: Value = serde_json::from_slice(&fs::read(dir.join("plan.json")).unwrap()).unwrap();
    let noticed: u64 = plan["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["noticed"].as_u64().unwrap())
        .sum();
    assert_eq!(shown["faults"]["noticed"], noticed);
    assert!(noticed > 0, "{plan}");
    // What the run touched (task 5.1): the copies were measured at the end,
    // and outside a repository nothing is named as written into one.
    assert!(shown["copies"]["bytes"].as_u64() > Some(0), "{shown}");
    assert_eq!(shown["writes_in_repository"], json!([]));
}

#[test]
fn open_starts_the_browser_once_on_the_served_page() {
    let (dir, config) = project("open");
    let place = dir.join("report.html");
    let show = PageArgs {
        page: Some(Some(place.clone())),
        open: true,
        listen: None,
        opener: Some(browser),
    };
    assert_eq!(run_plan(arguments(&dir, &config, show)), Ok(0));
    let opened = OPENED.lock().unwrap().clone();
    assert_eq!(opened.len(), 1, "once, at the start");
    let (address, page) = &opened[0];
    assert!(address.starts_with("http://127.0.0.1:"), "{address}");
    assert!(page.starts_with("HTTP/1.1 200 OK"), "{page}");
    assert!(page.contains("new EventSource(events)"), "{page}");
    assert_eq!(inside(&place)["ended"], true);
    // Nothing is served after the run.
    let host = address
        .strip_prefix("http://")
        .unwrap()
        .split('/')
        .next()
        .unwrap();
    assert!(TcpStream::connect(host).is_err());
}

#[test]
fn a_browser_that_cannot_be_started_does_not_change_the_run() {
    let (dir, config) = project("nobrowser");
    let place = dir.join("report.html");
    let show = PageArgs {
        page: Some(Some(place.clone())),
        open: true,
        listen: None,
        opener: Some(no_browser),
    };
    assert_eq!(run_plan(arguments(&dir, &config, show)), Ok(0));
    assert_eq!(OPENED.lock().unwrap().len(), 1, "tried once");
    assert_eq!(inside(&place)["exit_code"], 0);
}

#[test]
fn a_listener_that_cannot_start_does_not_change_the_run() {
    let (dir, config) = project("taken");
    let taken = TcpListener::bind(LOCAL).unwrap();
    let place = dir.join("report.html");
    let show = PageArgs {
        page: Some(Some(place.clone())),
        open: true,
        listen: Some(taken.local_addr().unwrap().to_string()),
        opener: Some(browser),
    };
    assert_eq!(run_plan(arguments(&dir, &config, show)), Ok(0));
    assert!(OPENED.lock().unwrap().is_empty(), "nothing to open");
    assert_eq!(inside(&place)["ended"], true);
}

#[test]
fn a_place_whose_directory_is_missing_is_refused_before_anything_runs() {
    let (dir, config) = project("missing");
    let show = PageArgs {
        page: Some(Some(dir.join("nowhere/report.html"))),
        ..PageArgs::default()
    };
    let why = run_plan(arguments(&dir, &config, show)).unwrap_err();
    assert!(why.contains("does not exist"), "{why}");
    assert!(!dir.join("work").exists(), "nothing ran");
}

fn git(dir: &Path, args: &[&str]) {
    let done = Command::new("git").arg("-C").arg(dir).args(args).output();
    assert!(done.unwrap().status.success(), "git {args:?}");
}

/// The tool itself, as a person starts it.
fn tool(dir: &Path, config: &Path, more: &[&str]) -> (Option<i32>, String, String) {
    let done = Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
        .args(["step", "plan", "--config"])
        .arg(config)
        .arg("--work")
        .arg(dir.join("work"))
        .arg("--out")
        .arg(dir.join("plan.json"))
        .args(more)
        .output()
        .unwrap();
    let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();
    (done.status.code(), text(&done.stdout), text(&done.stderr))
}

#[test]
fn a_place_among_a_repositorys_files_is_refused_unless_it_is_named() {
    let (dir, config) = project("tracked");
    git(&dir, &["init", "--quiet"]);
    // Not named: beside the plan, which Git would list. Refused before anything runs.
    let (code, _, said) = tool(&dir, &config, &["--page"]);
    assert_eq!(code, Some(3), "{said}");
    assert!(
        said.contains("Refused") && said.contains("--page <place>"),
        "{said}"
    );
    assert!(!dir.join("work").exists() && !dir.join("plan.html").exists());
    // Named: the same place is written, and where it is gets printed.
    let named = dir.join("plan.html");
    let (code, printed, said) = tool(&dir, &config, &["--page", named.to_str().unwrap()]);
    assert_eq!(code, Some(0), "{said}");
    assert!(
        printed.contains(&format!("The page of the run is at {}", named.display())),
        "{printed}"
    );
    assert_eq!(inside(&named)["ended"], true);
    // The plan, the page and the copies lie in the repository here, and the page names them.
    assert_eq!(
        inside(&named)["writes_in_repository"],
        json!(["plan.json", "plan.html", "work"])
    );
}

#[test]
fn without_a_place_the_page_goes_beside_the_plan_where_git_ignores_it() {
    let (dir, config) = project("beside");
    git(&dir, &["init", "--quiet"]);
    fs::write(dir.join(".gitignore"), "*.html\n").unwrap();
    let (code, printed, said) = tool(&dir, &config, &["--page"]);
    assert_eq!(code, Some(0), "{said}");
    let beside = dir.join("plan.html");
    assert!(printed.contains(&beside.display().to_string()), "{printed}");
    assert_eq!(inside(&beside)["ended"], true);
    assert!(
        !printed.contains("http://"),
        "nothing is served without --open: {printed}"
    );
}
