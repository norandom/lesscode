//! Dedicated S10 turn-check binary (keeps the main CLI inside its registration budget).

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut flag = |name: &str| {
        args.iter().position(|a| a == name).map(|i| {
            let value = args.get(i + 1).cloned().unwrap_or_default();
            args.drain(i..=i + 1);
            value
        })
    };
    let (Some(turns), Some(out)) = (flag("--turns"), flag("--out")) else {
        eprintln!("usage: lesscode-turncheck --turns DESCRIPTOR --out RECEIPT");
        return ExitCode::from(3);
    };
    match lesscode_core::turncheck::cli_run(&PathBuf::from(turns), &PathBuf::from(out)) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(3)
        }
    }
}
