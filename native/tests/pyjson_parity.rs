//! S2 parity tests: the Rust formatter must reproduce the Python apparatus
//! byte for byte on the generated fixtures and on the frozen golden traces.

use lesscode_core::admission;
use lesscode_core::pyjson::{self, Fmt, J};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("native/ has a parent")
        .to_path_buf()
}
fn fixture_dir() -> PathBuf {
    repo_root().join("native/tests/fixtures/pyjson")
}
fn read_bytes(path: &PathBuf) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn expected_for(case: &str) -> (String, u64) {
    let text = std::fs::read_to_string(fixture_dir().join("expected.json")).expect("expected.json");
    let value = pyjson::parse(&text).expect("expected.json parses");
    let J::Obj(pairs) = &value else {
        panic!("expected.json is an object")
    };
    let cases = pairs
        .iter()
        .find(|(k, _)| k == "cases")
        .and_then(|(_, v)| match v {
            J::Arr(items) => Some(items.clone()),
            _ => None,
        })
        .expect("cases array");
    for case_value in cases {
        let J::Obj(fields) = &case_value else {
            continue;
        };
        let get = |key: &str| {
            fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        };
        if get("case") == Some(J::str(case)) {
            let digest = match get("sha256_lf_body") {
                Some(J::Str(s)) => s,
                other => panic!("digest for {case}: {other:?}"),
            };
            let bytes = match get("bytes") {
                Some(J::Int(n)) => n.parse::<u64>().expect("bytes"),
                other => panic!("bytes for {case}: {other:?}"),
            };
            return (digest, bytes);
        }
    }
    panic!("case {case} missing from expected.json");
}

fn assert_fixture_parity(case: &str, fmt: Fmt) {
    let (expected_digest, expected_bytes) = expected_for(case);
    let raw = read_bytes(&fixture_dir().join(format!("{case}.json")));
    assert_eq!(raw.len() as u64, expected_bytes, "{case}: stored size");
    let text = std::str::from_utf8(&raw).expect("fixture is UTF-8");
    let value = pyjson::parse(text).unwrap_or_else(|e| panic!("{case} parse: {e}"));
    let mut body = String::new();
    pyjson::write(&value, fmt, &mut body);
    assert_eq!(
        body.as_bytes(),
        &raw[..],
        "{case}: byte-identical re-emission"
    );
    assert_eq!(
        pyjson::sha256_hex(body.as_bytes()),
        expected_digest,
        "{case}: digest"
    );
    let round = pyjson::parse(&body).expect("re-emission reparses");
    assert_eq!(round, value, "{case}: round-trip value equality");
}

#[test]
fn fixtures_reproduce_python_bytes() {
    assert_fixture_parity("canon_compact", Fmt::Canon);
    assert_fixture_parity("default_compact", Fmt::DefaultCompact);
    assert_fixture_parity(
        "golden_indent0",
        Fmt::Indent {
            width: 0,
            sort_keys: true,
        },
    );
    assert_fixture_parity(
        "receipt_indent1",
        Fmt::Indent {
            width: 1,
            sort_keys: false,
        },
    );
}

#[test]
fn python_float_repr_rules() {
    let emit = |x: f64| {
        let mut out = String::new();
        pyjson::write(&J::f64(x), Fmt::Canon, &mut out);
        out
    };
    assert_eq!(emit(-0.0), "-0.0");
    assert_eq!(emit(0.0), "0.0");
    assert_eq!(emit(1.0), "1.0");
    assert_eq!(emit(100.0), "100.0");
    assert_eq!(emit(0.0001), "0.0001");
    assert_eq!(emit(1e-07), "1e-07");
    assert_eq!(emit(1e16), "1e+16");
    assert_eq!(emit(1.5e23), "1.5e+23");
    assert_eq!(emit(1e15), "1000000000000000.0");
    assert_eq!(emit(1e100), "1e+100");
    assert_eq!(emit(f64::NAN), "NaN");
    assert_eq!(emit(f64::INFINITY), "Infinity");
    assert_eq!(emit(f64::NEG_INFINITY), "-Infinity");
    assert_eq!(emit(0.1), "0.1");
    assert_eq!(emit(1.002), "1.002");
}

/// The decisive parity proof: re-emit the frozen E4 golden trace (5.6 MB as
/// stored) and reproduce the recorder's declared LF-body digest.
#[test]
fn golden_e4_trace_digest_parity() {
    for name in ["golden-a.json", "held-out-a.json"] {
        let artifact = repo_root().join(".runs/e4-lifecycle").join(name);
        if !artifact.is_file() {
            eprintln!("skip: {} absent", artifact.display());
            continue;
        }
        let sidecar_text = std::fs::read_to_string(
            repo_root()
                .join(".runs/e4-lifecycle")
                .join(format!("{name}.sha256")),
        )
        .unwrap_or_else(|e| panic!("sidecar for {name}: {e}"));
        let sidecar = admission::parse_sidecar(&sidecar_text);
        let stored_expected = sidecar.stored.expect("sidecar stored digest");
        let declared = sidecar
            .declared_lf_body
            .expect("sidecar declared LF-body digest");
        let bytes = read_bytes(&artifact);
        assert_eq!(
            pyjson::sha256_hex(&bytes),
            stored_expected,
            "{name}: bytes as stored match the sidecar"
        );
        // Declared normalization: the recorder hashed the LF body.
        let mut lf_bytes = Vec::with_capacity(bytes.len());
        for i in 0..bytes.len() {
            if bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
                continue;
            }
            lf_bytes.push(bytes[i]);
        }
        let text = std::str::from_utf8(&lf_bytes).expect("golden trace is UTF-8");
        let value = pyjson::parse(text).unwrap_or_else(|e| panic!("{name} parse: {e}"));
        let body = {
            let mut s = String::new();
            pyjson::write(
                &value,
                Fmt::Indent {
                    width: 0,
                    sort_keys: true,
                },
                &mut s,
            );
            s
        };
        assert_eq!(
            pyjson::sha256_hex(body.as_bytes()),
            declared,
            "{name}: Rust re-emission reproduces the Python recorder digest"
        );
    }
}
