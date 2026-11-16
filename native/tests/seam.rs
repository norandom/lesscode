//! The launcher (`seam.rs`, spec `model-seam` task 7.1): `lesscode-model` is
//! found, given one request and read for one answer. A scripted stand-in
//! takes its place; no model is called and nothing goes over a network.
//!
//! The launcher reads its executable from the environment, so each test sets
//! the variables it needs under one lock and puts them back (cargo-nextest
//! runs each test in a process of its own as well).

mod seam_support;

use lesscode_core::jsonval::{j_get, J};
use lesscode_core::seam::{call_through_seam, SeamReply};
use seam_support::{answered, failed, scripted, Script};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static ENV: Mutex<()> = Mutex::new(());

/// Runs `f` with these variables set (`Some`) or removed (`None`), and puts
/// back what was there before.
fn with_env<T>(vars: &[(&str, Option<OsString>)], f: impl FnOnce() -> T) -> T {
    let _lock = ENV.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let saved: Vec<(&str, Option<OsString>)> = vars
        .iter()
        .map(|(k, _)| (*k, std::env::var_os(k)))
        .collect();
    let set = |pairs: &[(&str, Option<OsString>)]| {
        for (key, value) in pairs {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    };
    set(vars);
    let out = f();
    set(&saved);
    out
}

fn dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lc-seam-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// One call through the launcher with the stand-in as `lesscode-model`.
fn call_scripted(
    script: &Script,
    endpoint_file: Option<&Path>,
    temperature: Option<f64>,
) -> Result<SeamReply, String> {
    let vars = [
        ("LESSCODE_MODEL_BIN", Some(scripted().into_os_string())),
        (
            "LESSCODE_SCRIPTED_MODEL",
            Some(script.0.clone().into_os_string()),
        ),
    ];
    with_env(&vars, || {
        call_through_seam(
            endpoint_file,
            "system text",
            "user text",
            512,
            temperature,
            None,
        )
    })
}

#[test]
fn an_answered_call_gives_the_text_and_the_rest_of_the_reply() {
    let work = dir("answered");
    let script = Script::new(&work, "answered", &answered("```json\n{}\n```"), "", 0);
    let endpoint = work.join("model.env");
    let reply = call_scripted(&script, Some(&endpoint), None).unwrap();

    assert_eq!(
        reply.text, "```json\n{}\n```",
        "the text as the model wrote it"
    );
    assert_eq!(j_get(&reply.meta, "stop"), Some(&J::str("complete")));
    assert_eq!(j_get(&reply.meta, "output_tokens"), Some(&J::int(30)));
    let sent = j_get(&reply.meta, "sent").expect("what was sent");
    assert_eq!(j_get(sent, "model"), Some(&J::str("scripted-model")));
    assert!(j_get(&reply.meta, "text").is_none());

    let (args, request) = script.seen();
    let endpoint = endpoint.to_string_lossy().into_owned();
    assert_eq!(args, ["call", "--endpoint-env", endpoint.as_str()]);
    assert_eq!(request["protocol"], 1);
    assert_eq!(request["system"], "system text");
    assert_eq!(request["user"], "user text");
    assert_eq!(request["max_output_tokens"], 512);
    assert!(request["purpose"].is_string());
    assert!(request.get("temperature").is_none(), "{request}");
    assert!(request.get("structure").is_none(), "{request}");
    let _ = fs::remove_dir_all(&work);
}

#[test]
fn a_stated_temperature_is_part_of_the_request_and_no_endpoint_file_is_named_unless_given() {
    let work = dir("temperature");
    let script = Script::new(&work, "temperature", &answered("{}"), "", 0);
    call_scripted(&script, None, Some(0.3)).unwrap();

    let (args, request) = script.seen();
    assert_eq!(args, ["call"]);
    assert_eq!(request["temperature"], 0.3);
    let _ = fs::remove_dir_all(&work);
}

#[test]
fn a_failed_call_gives_the_kind_and_the_reason_of_the_failure() {
    let work = dir("failed");
    let reason = "localhost refused the call (status 403): not allowed";
    let script = Script::new(&work, "failed", &failed("refused", reason), reason, 3);
    let error = call_scripted(&script, None, None).unwrap_err();

    assert!(error.contains("lesscode-model"), "{error}");
    assert!(error.contains("(refused)"), "{error}");
    assert!(error.contains(reason), "{error}");
    let _ = fs::remove_dir_all(&work);
}

#[test]
fn an_answer_that_cannot_be_read_is_an_error_that_says_so() {
    let work = dir("unreadable");
    let cases = [
        ("not-json", "this is not JSON".to_owned(), 0, "not JSON"),
        ("nothing", String::new(), 2, "not JSON"),
        (
            "protocol",
            answered("{}").replace("\"protocol\":1", "\"protocol\":2"),
            0,
            "protocol 2",
        ),
        (
            "no-reply",
            r#"{"protocol": 1, "ok": true}"#.to_owned(),
            0,
            "neither",
        ),
    ];
    for (name, stdout, exit, says) in cases {
        let script = Script::new(&work, name, &stdout, "", exit);
        let error = call_scripted(&script, None, None).unwrap_err();
        assert!(error.contains("cannot be read"), "{name}: {error}");
        assert!(error.contains(says), "{name}: {error}");
        assert!(error.contains(&exit.to_string()), "{name}: {error}");
    }
    let _ = fs::remove_dir_all(&work);
}

#[test]
fn a_missing_executable_says_what_needs_it_and_where_it_was_looked_for() {
    let work = dir("missing");
    let named = work.join("no-such-lesscode-model");
    let error = with_env(
        &[("LESSCODE_MODEL_BIN", Some(named.clone().into()))],
        || call_through_seam(None, "s", "u", 64, None, None).unwrap_err(),
    );
    assert!(error.contains("need lesscode-model"), "{error}");
    assert!(error.contains("LESSCODE_MODEL_BIN"), "{error}");
    assert!(error.contains(&named.display().to_string()), "{error}");

    let empty = work.join("empty");
    fs::create_dir_all(&empty).unwrap();
    let vars = [
        ("LESSCODE_MODEL_BIN", None),
        ("PATH", Some(empty.into_os_string())),
    ];
    let error = with_env(&vars, || {
        call_through_seam(None, "s", "u", 64, None, None).unwrap_err()
    });
    let beside = std::env::current_exe().unwrap();
    let beside = beside.parent().unwrap().display().to_string();
    assert!(error.contains("need lesscode-model"), "{error}");
    assert!(error.contains("LESSCODE_MODEL_BIN is not set"), "{error}");
    assert!(error.contains(&beside), "{error}");
    assert!(error.contains("search path"), "{error}");
    let _ = fs::remove_dir_all(&work);
}

#[test]
fn the_executable_is_found_on_the_search_path() {
    let work = dir("search-path");
    let on_path = work.join("bin");
    fs::create_dir_all(&on_path).unwrap();
    let name = format!("lesscode-model{}", std::env::consts::EXE_SUFFIX);
    fs::copy(scripted(), on_path.join(name)).unwrap();
    let script = Script::new(&work, "search-path", &answered("found"), "", 0);
    let vars = [
        ("LESSCODE_MODEL_BIN", None),
        (
            "LESSCODE_SCRIPTED_MODEL",
            Some(script.0.clone().into_os_string()),
        ),
        ("PATH", Some(on_path.into_os_string())),
    ];
    let reply = with_env(&vars, || call_through_seam(None, "s", "u", 64, None, None));

    assert_eq!(reply.unwrap().text, "found");
    assert!(script.was_called());
    drop(script);
    let _ = fs::remove_dir_all(&work);
}
