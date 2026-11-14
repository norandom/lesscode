//! Requirement 6: repairing a rejected answer (task 3), with scripted
//! replies. No endpoint is called and no file is written.

use lesscode_model::repair::{Attempt, Outcome, REPAIRS, Verdict, with_repairs};
use lesscode_model::{CallError, CallRequest, Reply, Sent, Stop};

fn request() -> CallRequest {
    CallRequest {
        purpose: "repair test".into(),
        system: "Answer in one word.".into(),
        user: "Name a colour.".into(),
        max_output_tokens: 100,
        temperature: Some(0.2),
        structure: None,
    }
}

fn reply(text: &str, stop: Stop, output_tokens: u64) -> Reply {
    Reply {
        text: text.into(),
        stop,
        input_tokens: 10,
        output_tokens,
        reasoning_tokens: None,
        cached_input_tokens: None,
        structured: None,
        seconds: 0.0,
        waits: Vec::new(),
        sent: Sent {
            host: "stub".into(),
            model: "stub-model".into(),
            max_output_tokens: 100,
            temperature: Some(0.2),
            reasoning_level: None,
        },
    }
}

fn complete(text: &str) -> Result<Reply, CallError> {
    Ok(reply(text, Stop::Complete, 1))
}

struct Run {
    outcome: Outcome,
    /// The requests in the order they were sent.
    sent: Vec<CallRequest>,
    /// The texts in the order the check was handed them.
    checked: Vec<String>,
}

/// The loop over a script of replies. The check passes the text "blue" and
/// says of any other text that it is not blue.
fn run(script: Vec<Result<Reply, CallError>>, repairs: usize) -> Run {
    let mut script = script.into_iter();
    let (mut sent, mut checked) = (Vec::new(), Vec::new());
    let mut call = |asked: &CallRequest| {
        sent.push(asked.clone());
        script.next().expect("a reply for every call")
    };
    let mut check = |text: &str| {
        checked.push(text.to_owned());
        Verdict {
            passed: text == "blue",
            reasons: vec![format!("\"{text}\" is not blue")],
        }
    };
    let outcome = with_repairs(&mut call, &request(), &mut check, repairs);
    Run {
        outcome,
        sent,
        checked,
    }
}

fn json(request: &CallRequest) -> serde_json::Value {
    serde_json::to_value(request).expect("a request is JSON")
}

fn passed(attempts: &[Attempt]) -> Vec<bool> {
    attempts.iter().map(|attempt| attempt.passed).collect()
}

#[test]
fn a_reply_that_passes_at_once_is_accepted_after_one_call() {
    let run = run(vec![complete("blue")], REPAIRS);
    let Outcome::Accepted { reply, attempts } = run.outcome else {
        panic!("not accepted: {:?}", run.outcome);
    };
    assert_eq!(reply.text, "blue");
    assert_eq!(passed(&attempts), [true]);
    assert_eq!(run.sent.len(), 1);
    assert_eq!(json(&run.sent[0]), json(&request()));
}

#[test]
fn a_rejected_reply_is_repaired_by_a_call_that_shows_request_reply_and_reasons() {
    let run = run(vec![complete("red"), complete("blue")], REPAIRS);
    let Outcome::Accepted { reply, attempts } = run.outcome else {
        panic!("not accepted: {:?}", run.outcome);
    };
    assert_eq!(reply.text, "blue");
    assert_eq!(passed(&attempts), [false, true]);
    assert_eq!(run.sent.len(), 2);
    let repair = &run.sent[1];
    assert!(repair.user.starts_with("Name a colour."), "{}", repair.user);
    assert!(
        repair.user.contains("The rejected answer:\nred\n"),
        "{}",
        repair.user
    );
    assert!(
        repair.user.contains("- \"red\" is not blue\n"),
        "{}",
        repair.user
    );
    // Only the request text differs from the request as stated.
    let mut as_stated = repair.clone();
    as_stated.user = request().user;
    assert_eq!(json(&as_stated), json(&request()));
}

#[test]
fn the_second_repair_is_a_fresh_call_that_shows_only_the_reply_just_rejected() {
    let run = run(
        vec![complete("red"), complete("green"), complete("blue")],
        REPAIRS,
    );
    let Outcome::Accepted { reply, attempts } = run.outcome else {
        panic!("not accepted: {:?}", run.outcome);
    };
    assert_eq!(reply.text, "blue");
    assert_eq!(passed(&attempts), [false, false, true]);
    assert_eq!(run.sent.len(), 3);
    let second = &run.sent[2].user;
    assert!(second.contains("The rejected answer:\ngreen\n"), "{second}");
    assert!(second.contains("- \"green\" is not blue\n"), "{second}");
    assert!(!second.contains("red"), "{second}");
}

#[test]
fn after_two_repairs_the_last_reply_comes_back_rejected_with_the_reasons_of_every_attempt() {
    let run = run(
        vec![
            complete("red"),
            complete("green"),
            complete("grey"),
            complete("blue"),
        ],
        REPAIRS,
    );
    let Outcome::Rejected { last, attempts } = run.outcome else {
        panic!("not rejected: {:?}", run.outcome);
    };
    assert_eq!(run.sent.len(), 3, "a first call and two repairs");
    assert_eq!(last.text, "grey");
    let reasons: Vec<&str> = attempts
        .iter()
        .flat_map(|attempt| attempt.reasons.iter().map(String::as_str))
        .collect();
    assert_eq!(
        reasons,
        [
            "\"red\" is not blue",
            "\"green\" is not blue",
            "\"grey\" is not blue"
        ]
    );
    assert_eq!(passed(&attempts), [false, false, false]);
}

#[test]
fn a_reply_that_was_cut_off_or_did_not_arrive_is_asked_again_with_the_request_as_it_was() {
    let run = run(
        vec![
            Ok(reply("bl", Stop::CutOff, 100)),
            Ok(reply("  ", Stop::Complete, 0)),
            complete("blue"),
        ],
        REPAIRS,
    );
    assert!(
        matches!(run.outcome, Outcome::Accepted { .. }),
        "{:?}",
        run.outcome
    );
    assert_eq!(run.sent.len(), 3);
    for sent in &run.sent {
        assert_eq!(json(sent), json(&request()));
    }
}

#[test]
fn the_caller_states_how_many_repairs_are_allowed() {
    let none = run(vec![complete("red"), complete("blue")], 0);
    assert!(
        matches!(none.outcome, Outcome::Rejected { .. }),
        "{:?}",
        none.outcome
    );
    assert_eq!(none.sent.len(), 1);

    let three = run(
        vec![
            complete("red"),
            complete("green"),
            complete("grey"),
            complete("blue"),
        ],
        3,
    );
    assert!(
        matches!(three.outcome, Outcome::Accepted { .. }),
        "{:?}",
        three.outcome
    );
    assert_eq!(three.sent.len(), 4);
}

#[test]
fn every_attempt_is_recorded_with_its_verdict_its_stop_and_its_tokens() {
    let run = run(
        vec![
            Ok(reply("bl", Stop::CutOff, 100)),
            Ok(reply("red", Stop::Complete, 3)),
            Ok(reply("blue", Stop::Complete, 4)),
        ],
        REPAIRS,
    );
    let Outcome::Accepted { attempts, .. } = run.outcome else {
        panic!("not accepted: {:?}", run.outcome);
    };
    let attempt = |passed, reason: Option<&str>, stop, output_tokens| Attempt {
        passed,
        reasons: reason.into_iter().map(str::to_owned).collect(),
        stop,
        input_tokens: 10,
        output_tokens,
    };
    assert_eq!(
        attempts,
        [
            attempt(false, Some("\"bl\" is not blue"), Stop::CutOff, 100),
            attempt(false, Some("\"red\" is not blue"), Stop::Complete, 3),
            attempt(true, Some("\"blue\" is not blue"), Stop::Complete, 4),
        ]
    );
}

#[test]
fn a_call_that_fails_ends_the_loop_as_a_failure_of_the_endpoint_not_as_a_rejection() {
    let unreachable = CallError::Unreachable {
        host: "stub".into(),
        detail: "connection refused".into(),
    };
    let run = run(vec![complete("red"), Err(unreachable.clone())], REPAIRS);
    let Outcome::Failed { error, attempts } = run.outcome else {
        panic!("not failed: {:?}", run.outcome);
    };
    assert_eq!(error, unreachable);
    assert_eq!(passed(&attempts), [false]);
    assert_eq!(run.checked, ["red"], "a failure is not handed to the check");
}

/// Every course of three replies: the outcome is accepted exactly when the
/// check passed a reply, the accepted reply is the one the check saw last,
/// and nothing is sent after it.
#[test]
fn an_accepted_outcome_exists_only_after_a_verdict_that_passed() {
    for course in 0..8_u32 {
        let texts: Vec<&str> = (0..3)
            .map(|at| if course >> at & 1 == 1 { "blue" } else { "red" })
            .collect();
        let run = run(texts.iter().map(|text| complete(text)).collect(), REPAIRS);
        let first_pass = texts.iter().position(|text| *text == "blue");
        match (&run.outcome, first_pass) {
            (Outcome::Accepted { reply, attempts }, Some(at)) => {
                assert_eq!(run.checked.len(), at + 1, "course {texts:?}");
                assert_eq!(run.checked.last(), Some(&reply.text), "course {texts:?}");
                assert_eq!(run.sent.len(), at + 1, "course {texts:?}");
                assert!(attempts[at].passed, "course {texts:?}");
            }
            (Outcome::Rejected { attempts, .. }, None) => {
                assert_eq!(passed(attempts), [false, false, false]);
            }
            (outcome, _) => panic!("course {texts:?} ended as {outcome:?}"),
        }
    }
}
