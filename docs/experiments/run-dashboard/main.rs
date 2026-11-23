//! Prototype for the run page (spec run-dashboard, prototype gate). It plays
//! a recorded fault run back as if it were happening: it writes the page,
//! rewrites the data beside it, can serve the same data as an event stream
//! on 127.0.0.1 with the standard library alone, and writes the page with
//! the data inside at the end. `fixtures` writes pages for fixed situations.
//! The page is in `page.html`; the view library is handed in as a file.

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, fs, process::ExitCode, thread};

const PAGE: &str = include_str!("page.html");

/// The time as browsers read it, to the millisecond, in UTC.
fn stamp(at: SystemTime) -> String {
    let since = at.duration_since(UNIX_EPOCH).unwrap_or_default();
    let (seconds, millis) = (since.as_secs() as i64, since.subsec_millis());
    let (days, rest) = (seconds.div_euclid(86_400), seconds.rem_euclid(86_400));
    // Days since 1970 to a date (the algorithm of Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{millis:03}Z", rest / 3_600, rest % 3_600 / 60, rest % 60)
}

fn page(library: &str, config: &Value, data: &Value) -> String {
    PAGE.replace("/*LIB*/", library).replace("/*CONFIG*/", &config.to_string()).replace("/*DATA*/", &data.to_string())
}

/// Puts new data in place of the old. Returns how often it had to try again,
/// or `None` when the write was lost.
fn put(directory: &Path, snapshot: &Value) -> Option<u32> {
    let (fresh, place) = (directory.join("run-data.js.new"), directory.join("run-data.js"));
    fs::write(&fresh, format!("window.__lesscodeRun({snapshot});\n")).ok()?;
    for again in 0..40 {
        if fs::rename(&fresh, &place).is_ok() {
            return Some(again);
        }
        thread::sleep(Duration::from_millis(5));
    }
    None
}

struct Recorded {
    receipt: Value,
    plan: Value,
}

impl Recorded {
    fn read(receipt: &str, plan: &str) -> Result<Self, String> {
        let load = |path: &str| -> Result<Value, String> {
            let text = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
            serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| format!("{path}: {e}"))
        };
        Ok(Self { receipt: load(receipt)?, plan: load(plan)? })
    }

    fn faults(&self) -> Vec<&Value> {
        self.receipt["mutants"].as_array().map(|m| m.iter().collect()).unwrap_or_default()
    }

    /// The run after `done` of its faults, as the page is to show it.
    fn snapshot(&self, done: usize, phase: &str, sequence: u64, began: Instant) -> Value {
        let faults = self.faults();
        let total = faults.len();
        let workers = self.receipt["workers"].as_array().cloned().unwrap_or_default();
        let dropped: Vec<Value> = workers
            .iter()
            .filter(|w| w["stopped"].is_string() && phase != "copy")
            .map(|w| json!({"copy": w["worker"], "at": stamp(SystemTime::now()), "reason": w["stopped"]}))
            .collect();
        let working = workers.len() - dropped.len();
        let mut files: BTreeMap<String, [u64; 4]> = BTreeMap::new();
        let (mut noticed, mut unnoticed, mut not_built, mut seconds) = (0u64, 0u64, 0u64, 0.0f64);
        for (at, fault) in faults.iter().enumerate() {
            let file = fault["site"]["file"].as_str().unwrap_or_default().to_owned();
            let tally = files.entry(file).or_default();
            if at >= done {
                tally[3] += 1;
                continue;
            }
            seconds += fault["seconds"].as_f64().unwrap_or(0.0);
            match fault["class"].as_str() {
                Some("killed") => {
                    noticed += 1;
                    tally[0] += 1;
                }
                Some("survived") => {
                    unnoticed += 1;
                    tally[1] += 1;
                }
                _ => {
                    not_built += 1;
                    tally[2] += 1;
                }
            }
        }
        let each = if done > 0 { seconds / done as f64 } else { 0.0 };
        let left = (total - done) as f64 * each / working.max(1) as f64;
        let status = |stage: &str| {
            let order = ["copy", "tests", "faults", "close", "ended"];
            let (now, this) = (order.iter().position(|p| *p == phase), order.iter().position(|p| *p == stage));
            match (now, this) {
                (Some(n), Some(t)) if t < n => "ended",
                (Some(n), Some(t)) if t == n => "going on",
                _ => "not begun",
            }
        };
        let first_run = workers.iter().filter_map(|w| w["baseline_s"][0].as_f64()).fold(0.0, f64::max);
        let ended = phase == "ended";
        let functions: Vec<Value> = if ended {
            let listed = self.plan["functions"].as_array().cloned().unwrap_or_default();
            listed
                .iter()
                .map(|f| {
                    json!({"file": f["file"], "function": f["function"], "noticed": f["noticed"], "unnoticed": f["unnoticed"],
                           "not_built": f["other"], "unprotected": f["unprotected"], "reaches_out": f["reaches_out"]["calls"]})
                })
                .collect()
        } else {
            Vec::new()
        };
        let example = faults.first().filter(|_| done > 0).map(|f| {
            json!({"file": f["site"]["file"], "line": f["site"]["start_line"], "what_was_changed": f["operator"], "text": f["site"]["text"],
                   "outcome": match f["class"].as_str() { Some("killed") => "noticed: a test failed", Some("survived") => "not noticed: every test passed", _ => "did not build" }})
        });
        let unprotected = functions.iter().filter(|f| f["unprotected"] == true).count();
        json!({
            "schema": 1, "sequence": sequence, "tool": {"name": "lesscode-core", "version": "2.1.0 (replay of a recorded run)"},
            "command": "step plan", "started": stamp(SystemTime::now() - began.elapsed()), "heartbeat": stamp(SystemTime::now()),
            "ended": ended, "outcome": if ended { json!("The plan is written") } else { Value::Null },
            "outcome_detail": if ended { json!(format!("{unprotected} of {} functions are unprotected.", functions.len())) } else { Value::Null },
            "exit_code": if ended { json!(0) } else { Value::Null },
            "seconds": seconds / working.max(1) as f64 + first_run,
            "subject": {"repository": "D:/Source/drift-rdp", "branch": "master", "commit": "09b4ce8ce2c6a4224ae4754c7bbda6122e2e7ac6", "dirty": false, "on_main_branch": true},
            "copies": {"directory": "D:/BuildArtifacts/lesscode/lesscode-metrics/r164-renderer-baseline/work", "planned": workers.len(), "working": working,
                       "dropped": dropped, "bytes": if ended { json!(13_850_000_000u64) } else { Value::Null }},
            "commands": [["cargo", "nextest", "run", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--lib", "--no-fail-fast"]],
            "writes_in_repository": [],
            "tests": {"command_given": true, "passed_unchanged": if phase == "copy" { Value::Null } else { json!(true) }, "tests_run": 536},
            "stages": [
                {"id": "copy", "title": "Copy the project", "status": status("copy"), "seconds": 4,
                 "what": "Your project is copied three times to a place outside the repository. Everything after this happens in the copies."},
                {"id": "tests", "title": "Run your tests on the unchanged code", "status": status("tests"), "seconds": first_run, "seconds_so_far": first_run / 2.0,
                 "what": "Each copy builds the project from nothing and runs your tests once. This is the slowest stage and has nothing to count. If the tests do not pass here, nothing can be judged."},
                {"id": "faults", "title": "Plant faults", "status": status("faults"), "done": done, "total": total, "seconds_left": left, "seconds": seconds / working.max(1) as f64,
                 "what": "One small fault at a time is put into a copy, the project is built and your tests run. Each fault is then taken out again."},
                {"id": "close", "title": "Check the copies and write the plan", "status": status("close"), "seconds": 9,
                 "what": "Your tests run once more on the unchanged code in each copy, to see that no fault was left behind."}
            ],
            "faults": {"total": total, "done": done, "noticed": noticed, "unnoticed": unnoticed, "not_built": not_built,
                       "not_compiled_here": if ended { json!(64) } else { Value::Null }, "example": example,
                       "files": files.iter().map(|(f, t)| json!({"file": f, "noticed": t[0], "unnoticed": t[1], "not_built": t[2], "left": t[3]})).collect::<Vec<_>>(),
                       "functions": functions},
            "next": if ended { json!([
                "Read the tab Unprotected. In those functions a change can break the code without any test failing.",
                "Write tests for a function before you change it, and run this again to see that they notice its faults.",
                "Remove the copies when you are done: delete the directory named under Your code."]) } else { json!([]) },
        })
    }
}

/// One long answer that carries every new snapshot (an event stream).
fn serve(listener: TcpListener, secret: String, library: String, latest: Arc<Mutex<(u64, String)>>) {
    for stream in listener.incoming().flatten() {
        let (secret, library, latest) = (secret.clone(), library.clone(), Arc::clone(&latest));
        thread::spawn(move || answer(stream, &secret, &library, &latest));
    }
}

fn answer(mut stream: TcpStream, secret: &str, library: &str, latest: &Mutex<(u64, String)>) {
    let mut line = String::new();
    if BufReader::new(&stream).read_line(&mut line).is_err() {
        return;
    }
    let path = line.split_whitespace().nth(1).unwrap_or_default();
    if path == format!("/{secret}/") {
        let body = page(library, &json!({"live": "events", "events": format!("/{secret}/events")}), &Value::Null);
        let _ = write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}", body.len());
    } else if path == format!("/{secret}/events") {
        if stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n").is_err() {
            return;
        }
        let mut sent = u64::MAX;
        loop {
            let (sequence, text) = latest.lock().unwrap().clone();
            if sequence != sent {
                if write!(stream, "data: {text}\n\n").and_then(|()| stream.flush()).is_err() {
                    return;
                }
                sent = sequence;
            }
            thread::sleep(Duration::from_millis(40));
        }
    } else {
        // No secret, no answer worth reading.
        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    }
}

fn replay(args: &[String]) -> Result<Value, String> {
    let [receipt, plan, library, out, rest @ ..] = args else {
        return Err("usage: run-page-proto replay RECEIPT PLAN LIBRARY OUT [--tick-ms N] [--read-ms N] [--step N] [--serve] [--stop-early]".into());
    };
    let number = |name: &str, otherwise: u64| rest.iter().position(|a| a == name).and_then(|i| rest.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(otherwise);
    let (tick, read, step) = (number("--tick-ms", 100), number("--read-ms", 1000), number("--step", 1) as usize);
    let recorded = Recorded::read(receipt, plan)?;
    let library = fs::read_to_string(library).map_err(|e| format!("{library}: {e}"))?;
    let out = Path::new(out);
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    fs::write(out.join("run.html"), page(&library, &json!({"live": "file", "data": "run-data.js", "read_every_ms": read}), &Value::Null)).map_err(|e| e.to_string())?;
    let latest = Arc::new(Mutex::new((0u64, String::from("null"))));
    if rest.iter().any(|a| a == "--serve") {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        let secret = format!("{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() ^ u128::from(std::process::id()) << 64);
        let address = format!("http://{}/{secret}/", listener.local_addr().map_err(|e| e.to_string())?);
        fs::write(out.join("address.txt"), &address).map_err(|e| e.to_string())?;
        let (library, latest) = (library.clone(), Arc::clone(&latest));
        thread::spawn(move || serve(listener, secret, library, latest));
    }
    let began = Instant::now();
    let (mut sequence, mut retries, mut lost, mut rewrites) = (0u64, 0u32, 0u32, 0u32);
    let total = recorded.faults().len();
    let stop_early = rest.iter().any(|a| a == "--stop-early");
    let mut moments: Vec<(usize, &str)> = vec![(0, "copy"), (0, "tests"), (0, "tests"), (0, "tests")];
    moments.extend((0..=total).step_by(step.max(1)).map(|done| (done, "faults")));
    moments.push((total, "faults"));
    moments.extend([(total, "close"), (total, "ended")]);
    let mut last = Value::Null;
    for (done, phase) in moments {
        if stop_early && phase == "close" {
            break;
        }
        sequence += 1;
        last = recorded.snapshot(done, phase, sequence, began);
        rewrites += 1;
        match put(out, &last) {
            Some(again) => retries += again,
            None => lost += 1,
        }
        *latest.lock().unwrap() = (sequence, last.to_string());
        thread::sleep(Duration::from_millis(tick));
    }
    if !stop_early {
        // The report: the same page with the data inside, and nothing beside it needed.
        fs::write(out.join("run.html"), page(&library, &json!({"live": "none"}), &last)).map_err(|e| e.to_string())?;
        thread::sleep(Duration::from_millis(number("--linger-ms", 0)));
    }
    Ok(json!({"rewrites": rewrites, "tried_again": retries, "lost": lost, "report_bytes": fs::metadata(out.join("run.html")).map(|m| m.len()).unwrap_or(0)}))
}

fn fixtures(args: &[String]) -> Result<Value, String> {
    let [receipt, plan, library, out] = args else {
        return Err("usage: run-page-proto fixtures RECEIPT PLAN LIBRARY OUT".into());
    };
    let recorded = Recorded::read(receipt, plan)?;
    let library = fs::read_to_string(library).map_err(|e| format!("{library}: {e}"))?;
    let out = Path::new(out);
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let total = recorded.faults().len();
    let began = Instant::now();
    let ended = recorded.snapshot(total, "ended", 1, began);
    let midway = recorded.snapshot(total / 2, "faults", 1, began);
    let heard = SystemTime::now();
    let at = |seconds: u64| json!({"live": "none", "now": stamp(heard + Duration::from_secs(seconds))});
    let mut going = midway.clone();
    going["heartbeat"] = json!(stamp(heard));
    let mut large = ended.clone();
    large["faults"]["functions"] = (0..2000)
        .map(|i| json!({"file": format!("src/renderer/part_{:02}.rs", i % 20), "function": format!("{}_{i}", "a_function_with_a_very_long_name".repeat(4)).chars().take(120).collect::<String>(),
                        "noticed": i % 5, "unnoticed": i % 3, "not_built": 0, "unprotected": i % 5 == 0, "reaches_out": ["create_surface", "write_texture"]}))
        .collect();
    large["faults"]["files"] = (0..20).map(|i| json!({"file": format!("src/renderer/part_{i:02}.rs"), "noticed": 200, "unnoticed": 100, "not_built": 0, "left": 0})).collect();
    let mut void = ended.clone();
    void["tests"] = json!({"command_given": false});
    void["outcome"] = json!("Could not judge");
    void["outcome_detail"] = json!("No test command was given, so nothing below was measured.");
    void["exit_code"] = json!(2);
    void["faults"] = json!({"total": 0, "done": 0, "noticed": 0, "unnoticed": 0, "not_built": 0, "files": [], "functions": []});
    let mut refused = ended.clone();
    refused["command"] = json!("step check");
    refused["outcome"] = json!("Refused");
    refused["outcome_detail"] = json!("The step is not accepted as it stands.");
    refused["exit_code"] = json!(1);
    refused["report"] = json!({
        "lenses": [
            {"lens": "Size", "measure": "tokens of the file", "before": 4495, "after": 4567, "expected": "may rise by 60", "moved": false, "measured": true},
            {"lens": "State", "measure": "ways the type can be written", "before": 16, "after": 6, "expected": "falls", "moved": true, "measured": true},
            {"lens": "Behaviour kept", "measure": "tests that pass, unchanged", "before": 542, "after": 542, "expected": "stays", "moved": true, "measured": true},
            {"lens": "Protection", "measure": "planted faults unnoticed", "before": "", "after": "", "expected": "does not rise", "measured": false}],
        "refusal": {"reasons": ["the scope grows from 4495 to 4567 tokens; the declaration allows 60 more"]}});
    let pages: Vec<(&str, Value, Value)> = vec![
        ("waiting", json!({"live": "file", "data": "no-such-file.js"}), Value::Null),
        ("running", at(3), going.clone()),
        ("quiet", at(30), going.clone()),
        ("stopped", at(120), going),
        ("ended", json!({"live": "none"}), ended),
        ("cannot-read", json!({"live": "none"}), json!({"schema": 99})),
        ("damaged", json!({"live": "none"}), json!("this is not a run")),
        ("sparse", at(1), json!({"schema": 1, "heartbeat": stamp(heard)})),
        ("large", json!({"live": "none"}), large),
        ("void", json!({"live": "none"}), void),
        ("refused", json!({"live": "none"}), refused),
    ];
    for (name, config, data) in &pages {
        fs::write(out.join(format!("{name}.html")), page(&library, config, data)).map_err(|e| e.to_string())?;
    }
    Ok(json!({"pages": pages.iter().map(|p| p.0).collect::<Vec<_>>()}))
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let outcome = match args.split_first() {
        Some((command, rest)) if command == "replay" => replay(rest),
        Some((command, rest)) if command == "fixtures" => fixtures(rest),
        _ => Err("usage: run-page-proto replay|fixtures ...".into()),
    };
    match outcome {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(3)
        }
    }
}
