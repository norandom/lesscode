//! A stand-in for `lesscode-model call`, used by the tests of the launcher
//! (`src/seam.rs`) and of `regenerate run`. It needs no network and behaves
//! the same on every platform.
//!
//!     scripted_model call [--endpoint-env FILE]
//!
//! What it does comes from the JSON file named by `LESSCODE_SCRIPTED_MODEL`:
//! `stdout` is written to standard output as it is, `stderr` to standard
//! error, and `exit` is the exit code (0 when absent). It first reads the
//! request on standard input and writes it with its arguments, as
//! `{"args": [...], "request": "..."}`, to the script's path with `.seen`
//! appended. Exit 2 when the script cannot be used.

use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(script) = std::env::var_os("LESSCODE_SCRIPTED_MODEL").map(PathBuf::from) else {
        eprintln!("scripted_model: LESSCODE_SCRIPTED_MODEL names no script");
        return ExitCode::from(2);
    };
    let read = std::fs::read(&script).map_err(|e| e.to_string());
    let Ok(said) = read.and_then(|bytes| {
        serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e| e.to_string())
    }) else {
        eprintln!("scripted_model: {} cannot be read", script.display());
        return ExitCode::from(2);
    };
    let mut request = String::new();
    if std::io::stdin().read_to_string(&mut request).is_err() {
        eprintln!("scripted_model: standard input cannot be read");
        return ExitCode::from(2);
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seen = serde_json::json!({"args": args, "request": request});
    let mut seen_path = script.into_os_string();
    seen_path.push(".seen");
    if std::fs::write(&seen_path, seen.to_string()).is_err() {
        eprintln!("scripted_model: what was seen cannot be written");
        return ExitCode::from(2);
    }
    if let Some(text) = said["stderr"].as_str() {
        eprint!("{text}");
    }
    if let Some(text) = said["stdout"].as_str() {
        print!("{text}");
    }
    ExitCode::from(said["exit"].as_u64().map_or(0, |code| code as u8))
}
