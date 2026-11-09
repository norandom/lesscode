//! S2 admission tests: sidecar parsing, binding verification states and the
//! receipt writer, including the fail-closed classes (missing, mismatch,
//! no-bound-sidecar).

use lesscode_core::admission::{self, parse_sidecar, sha256_hex, verify, Status};
use lesscode_core::receipt::{write_receipt, J};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("lc-admission-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn sidecar_parsing() {
    let sidecar = parse_sidecar("62791b3269b843586300f394ce53c88193269ac0d509f0eb0443a5140c805bd3  golden-a.json\r\n# lf-body sha256 0b2e383dd6fbbc243215e24a7844674404331308b3a547239594461c5bea5584 5,479,067 bytes\r\n");
    assert_eq!(
        sidecar.stored.as_deref(),
        Some("62791b3269b843586300f394ce53c88193269ac0d509f0eb0443a5140c805bd3")
    );
    assert_eq!(
        sidecar.declared_lf_body.as_deref(),
        Some("0b2e383dd6fbbc243215e24a7844674404331308b3a547239594461c5bea5584")
    );
    assert!(parse_sidecar("not a sidecar\n").stored.is_none());
    assert!(parse_sidecar("").declared_lf_body.is_none());
}

#[test]
fn binding_states() {
    let root = temp_root("states");
    let write = |rel: &str, bytes: &[u8]| {
        let path = root.join(rel);
        std::fs::write(&path, bytes).expect("write");
        path
    };
    write("good.json", b"{\"a\": 1}\n");
    let good_digest = sha256_hex(b"{\"a\": 1}\n");
    write(
        "good.json.sha256",
        format!("{good_digest}  good.json\n").as_bytes(),
    );
    write("stale.json", b"[1]\n");
    write(
        "stale.json.sha256",
        b"0000000000000000000000000000000000000000000000000000000000000000  stale.json\n",
    );

    let verified = verify(&root, "good.json", &["good.json.sha256".to_string()]);
    assert_eq!(verified.status, Status::Verified);
    assert_eq!(verified.bytes, Some(9));
    assert_eq!(verified.sidecars[0].matches_stored, Some(true));

    let mismatch = verify(&root, "stale.json", &["stale.json.sha256".to_string()]);
    assert_eq!(mismatch.status, Status::Mismatch);

    let missing = verify(&root, "absent.json", &["good.json.sha256".to_string()]);
    assert_eq!(missing.status, Status::Missing);
    assert!(missing.bytes.is_none());

    let unbound = verify(&root, "good.json", &[]);
    assert_eq!(unbound.status, Status::NoBoundSidecar);
    assert_eq!(unbound.sha256_stored.as_deref(), Some(good_digest.as_str()));
}

#[test]
fn receipt_round_trip() {
    let root = temp_root("receipt");
    let report = verify(&root, "x.json", &[]);
    let value = admission::receipt("lesscode-core admission --root .", &[report]);
    let out = root.join("receipt.json");
    admission::write_receipt(&out, &value).expect("write receipt");
    let bytes = std::fs::read(&out).expect("read receipt");
    let text = std::str::from_utf8(&bytes).expect("utf8");
    // Receipt writer: indent=1, insertion order preserved ("receipt" stays first).
    assert!(text.starts_with("{\n \"receipt\": \"s2-artifact-admission-rust-1\""));
    let reparsed: serde_json::Value = serde_json::from_str(text).expect("receipt parses");
    assert_eq!(reparsed["receipt"], "s2-artifact-admission-rust-1");
    assert_eq!(reparsed["command"], "lesscode-core admission --root .");
    assert_eq!(
        reparsed["bindings"][0]["status"],
        serde_json::json!("missing")
    );
    assert_eq!(reparsed["summary"]["total"], 1);
    let sidecar_text = std::fs::read_to_string(root.join("receipt.json.sha256")).expect("sidecar");
    let parsed = parse_sidecar(&sidecar_text);
    assert_eq!(parsed.stored.as_deref(), Some(sha256_hex(&bytes).as_str()));
    assert!(parsed.declared_lf_body.is_none());
}

#[test]
fn cli_admission_end_to_end() {
    let root = temp_root("cli");
    std::fs::write(root.join("a.json"), b"[1, 2]\n").expect("artifact");
    let digest = sha256_hex(b"[1, 2]\n");
    std::fs::write(
        root.join("a.json.sha256"),
        format!("{digest}  a.json\n").as_bytes(),
    )
    .expect("sidecar");
    std::fs::write(root.join("b.json"), b"{}\n").expect("artifact b");
    let bindings = root.join("bindings.json");
    std::fs::write(
        &bindings,
        br#"{"bindings": [{"artifact": "a.json", "sidecars": ["a.json.sha256"]}, {"artifact": "b.json"}]}"#,
    )
    .expect("bindings");
    let out = root.join("out").join("receipt.json");
    let code = admission::cli_run(&root, &bindings, &out).expect("cli run");
    // b.json has bytes but no bound sidecar: not a failure, recorded unbound.
    assert_eq!(code, 0);
    let text = std::fs::read_to_string(&out).expect("receipt");
    let reparsed: serde_json::Value = serde_json::from_str(&text).expect("receipt parses");
    let get = |key: &str| reparsed["summary"][key].as_i64().unwrap();
    assert_eq!(get("total"), 2);
    assert_eq!(get("verified"), 1);
    assert_eq!(get("no_bound_sidecar"), 1);
    assert_eq!(get("mismatch"), 0);
    assert_eq!(get("missing"), 0);
}

#[test]
fn receipt_parity_matches_python_json_dumps() {
    let root = temp_root("parity");
    let value = J::obj(vec![
        ("zebra", J::int(1)),
        ("alpha", J::str("a")),
        ("empty_obj", J::obj(vec![])),
        ("empty_arr", J::Arr(vec![])),
        (
            "nested",
            J::obj(vec![
                (
                    "zz",
                    J::Arr(vec![
                        J::int(1),
                        J::str("two"),
                        J::Null,
                        J::Bool(true),
                        J::Bool(false),
                    ]),
                ),
                (
                    "aa",
                    J::obj(vec![("deep", J::Arr(vec![J::Arr(vec![]), J::obj(vec![])]))]),
                ),
            ]),
        ),
    ]);
    let out = root.join("fresh-dir").join("receipt.json");
    write_receipt(&out, &value).expect("write receipt");
    // Reference bytes: Python json.dumps(value, indent=1, ensure_ascii=True) + "\n".
    // Insertion order is preserved on the non-alphabetical keys at every level.
    assert_eq!(
        std::fs::read_to_string(&out).unwrap(),
        r#"{
 "zebra": 1,
 "alpha": "a",
 "empty_obj": {},
 "empty_arr": [],
 "nested": {
  "zz": [
   1,
   "two",
   null,
   true,
   false
  ],
  "aa": {
   "deep": [
    [],
    {}
   ]
  }
 }
}
"#
    );
}

#[test]
fn receipt_ensure_ascii_escaping_matches_python() {
    let root = temp_root("escapes");
    let value = J::obj(vec![(
        "escapes",
        J::str("quote\" backslash\\ \u{8}\t\n\u{c}\r \u{1f} \u{7f} é \u{1F600}"),
    )]);
    let out = root.join("escapes.json");
    write_receipt(&out, &value).expect("write receipt");
    // Reference bytes: Python json.dumps(..., indent=1, ensure_ascii=True) + "\n":
    // short escapes, lowercase \u00xx for control characters and DEL, \u00e9 for
    // BMP, and the \ud83d\ude00 surrogate pair for U+1F600.
    assert_eq!(
        std::fs::read_to_string(&out).unwrap(),
        "{\n \"escapes\": \"quote\\\" backslash\\\\ \\b\\t\\n\\f\\r \\u001f \\u007f \\u00e9 \\ud83d\\ude00\"\n}\n"
    );
}

#[test]
fn receipt_sha256_sidecar_companion() {
    let root = temp_root("sidecar");
    let value = J::obj(vec![("zebra", J::int(1)), ("alpha", J::Null)]);
    let out = root.join("fresh-dir").join("receipt.json");
    write_receipt(&out, &value).expect("write receipt");
    let bytes = std::fs::read(&out).expect("receipt");
    let sidecar = std::fs::read_to_string(root.join("fresh-dir").join("receipt.json.sha256"))
        .expect("sidecar");
    let parsed = parse_sidecar(&sidecar);
    assert_eq!(parsed.stored.as_deref(), Some(sha256_hex(&bytes).as_str()));
    assert!(parsed.declared_lf_body.is_none());
    assert!(sidecar.ends_with("  receipt.json\n"));
}

#[test]
fn admission_cli_exit_codes_fail_closed() {
    let root = temp_root("exits");
    std::fs::write(root.join("good.json"), b"{\"a\": 1}\n").expect("write");
    let digest = sha256_hex(b"{\"a\": 1}\n");
    std::fs::write(
        root.join("good.json.sha256"),
        format!("{digest}  good.json\n").as_bytes(),
    )
    .expect("write");
    std::fs::write(root.join("stale.json"), b"[1]\n").expect("write");
    std::fs::write(
        root.join("stale.json.sha256"),
        b"0000000000000000000000000000000000000000000000000000000000000000  stale.json\n",
    )
    .expect("write");
    std::fs::write(root.join("unbound.json"), b"[]\n").expect("write");

    let bindings = root.join("bindings-all.json");
    std::fs::write(
        &bindings,
        br#"{"bindings": [
            {"artifact": "good.json", "sidecars": ["good.json.sha256"]},
            {"artifact": "stale.json", "sidecars": ["stale.json.sha256"]},
            {"artifact": "absent.json", "sidecars": ["absent.json.sha256"]},
            {"artifact": "unbound.json"}]}"#,
    )
    .expect("bindings");
    let out = root.join("all-receipt.json");
    // Mismatch and missing bindings fail closed with exit code 1.
    assert_eq!(
        admission::cli_run(&root, &bindings, &out).expect("cli run"),
        1
    );
    let all: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    let get = |key: &str| all["summary"][key].as_i64().unwrap();
    assert_eq!(get("total"), 4);
    assert_eq!(get("verified"), 1);
    assert_eq!(get("mismatch"), 1);
    assert_eq!(get("missing"), 1);
    assert_eq!(get("no_bound_sidecar"), 1);

    // Verified plus unbound-only bindings pass under default admission: exit 0
    // (admission has no --require-sidecar mode; exit codes are 0/1/3 only).
    let ok = root.join("bindings-ok.json");
    std::fs::write(
        &ok,
        br#"{"bindings": [
            {"artifact": "good.json", "sidecars": ["good.json.sha256"]},
            {"artifact": "unbound.json"}]}"#,
    )
    .expect("bindings");
    let out_ok = root.join("ok-receipt.json");
    assert_eq!(admission::cli_run(&root, &ok, &out_ok).expect("cli run"), 0);
    let passed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out_ok).unwrap()).unwrap();
    assert_eq!(passed["summary"]["verified"], 1);
    assert_eq!(passed["summary"]["no_bound_sidecar"], 1);
}

#[test]
fn historical_s2_bindings_drift_reproduces_counts() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf();
    let bindings = repo.join("docs/experiments/deconvolution/native-prototype/s2-bindings.json");
    if !bindings.exists() {
        eprintln!("s2-bindings.json not found at {bindings:?}; skipping historical drift test");
        return;
    }
    let out = temp_root("s2-drift").join("s2-admission.json");
    // R28 smoke through the single binary: historical drift in the frozen
    // s2 bindings must reproduce the recorded counts and exit 1.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
        .arg("admission")
        .arg("--root")
        .arg(&repo)
        .arg("--bindings")
        .arg(&bindings)
        .arg("--out")
        .arg(&out)
        .output()
        .expect("run lesscode-core admission");
    assert_eq!(output.status.code(), Some(1));
    let receipt: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    let get = |key: &str| receipt["summary"][key].as_i64().unwrap();
    assert_eq!(get("total"), 21);
    assert_eq!(get("verified"), 11);
    assert_eq!(get("mismatch"), 5);
    assert_eq!(get("missing"), 1);
    assert_eq!(get("no_bound_sidecar"), 4);
}
