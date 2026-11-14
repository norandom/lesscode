//! The command line of `lesscode-model`.

use lesscode_model::check::check;
use lesscode_model::measure::{Measured, QUICK_TRIALS, TRIALS, measure};
use lesscode_model::protocol::{EXIT_USAGE, answer, exit_code, read_request, unusable};
use lesscode_model::repair::REPAIRS;
use lesscode_model::taskset::{Scratch, built_in, load};
use lesscode_model::{CallRequest, Limits, Notice, Terminal, call, endpoint_from, host_of};
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: lesscode-model call [--endpoint-env FILE] [--record PATH] \
                     [--max-wait SECONDS] [--answer-limit SECONDS]\n\
                     \x20 reads one request as JSON on standard input and writes one answer as \
                     JSON on standard output\n\
                     usage: lesscode-model check [--endpoint-env FILE] [--max-wait SECONDS]\n\
                     \x20 sends one small fixed task and says whether the endpoint can be used\n\
                     usage: lesscode-model measure [--endpoint-env FILE] [--trials N] [--quick] \
                     [--task-set DIR] [--out FILE]\n\
                     \x20 takes a task set through its check and says whether the model is good \
                     enough, direct or with the repair loop";

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

fn whole(name: &str, value: Option<&String>, otherwise: u64) -> Result<u64, String> {
    value.map_or(Ok(otherwise), |text| {
        text.parse()
            .map_err(|_| format!("{name} needs a whole number, got {text}"))
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
        limits.wait.total_seconds = whole(names[2], values[2].as_ref(), limits.wait.total_seconds)?;
        limits.answer_seconds = whole(names[3], values[3].as_ref(), limits.answer_seconds)?;
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
        limits.wait.total_seconds = whole(names[1], values[1].as_ref(), limits.wait.total_seconds)?;
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

/// What `measure` was asked for: the endpoint file, the trials, whether in
/// the quick mode, a task set in place of the built-in one, a file for the
/// report as JSON.
struct MeasureAsked {
    endpoint_file: Option<PathBuf>,
    trials: u32,
    quick: bool,
    task_set: Option<PathBuf>,
    out: Option<PathBuf>,
}

fn measure_asked(args: &[String]) -> Result<MeasureAsked, String> {
    let quick = args.iter().any(|arg| arg == "--quick");
    let paired: Vec<String> = args
        .iter()
        .filter(|arg| *arg != "--quick")
        .cloned()
        .collect();
    let names = ["--endpoint-env", "--trials", "--task-set", "--out"];
    let values = options(&paired, &names)?;
    if quick && values[1].is_some() {
        return Err(format!(
            "--quick runs {QUICK_TRIALS} trials; state --trials or --quick"
        ));
    }
    let otherwise = if quick { QUICK_TRIALS } else { TRIALS };
    let trials = whole(names[1], values[1].as_ref(), u64::from(otherwise))?;
    let trials = u32::try_from(trials)
        .ok()
        .filter(|trials| *trials > 0)
        .ok_or("--trials needs a number from 1")?;
    let path = |at: usize| values[at].clone().map(PathBuf::from);
    Ok(MeasureAsked {
        endpoint_file: path(0),
        trials,
        quick,
        task_set: path(2),
        out: path(3),
    })
}

/// `measure`: trials over a task set through the repair loop, the counts
/// and the verdict.
fn measure_command(args: &[String]) -> ExitCode {
    let asked = match measure_asked(args) {
        Ok(asked) => asked,
        Err(problem) => return usage(&problem),
    };
    let endpoint = match endpoint_from(asked.endpoint_file.as_deref()) {
        Ok(endpoint) => endpoint,
        Err(error) => {
            Terminal.failed(&error.reason());
            return ExitCode::from(EXIT_USAGE);
        }
    };
    let loaded = match &asked.task_set {
        Some(dir) => Scratch::new().and_then(|scratch| load(dir, &dir.to_string_lossy(), scratch)),
        None => built_in(),
    };
    let (set, mut core) = match loaded {
        Ok(loaded) => loaded,
        Err(problem) => return usage(&format!("the task set cannot be used: {problem}")),
    };

    let limits = Limits::default();
    let mut through = |request: &CallRequest| call(&endpoint, request, &limits, &Terminal, None);
    let counts = measure(
        &mut through,
        &set,
        &mut |task, text| core.judge(task, text),
        asked.trials,
        REPAIRS,
    );
    let measured = Measured {
        task_set: set.name,
        check: set.check_name,
        model: endpoint.model.clone(),
        host: host_of(&endpoint),
        repairs: REPAIRS,
        quick: asked.quick,
        counts,
    };
    println!("{}", measured.lines().join("\n"));
    if let Some(out) = &asked.out
        && let Err(problem) = std::fs::write(out, format!("{:#}\n", measured.json()))
    {
        eprintln!(
            "lesscode-model: {} cannot be written: {problem}",
            out.display()
        );
        return ExitCode::from(EXIT_USAGE);
    }
    ExitCode::from(measured.verdict().exit())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("call") => call_command(&args[1..]),
        Some("check") => check_command(&args[1..]),
        Some("measure") => measure_command(&args[1..]),
        Some(other) => usage(&format!("unknown command {other}")),
        None => usage("no command"),
    }
}
