//! What the user is told while a call waits, runs long or fails. The lines
//! are built here so that every caller says the same thing.

/// Receives the notices of a call.
pub trait Notice {
    /// Before a wait: the endpoint asked to come back later.
    fn waiting(&self, host: &str, expected_seconds: u64, waited_so_far: u64, attempt: u32);
    /// At intervals while an accepted call takes longer than expected.
    fn still_running(&self, host: &str, running_seconds: u64);
    /// Once, when a call has failed.
    fn failed(&self, reason: &str);
}

pub fn waiting_line(host: &str, expected_seconds: u64, waited_so_far: u64, attempt: u32) -> String {
    format!(
        "[model] {host} asks to come back later (attempt {attempt}); waiting about \
         {expected_seconds} s, {waited_so_far} s waited so far"
    )
}

pub fn still_running_line(host: &str, running_seconds: u64) -> String {
    format!("[model] still waiting for the answer of {host}: {running_seconds} s so far")
}

pub fn failed_line(reason: &str) -> String {
    format!("[model] the call failed: {reason}")
}

/// Writes the notices to standard error, where a user sees them and a
/// caller's standard output stays clean.
pub struct Terminal;

impl Notice for Terminal {
    fn waiting(&self, host: &str, expected_seconds: u64, waited_so_far: u64, attempt: u32) {
        eprintln!(
            "{}",
            waiting_line(host, expected_seconds, waited_so_far, attempt)
        );
    }

    fn still_running(&self, host: &str, running_seconds: u64) {
        eprintln!("{}", still_running_line(host, running_seconds));
    }

    fn failed(&self, reason: &str) {
        eprintln!("{}", failed_line(reason));
    }
}
