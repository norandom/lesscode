//! A stand-in for a project's test command, used by the driver's run tests.
//! It needs no network and behaves the same on every platform.
//!
//!     toy_check --file F --expect TEXT [--expect TEXT ...]
//!               [--expect-from FILE] [--print TEXT] [--print-if-missing TEXT]
//!               [--sleep-if-missing-ms N] [--missing-exit CODE] [--poison MARKER]
//!
//! `--expect-from` reads further expected texts, one per line, from a file:
//! the stand-in for a project's test files. Exit 2 when it cannot be read.
//!
//! Exit 0 when the file contains every expected text; otherwise the exit
//! code given (1 by default), after the optional output and sleep.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let values = |flag: &str| -> Vec<&String> {
        let pairs = args.iter().zip(args.iter().skip(1));
        pairs
            .filter(|(name, _)| *name == flag)
            .map(|(_, value)| value)
            .collect()
    };
    let one = |flag: &str| values(flag).first().map(|v| v.as_str());
    let Some(file) = one("--file") else {
        eprintln!("toy_check: --file is required");
        return ExitCode::from(2);
    };
    let Ok(text) = std::fs::read_to_string(file) else {
        eprintln!("toy_check: cannot read {file}");
        return ExitCode::from(2);
    };
    if let Some(always) = one("--print") {
        println!("{always}");
    }
    // A check that leaves something behind when it fails, and fails from
    // then on: what a build cache shared between copies does to a real run.
    let poison = one("--poison");
    if poison.is_some_and(|marker| std::path::Path::new(marker).exists()) {
        println!("poisoned by an earlier run");
        return ExitCode::from(1);
    }
    let mut expected: Vec<String> = values("--expect").into_iter().cloned().collect();
    for list in values("--expect-from") {
        let Ok(lines) = std::fs::read_to_string(list) else {
            eprintln!("toy_check: cannot read {list}");
            return ExitCode::from(2);
        };
        expected.extend(
            lines
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(str::to_owned),
        );
    }
    if expected.iter().all(|e| text.contains(e.as_str())) {
        return ExitCode::SUCCESS;
    }
    if let Some(marker) = poison {
        let _ = std::fs::write(marker, "");
    }
    if let Some(note) = one("--print-if-missing") {
        println!("{note}");
    }
    if let Some(ms) = one("--sleep-if-missing-ms").and_then(|v| v.parse().ok()) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
    let code = one("--missing-exit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    ExitCode::from(code)
}
