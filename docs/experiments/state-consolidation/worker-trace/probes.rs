//! Compiled into the disposable historical library. The original operation
//! runs once, with the original inputs; its return value goes to its caller.
use crate::specimen_runner::sha256_hex_bytes;
use serde_json::{json, Value};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

fn emit(site: &str, op: &str, data: Value) {
    static SEQUENCE: Mutex<u64> = Mutex::new(0);
    let mut sequence = SEQUENCE.lock().expect("trace writer");
    *sequence += 1;
    let path = std::env::var_os("LC_WORKER_TRACE_FILE").expect("missing trace destination");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open trace");
    writeln!(
        file,
        "{}",
        json!({"sequence":*sequence,"site":site,"op":op,"data":data})
    )
    .expect("write trace");
}
fn absolute(path: &Path) -> PathBuf {
    path.canonicalize().expect("trace path must exist")
}
pub fn read(site: &str, path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let bytes = fs::read(&path)?;
    emit(
        site,
        "read_test",
        json!({"path":absolute(path.as_ref()),"hash":sha256_hex_bytes(&bytes)}),
    );
    Ok(bytes)
}
pub fn copy(site: &str, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<u64> {
    let count = fs::copy(&from, &to)?;
    emit(
        site,
        "copy",
        json!({"from":absolute(from.as_ref()),"to":absolute(to.as_ref()),"bytes":count}),
    );
    Ok(count)
}
pub fn digest(
    site: &str,
    result: Result<Option<String>, String>,
) -> Result<Option<String>, String> {
    if let Ok(value) = &result {
        emit(site, "test_digest", json!({"hash":value}));
    }
    result
}
pub fn identity(site: &str, value: String) -> String {
    emit(
        site,
        "identity",
        json!({"hash":sha256_hex_bytes(value.as_bytes())}),
    );
    value
}
pub fn key(site: &str, identity: &str, value: String) -> String {
    emit(
        site,
        "key",
        json!({"hash":value,"identity":sha256_hex_bytes(identity.as_bytes())}),
    );
    value
}
pub fn lookup<T>(site: &str, key: &str, value: Option<T>) -> Option<T> {
    emit(site, "lookup", json!({"hash":key,"hit":value.is_some()}));
    value
}
pub fn workspace<T, E>(site: &str, op: &str, value: Result<T, E>) -> Result<T, E> {
    emit(site, op, json!({"ok":value.is_ok()}));
    value
}
pub fn tree<'a>(site: &str, path: &'a Path) -> &'a Path {
    emit(site, "mutant_tree", json!({"cwd":absolute(path)}));
    path
}
pub fn cwd<'a>(site: &str, command: &[String], path: &'a Path) -> &'a Path {
    emit(
        site,
        "command_dir",
        json!({"cwd":absolute(path),"argv":command}),
    );
    path
}
