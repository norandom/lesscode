//! A record of every call: one JSON line appended to the path the caller
//! names, and nowhere else. It holds what is needed to compare a model's
//! cost and behaviour afterwards without calling it again, and no key.

use crate::client::host_of;
use crate::error::CallError;
use crate::request::{CallRequest, Reply};
use lesscode_core::regenerator::Endpoint;
use serde_json::{Value, json};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since 1970 as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn utc(seconds: u64) -> String {
    let (days, rest) = (seconds / 86_400, seconds % 86_400);
    // Days to a civil date, counted in 400-year eras from 0000-03-01.
    let shifted = days + 719_468;
    let (era, day_of_era) = (shifted / 146_097, shifted % 146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3_600,
        rest % 3_600 / 60,
        rest % 60
    )
}

/// The line for one call, successful or not.
pub fn line(
    endpoint: &Endpoint,
    request: &CallRequest,
    outcome: &Result<Reply, CallError>,
    seconds: f64,
) -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let mut line = json!({
        "time_utc": utc(now),
        "purpose": request.purpose,
        "host": host_of(endpoint),
        "model": endpoint.model,
        "ok": outcome.is_ok(),
        "sent": {
            "max_output_tokens": request.max_output_tokens,
            "temperature": request.temperature,
            "reasoning_level": endpoint.reasoning,
        },
        "seconds": (seconds * 10.0).round() / 10.0,
    });
    match outcome {
        Ok(reply) => {
            line["stop"] = json!(reply.stop);
            line["input_tokens"] = json!(reply.input_tokens);
            line["output_tokens"] = json!(reply.output_tokens);
            if let Some(tokens) = reply.reasoning_tokens {
                line["reasoning_tokens"] = json!(tokens);
            }
            if let Some(tokens) = reply.cached_input_tokens {
                line["cached_input_tokens"] = json!(tokens);
            }
            line["waits"] = json!(reply.waits);
        }
        Err(error) => line["failure"] = json!(error.reason()),
    }
    line
}

/// Appends the line of one call to `path`. Without a path nothing is written.
pub fn record(
    path: Option<&Path>,
    endpoint: &Endpoint,
    request: &CallRequest,
    outcome: &Result<Reply, CallError>,
    seconds: f64,
) -> std::io::Result<()> {
    let Some(path) = path else {
        return Ok(());
    };
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", line(endpoint, request, outcome, seconds))
}

#[cfg(test)]
mod tests {
    use super::utc;

    #[test]
    fn seconds_become_a_utc_date_and_time() {
        assert_eq!(utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(utc(1_794_576_111), "2026-11-13T13:21:51Z");
        assert_eq!(utc(4_102_444_799), "2099-12-31T23:59:59Z");
    }
}
