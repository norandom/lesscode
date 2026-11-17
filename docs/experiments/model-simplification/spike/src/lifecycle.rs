//! Disposable adapter around the existing seam; no endpoint is called by this module.
use lesscode_model::repair::{with_repairs, Outcome, Verdict};
use lesscode_model::{CallError, CallRequest, Reply, Stop};
use std::cell::RefCell;

#[derive(Debug)]
pub struct Record {
    pub outcome: Outcome,
    pub requests: Vec<CallRequest>,
    pub replies: Vec<Reply>,
    pub judge_failure: Option<String>,
}

pub fn trial(
    call: &mut dyn FnMut(&CallRequest) -> Result<Reply, CallError>,
    request: &CallRequest,
    judge: &mut dyn FnMut(&str) -> Result<Verdict, String>,
) -> Record {
    let stop = RefCell::new(Stop::Complete);
    let failure = RefCell::new(None::<String>);
    let (mut requests, mut replies) = (Vec::new(), Vec::new());
    let mut guarded_call = |asked: &CallRequest| {
        if let Some(error) = failure.borrow().as_ref() {
            return Err(CallError::Config(format!("terminal judge error: {error}")));
        }
        requests.push(asked.clone());
        let reply = call(asked)?;
        *stop.borrow_mut() = reply.stop.clone();
        replies.push(reply.clone());
        Ok(reply)
    };
    let mut guarded_judge = |text: &str| {
        if *stop.borrow() != Stop::Complete || text.trim().is_empty() {
            return Verdict {
                passed: false,
                reasons: vec!["incomplete or refused reply".into()],
            };
        }
        match judge(text) {
            Ok(verdict) => verdict,
            Err(error) => {
                *failure.borrow_mut() = Some(error.clone());
                Verdict {
                    passed: false,
                    reasons: vec![error],
                }
            }
        }
    };
    let mut outcome = with_repairs(&mut guarded_call, request, &mut guarded_judge, 2);
    if let Some(error) = failure.borrow().as_ref() {
        let attempts = match outcome {
            Outcome::Accepted { attempts, .. }
            | Outcome::Rejected { attempts, .. }
            | Outcome::Failed { attempts, .. } => attempts,
        };
        outcome = Outcome::Failed {
            error: CallError::Config(format!("terminal judge error: {error}")),
            attempts,
        };
    }
    Record {
        outcome,
        requests,
        replies,
        judge_failure: failure.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lesscode_model::Sent;

    fn reply(text: &str, stop: Stop) -> Reply {
        Reply {
            text: text.into(),
            stop,
            input_tokens: 1,
            output_tokens: 1,
            reasoning_tokens: None,
            cached_input_tokens: None,
            structured: None,
            seconds: 0.0,
            waits: vec![],
            sent: Sent {
                host: "stub".into(),
                model: "stub".into(),
                max_output_tokens: 8192,
                temperature: None,
                reasoning_level: None,
            },
        }
    }

    fn run(replies: Vec<Reply>, terminal: bool) -> Record {
        let request = CallRequest {
            purpose: "P0 control".into(),
            system: String::new(),
            user: "preserve behavior".into(),
            max_output_tokens: 8192,
            temperature: None,
            structure: None,
        };
        let mut replies = replies.into_iter();
        trial(
            &mut |_| Ok(replies.next().expect("unexpected extra provider call")),
            &request,
            &mut |text| {
                if terminal {
                    return Err("checker interrupted".into());
                }
                Ok(Verdict {
                    passed: text == "unchanged",
                    reasons: vec!["wrong return".into()],
                })
            },
        )
    }

    #[test]
    fn accepted_unchanged_stops_without_optimization() {
        let record = run(vec![reply("unchanged", Stop::Complete)], false);
        assert!(matches!(record.outcome, Outcome::Accepted { .. }));
        assert_eq!(record.requests.len(), 1);
        assert_eq!(record.replies.len(), 1);
    }

    #[test]
    fn rejected_then_repaired_retains_raw_attempts() {
        let record = run(
            vec![
                reply("wrong", Stop::Complete),
                reply("unchanged", Stop::Complete),
            ],
            false,
        );
        assert!(matches!(record.outcome, Outcome::Accepted { .. }));
        assert_eq!(record.replies[0].text, "wrong");
        assert!(record.requests[1].user.contains("wrong return"));
        assert_eq!(record.requests.len(), 2);
    }

    #[test]
    fn refused_reply_cannot_be_accepted_even_when_text_matches() {
        let record = run(vec![reply("unchanged", Stop::Refused)], false);
        assert!(matches!(record.outcome, Outcome::Rejected { .. }));
        assert_eq!(record.requests.len(), 1);
    }

    #[test]
    fn terminal_judge_error_never_calls_provider_again() {
        let record = run(vec![reply("unchanged", Stop::Complete)], true);
        assert!(matches!(record.outcome, Outcome::Failed { .. }));
        assert_eq!(record.requests.len(), 1);
        assert_eq!(record.judge_failure.as_deref(), Some("checker interrupted"));
    }

    #[test]
    fn cutoffs_do_not_reach_the_judge_or_pass_from_partial_text() {
        let record = run(vec![reply("unchanged", Stop::CutOff); 3], true);
        assert!(matches!(record.outcome, Outcome::Rejected { .. }));
        assert!(record.judge_failure.is_none());
        assert_eq!(record.requests.len(), 3);
        assert_eq!(record.requests[0].user, record.requests[2].user);
    }
}
