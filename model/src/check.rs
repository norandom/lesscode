//! The check command: is this endpoint usable. One small fixed task, the
//! first of the built-in set, goes to the configured model through the
//! repair loop, and every reply is judged by the core's checks on a copy in
//! a temporary directory. Nothing is written anywhere else.

use crate::error::CallError;
use crate::protocol::{EXIT_ANSWERED, EXIT_FAILED, EXIT_REJECTED, EXIT_USAGE};
use crate::repair::{Attempt, Outcome, REPAIRS, with_repairs};
use crate::request::{CallRequest, Reply};
use crate::taskset::built_in;

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

/// One line for each answer the check rejected, with the reasons: also for
/// an answer that a repair then put right.
fn rejected_lines(attempts: &[Attempt]) -> impl Iterator<Item = String> {
    attempts
        .iter()
        .enumerate()
        .filter(|(_, attempt)| !attempt.passed)
        .map(|(index, attempt)| format!("  call {}: {}", index + 1, attempt.reasons.join("; ")))
}

/// Sends the fixed task through `call` and the repair loop, at most
/// `REPAIRS + 1` calls. The error is a temporary directory that could not be
/// used; nothing was called then.
pub fn check(
    call: &mut dyn FnMut(&CallRequest) -> Result<Reply, CallError>,
) -> Result<Report, String> {
    let (set, mut core) = built_in()?;
    let task = &set.tasks[0];
    let request = CallRequest {
        purpose: "check".to_owned(),
        ..task.request.clone()
    };
    let mut answered = Vec::new();
    let mut counted =
        |request: &CallRequest| call(request).inspect(|reply| answered.push(reply.clone()));
    let outcome = with_repairs(
        &mut counted,
        &request,
        &mut |text| core.judge(task, text),
        REPAIRS,
    );

    let mut lines: Vec<String> = answered_line(&answered).into_iter().collect();
    let exit = match outcome {
        Outcome::Accepted { attempts, .. } => {
            lines.extend(rejected_lines(&attempts));
            lines.push("this endpoint can be used".to_owned());
            EXIT_ANSWERED
        }
        Outcome::Rejected { attempts, .. } => {
            lines.push(format!(
                "this endpoint cannot be used as it is: the check rejected the answer of each of {} calls",
                attempts.len()
            ));
            lines.extend(rejected_lines(&attempts));
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
