//! The check command: is this endpoint usable. One small fixed task goes to
//! the configured model through the repair loop, and every reply is judged
//! by the core's own parsing and static checks on a copy in a temporary
//! directory. Nothing is written anywhere else.

use crate::error::CallError;
use crate::protocol::{EXIT_ANSWERED, EXIT_FAILED, EXIT_REJECTED, EXIT_USAGE};
use crate::repair::{Outcome, REPAIRS, Verdict, with_repairs};
use crate::request::{CallRequest, Reply};
use crate::structure::unfenced;
use lesscode_core::jsonval::{from_serde, to_serde};
use lesscode_core::regenerator::{
    RegenError, RewardEvaluation, TurnOutcome, TurnRequest, build_prompt, evaluate_tier1_static,
    parse_result, prepare_candidate, write_candidate,
};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// The module of the task: three branches that do the same (R102).
const MODULE: &str = r#"def total_price(items):
    total = 0
    for item in items:
        if item["kind"] == "book":
            total = total + item["price"] * item["count"]
        elif item["kind"] == "food":
            total = total + item["price"] * item["count"]
        else:
            total = total + item["price"] * item["count"]
    return total
"#;

const DEFINITIONS: &str = "total_price(items) returns the sum of price times count over all \
                           items; every item has kind, price and count.";

const SURFACE: &str = r#"{"modules": {}}"#;

/// Room for the answer and for the reasoning of a model that reasons.
pub const MAX_OUTPUT_TOKENS: u32 = 8_000;

/// The fixed task laid out in a temporary directory, which is removed when
/// the task is dropped.
pub struct Task {
    dir: PathBuf,
    turn: TurnRequest,
    pub request: CallRequest,
    judged: usize,
}

impl Task {
    /// Writes the task's module and builds its request with the core's
    /// prompt. The error names what could not be written.
    pub fn in_temp() -> Result<Task, String> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        let dir = std::env::temp_dir().join(format!(
            "lesscode-model-check-{}-{stamp}",
            std::process::id()
        ));
        let frozen = dir.join("frozen");
        let surface = frozen.join("public-surface.json");
        let written = std::fs::create_dir_all(frozen.join("src"))
            .and_then(|()| std::fs::write(frozen.join("src").join("mod.py"), MODULE))
            .and_then(|()| std::fs::write(&surface, SURFACE));
        // From here on the directory is removed on every way out.
        let mut task = Task {
            turn: TurnRequest {
                scope_root: frozen.to_string_lossy().into_owned(),
                scope_files: vec!["src/mod.py".to_owned()],
                surface_path: surface.to_string_lossy().into_owned(),
                definitions: DEFINITIONS.to_owned(),
                targets: vec![("written_stmts_max".to_owned(), 6)],
                contract: "all functions declared".to_owned(),
                acceptance_command: String::new(),
                feedback: None,
                language: "python".to_owned(),
            },
            request: CallRequest {
                purpose: "check".to_owned(),
                system: String::new(),
                user: String::new(),
                max_output_tokens: MAX_OUTPUT_TOKENS,
                temperature: None,
                structure: None,
            },
            judged: 0,
            dir,
        };
        written.map_err(|problem| format!("{}: {problem}", task.dir.display()))?;
        let (system, user) = build_prompt(&task.turn).map_err(|e| e.message().to_owned())?;
        task.request.system = system;
        task.request.user = user;
        Ok(task)
    }

    /// The core's checks on one reply: its parsing, then a fresh copy of the
    /// task with the reply's files, then the static checks on that copy.
    fn evaluate(&self, text: &str, candidate: &str) -> Result<RewardEvaluation, RegenError> {
        let result = parse_result(unfenced(text)).map_err(|problem| {
            RegenError::FailedTurn(format!(
                "the answer is not the JSON object asked for: {problem}"
            ))
        })?;
        let surface = from_serde(&serde_json::from_str(SURFACE).expect("the surface is JSON"));
        let copy = self.dir.join(candidate);
        prepare_candidate(&self.dir.join("frozen"), &copy, &self.turn.scope_files)?;
        let written = write_candidate(&result, &copy, &self.turn.scope_files)?;
        let outcome = TurnOutcome {
            ok: true,
            kind: String::new(),
            detail: None,
            result: Some(result),
            written,
        };
        evaluate_tier1_static(&self.turn, &outcome, &copy, &surface, &self.turn.targets)
    }

    /// What the core's checks say about one reply, with the check that
    /// rejected it named first.
    pub fn judge(&mut self, text: &str) -> Verdict {
        self.judged += 1;
        let reason = match self.evaluate(text, &format!("candidate-{}", self.judged)) {
            Ok(reward) if reward.passed_tier1 => None,
            Ok(reward) => Some(format!(
                "{}: {}",
                reward.failure_kind.unwrap_or_default(),
                to_serde(&reward.diagnostics)
            )),
            Err(error) => Some(error.message().to_owned()),
        };
        Verdict {
            passed: reason.is_none(),
            reasons: reason.into_iter().collect(),
        }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// What the check command says, and how it ends.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub lines: Vec<String>,
    pub exit: u8,
}

/// The model, the time, the tokens and the waits of the calls that were
/// answered; nothing when none was.
fn answered_line(answered: &[Reply]) -> Option<String> {
    let sent = &answered.first()?.sent;
    let sum = |tokens: fn(&Reply) -> u64| answered.iter().map(tokens).sum::<u64>();
    let waits: Vec<f64> = answered
        .iter()
        .flat_map(|reply| reply.waits.iter().map(|wait| wait.seconds))
        .collect();
    let waited = match waits.len() {
        0 => "no wait".to_owned(),
        count => format!("waited {:.0} s in {count} waits", waits.iter().sum::<f64>()),
    };
    Some(format!(
        "model {} at {}: {} calls answered in {:.1} s, {} tokens in, {} tokens out, {waited}",
        sent.model,
        sent.host,
        answered.len(),
        answered.iter().map(|reply| reply.seconds).sum::<f64>(),
        sum(|reply| reply.input_tokens),
        sum(|reply| reply.output_tokens),
    ))
}

/// Sends the fixed task through `call` and the repair loop, at most
/// `REPAIRS + 1` calls. The error is a temporary directory that could not be
/// used; nothing was called then.
pub fn check(
    call: &mut dyn FnMut(&CallRequest) -> Result<Reply, CallError>,
) -> Result<Report, String> {
    let mut task = Task::in_temp()?;
    let request = task.request.clone();
    let mut answered = Vec::new();
    let mut counted =
        |request: &CallRequest| call(request).inspect(|reply| answered.push(reply.clone()));
    let outcome = with_repairs(
        &mut counted,
        &request,
        &mut |text| task.judge(text),
        REPAIRS,
    );

    let mut lines: Vec<String> = answered_line(&answered).into_iter().collect();
    let exit = match outcome {
        Outcome::Accepted { .. } => {
            lines.push("this endpoint can be used".to_owned());
            EXIT_ANSWERED
        }
        Outcome::Rejected { attempts, .. } => {
            lines.push(format!(
                "this endpoint cannot be used as it is: the check rejected the answer of each of {} calls",
                attempts.len()
            ));
            for (index, attempt) in attempts.iter().enumerate() {
                lines.push(format!(
                    "  call {}: {}",
                    index + 1,
                    attempt.reasons.join("; ")
                ));
            }
            EXIT_REJECTED
        }
        Outcome::Failed { error, .. } => {
            lines.push(format!("the endpoint failed: {}", error.reason()));
            match error {
                CallError::Config(_) => EXIT_USAGE,
                _ => EXIT_FAILED,
            }
        }
    };
    Ok(Report { lines, exit })
}
