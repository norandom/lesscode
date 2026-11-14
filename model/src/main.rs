//! The command line of `lesscode-model`.

use lesscode_model::check::check;
use lesscode_model::protocol::{EXIT_USAGE, answer, exit_code, read_request, unusable};
use lesscode_model::{CallRequest, Limits, Notice, Terminal, call, endpoint_from};
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: lesscode-model call [--endpoint-env FILE] [--record PATH] \
                     [--max-wait SECONDS] [--answer-limit SECONDS]\n\
                     \x20 reads one request as JSON on standard input and writes one answer as \
                     JSON on standard output\n\
                     usage: lesscode-model check [--endpoint-env FILE] [--max-wait SECONDS]\n\
                     \x20 sends one small fixed task and says whether the endpoint can be used";

/// The value after each `--name` of `names`, in their order. Anything else
/// on the command line is an error that names it.
fn options(args: &[String], names: &[&str]) -> Result<Vec<Option<String>>, String> {
    let mut values = vec![None; names.len()];
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        let at = names
            .iter()
            .position(|name| arg == name)
            .ok_or_else(|| format!("unknown argument {arg}"))?;
        let value = rest.next().ok_or_else(|| format!("{arg} needs a value"))?;
        values[at] = Some(value.clone());
    }
    Ok(values)
}

fn seconds(name: &str, value: Option<&String>, otherwise: u64) -> Result<u64, String> {
    value.map_or(Ok(otherwise), |text| {
        text.parse()
            .map_err(|_| format!("{name} needs whole seconds, got {text}"))
    })
}

fn usage(problem: &str) -> ExitCode {
    eprintln!("lesscode-model: {problem}\n{USAGE}");
    ExitCode::from(EXIT_USAGE)
}

/// `call`: one request from standard input through the seam.
fn call_command(args: &[String]) -> ExitCode {
    let names = ["--endpoint-env", "--record", "--max-wait", "--answer-limit"];
    let mut limits = Limits::default();
    let stated = options(args, &names).and_then(|values| {
        limits.wait.total_seconds =
            seconds(names[2], values[2].as_ref(), limits.wait.total_seconds)?;
        limits.answer_seconds = seconds(names[3], values[3].as_ref(), limits.answer_seconds)?;
        Ok((
            values[0].clone().map(PathBuf::from),
            values[1].clone().map(PathBuf::from),
        ))
    });
    let (endpoint_file, record) = match stated {
        Ok(paths) => paths,
        Err(problem) => return usage(&problem),
    };

    let mut text = String::new();
    let request = std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|problem| format!("standard input cannot be read: {problem}"))
        .and_then(|_| read_request(&text));
    let request = match request {
        Ok(request) => request,
        Err(problem) => {
            println!("{}", unusable(&problem));
            return usage(&problem);
        }
    };

    let outcome = endpoint_from(endpoint_file.as_deref())
        .inspect_err(|error| Terminal.failed(&error.reason()))
        .and_then(|endpoint| call(&endpoint, &request, &limits, &Terminal, record.as_deref()));
    println!("{}", answer(&outcome));
    ExitCode::from(exit_code(&outcome))
}

/// `check`: the fixed task through the repair loop, and a report.
fn check_command(args: &[String]) -> ExitCode {
    let names = ["--endpoint-env", "--max-wait"];
    let mut limits = Limits::default();
    let stated = options(args, &names).and_then(|values| {
        limits.wait.total_seconds =
            seconds(names[1], values[1].as_ref(), limits.wait.total_seconds)?;
        Ok(values[0].clone().map(PathBuf::from))
    });
    let endpoint_file = match stated {
        Ok(path) => path,
        Err(problem) => return usage(&problem),
    };

    let endpoint = endpoint_from(endpoint_file.as_deref())
        .inspect_err(|error| Terminal.failed(&error.reason()));
    let mut through = |request: &CallRequest| match &endpoint {
        Ok(endpoint) => call(endpoint, request, &limits, &Terminal, None),
        Err(error) => Err(error.clone()),
    };
    match check(&mut through) {
        Ok(report) => {
            println!("{}", report.lines.join("\n"));
            ExitCode::from(report.exit)
        }
        Err(problem) => {
            eprintln!("lesscode-model: the check needs a temporary directory: {problem}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("call") => call_command(&args[1..]),
        Some("check") => check_command(&args[1..]),
        Some(other) => usage(&format!("unknown command {other}")),
        None => usage("no command"),
    }
}
