//! S2 admission tests: sidecar parsing, binding verification states and the
//! receipt writer, including the fail-closed classes (missing, mismatch,
//! no-bound-sidecar).

use lesscode_core::admission::{self, parse_sidecar, verify, Status};
use lesscode_core::pyjson::{self, J};
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
    let good_digest = pyjson::sha256_hex(b"{\"a\": 1}\n");
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
    let reparsed = pyjson::parse(text).expect("receipt parses");
    assert_eq!(reparsed, value, "receipt round-trips through the formatter");
    let sidecar_text = std::fs::read_to_string(root.join("receipt.json.sha256")).expect("sidecar");
    let parsed = parse_sidecar(&sidecar_text);
    assert_eq!(
        parsed.stored.as_deref(),
        Some(pyjson::sha256_hex(&bytes).as_str())
    );
    assert!(parsed.declared_lf_body.is_none());
}

#[test]
fn cli_admission_end_to_end() {
    let root = temp_root("cli");
    std::fs::write(root.join("a.json"), b"[1, 2]\n").expect("artifact");
    let digest = pyjson::sha256_hex(b"[1, 2]\n");
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
    let value = pyjson::parse(&text).expect("receipt parses");
    let J::Obj(pairs) = &value else {
        panic!("object")
    };
    let summary = pairs
        .iter()
        .find(|(k, _)| k == "summary")
        .and_then(|(_, v)| match v {
            J::Obj(fields) => Some(fields.clone()),
            _ => None,
        })
        .expect("summary");
    let get = |key: &str| {
        summary
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| match v {
                J::Int(n) => n.parse::<i64>().unwrap(),
                other => panic!("{key}: {other:?}"),
            })
            .unwrap()
    };
    assert_eq!(get("total"), 2);
    assert_eq!(get("verified"), 1);
    assert_eq!(get("no_bound_sidecar"), 1);
    assert_eq!(get("mismatch"), 0);
    assert_eq!(get("missing"), 0);
}
