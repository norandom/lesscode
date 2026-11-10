//! The classes of a mutant and how one is decided from what its steps did.

use super::config::RunConfig;
use super::process::Finished;

pub const KILLED: &str = "killed";
pub const SURVIVED: &str = "survived";
pub const INVALID: &str = "invalid";
pub const TIMEOUT: &str = "timeout";
/// The machine or the command failed; this says nothing about the mutant.
pub const ERROR: &str = "error";

fn contains_any(output: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| output.contains(p.as_str()))
}

/// The class of a mutant from what its steps did, with the reason where the
/// class alone does not say it.
pub fn classify(
    config: &RunConfig,
    done: &[(&'static str, Finished)],
) -> (&'static str, Option<String>) {
    for (name, finished) in done {
        if let Some(why) = &finished.launch_error {
            return (ERROR, Some(format!("{name} could not start: {why}")));
        }
        if contains_any(&finished.output, &config.error_output_patterns) {
            return (ERROR, Some(format!("resource failure during {name}")));
        }
        let inner_limit = finished
            .exit_code
            .is_some_and(|code| config.timeout_exit_codes.contains(&code));
        if inner_limit {
            return (
                TIMEOUT,
                Some(format!("{name} reached the command's own time limit")),
            );
        }
        if finished.timed_out {
            return (TIMEOUT, Some(format!("{name} reached its time limit")));
        }
        if *name == "build" && finished.exit_code != Some(0) {
            return (INVALID, Some("did not build".into()));
        }
    }
    let Some((_, test)) = done.last().filter(|(name, _)| *name == "test") else {
        return (ERROR, Some("the test step did not run".into()));
    };
    if test.exit_code == Some(0) {
        return (SURVIVED, None);
    }
    let cannot_load = test
        .exit_code
        .is_some_and(|code| config.invalid_exit_codes.contains(&code));
    if cannot_load || contains_any(&test.output, &config.invalid_output_patterns) {
        return (
            INVALID,
            Some("the tests could not be built or loaded".into()),
        );
    }
    let evidence = &config.runner_evidence_patterns;
    if !evidence.is_empty() && !contains_any(&test.output, evidence) {
        return (ERROR, Some("no sign that the test runner ran".into()));
    }
    (KILLED, None)
}
