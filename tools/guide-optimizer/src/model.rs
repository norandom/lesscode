//! The model behind the optimiser: a live endpoint reached through the model
//! seam, or a scripted stand-in, metered against a budget.

use lesscode_core::regenerator::Endpoint;
use lesscode_model::{CallError, CallRequest, Limits, Reply, Sent, Stop, Terminal};
use serde_json::{Value, json};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub trait Model: Send + Sync {
    fn name(&self) -> String;
    fn chat(&self, request: &CallRequest) -> Result<Reply, CallError>;
}

pub struct Live(pub Endpoint);

impl Model for Live {
    fn name(&self) -> String {
        self.0.model.clone()
    }

    /// One call through the seam, which does the waiting. The tool keeps its
    /// own ledger, so the seam writes no record.
    fn chat(&self, request: &CallRequest) -> Result<Reply, CallError> {
        lesscode_model::call(&self.0, request, &Limits::default(), &Terminal, None)
    }
}

const SCRIPTED_HINT: &str = "Check every field against the examples before answering.";

/// A stand-in that makes no call. It answers a reflection request with the
/// current guide plus one sentence, and an authoring request with the shipped
/// pack of the language when the guide has that sentence and with a weakened
/// pack when it has not; a repair request is answered with the shipped pack.
/// For checking the plumbing only.
pub struct Scripted {
    pub packs: PathBuf,
}

impl Model for Scripted {
    fn name(&self) -> String {
        "scripted".into()
    }

    fn chat(&self, request: &CallRequest) -> Result<Reply, CallError> {
        let (system, user) = (&request.system, &request.user);
        let asked = user
            .starts_with("Example packs:")
            .then(|| user.rsplit_once("Write the pack for: "))
            .flatten();
        let text = match asked {
            Some((_, rest)) => {
                let language = rest.lines().next().unwrap_or_default().trim();
                let shipped = std::fs::read_to_string(self.packs.join(format!("{language}.json")))
                    .map_err(|e| CallError::Config(e.to_string()))?;
                // The seam's repair request names the reasons under this line.
                if system.contains(SCRIPTED_HINT) || rest.contains("Why it was rejected:") {
                    shipped
                } else {
                    // Without the hint the stand-in forgets how failures are rewritten.
                    let mut weak: Value = serde_json::from_str(&shipped)
                        .map_err(|e| CallError::Config(e.to_string()))?;
                    weak["operators"]["failure_state"] = json!({});
                    weak.to_string()
                }
            }
            None => {
                let current = user.split("```").nth(1).unwrap_or(system).trim();
                format!("```\n{current}\n\n{SCRIPTED_HINT}\n```")
            }
        };
        Ok(Reply {
            text,
            stop: Stop::Complete,
            input_tokens: 0,
            output_tokens: 0,
            reasoning_tokens: None,
            cached_input_tokens: None,
            structured: None,
            seconds: 0.0,
            waits: Vec::new(),
            sent: Sent {
                host: "scripted".into(),
                model: self.name(),
                max_output_tokens: request.max_output_tokens,
                temperature: request.temperature,
                reasoning_level: None,
            },
        })
    }
}

/// USD per million tokens (input, output) at the vendor's list price.
pub fn list_price(model: &str) -> Option<(f64, f64)> {
    match model {
        "claude-opus-5-5" => Some((4.0, 20.0)),
        "mistral-large-4" => Some((0.7, 2.1)),
        "scripted" => Some((0.0, 0.0)),
        _ => None,
    }
}

pub fn cost(price: (f64, f64), input_tokens: u64, output_tokens: u64) -> f64 {
    (input_tokens as f64 * price.0 + output_tokens as f64 * price.1) / 1e6
}

/// Spend recorded in every `ledger.jsonl` under `root`, at list prices where
/// known and at `fallback` otherwise.
pub fn recorded_spend(root: &Path, fallback: (f64, f64)) -> f64 {
    let mut total = 0.0;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for path in entries.flatten().map(|e| e.path()) {
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().is_some_and(|n| n == "ledger.jsonl") {
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                for row in text
                    .lines()
                    .filter_map(|l| serde_json::from_str::<Value>(l).ok())
                {
                    let price = row["model"]
                        .as_str()
                        .and_then(list_price)
                        .unwrap_or(fallback);
                    total += cost(
                        price,
                        row["input_tokens"].as_u64().unwrap_or(0),
                        row["output_tokens"].as_u64().unwrap_or(0),
                    );
                }
            }
        }
    }
    total
}

/// What a call that `Metered` refused says: it was not run.
pub const NOT_RUN: &str = "not run: the budget would be passed";

/// A model with a ledger and two stops: this run's maximum and the project's.
pub struct Metered {
    pub model: Box<dyn Model>,
    pub price: (f64, f64),
    max_usd: f64,
    /// Calls that must stay affordable after the current phase.
    held_back: Mutex<usize>,
    pub project_room_usd: f64,
    pub reserve_usd: f64,
    pub ledger: PathBuf,
    state: Mutex<(f64, f64, u64)>, // spent, reserved, calls
}

impl Metered {
    pub fn new(
        model: Box<dyn Model>,
        price: (f64, f64),
        max_usd: f64,
        project_room_usd: f64,
        ledger: PathBuf,
    ) -> Self {
        let reserve_usd = cost(price, 6_000, 4_000);
        Metered {
            model,
            price,
            max_usd,
            held_back: Mutex::new(0),
            project_room_usd,
            reserve_usd,
            ledger,
            state: Mutex::new((0.0, 0.0, 0)),
        }
    }

    /// Keep this many later calls affordable: their cost is taken to be the
    /// mean cost of the calls made so far, or the reserve before any was made.
    pub fn hold_back(&self, calls: usize) {
        *self.held_back.lock().unwrap() = calls;
    }

    pub fn spent(&self) -> (f64, u64) {
        let state = self.state.lock().unwrap();
        (state.0, state.2)
    }

    /// One call, refused with `NOT_RUN` when it could pass a stop. Every call
    /// is written to the ledger with its purpose.
    pub fn chat(&self, request: &CallRequest) -> Result<Reply, CallError> {
        {
            let mut state = self.state.lock().unwrap();
            let committed = state.0 + state.1 + self.reserve_usd;
            let per_call = if state.2 == 0 {
                self.reserve_usd
            } else {
                state.0 / state.2 as f64
            };
            let held = *self.held_back.lock().unwrap() as f64 * per_call;
            if committed + held > self.max_usd || committed > self.project_room_usd {
                return Err(CallError::Config(NOT_RUN.into()));
            }
            state.1 += self.reserve_usd;
        }
        let result = self.model.chat(request);
        let mut state = self.state.lock().unwrap();
        state.1 -= self.reserve_usd;
        state.2 += 1;
        let (tokens_in, tokens_out) = result
            .as_ref()
            .map(|r| (r.input_tokens, r.output_tokens))
            .unwrap_or((0, 0));
        state.0 += cost(self.price, tokens_in, tokens_out);
        let row = json!({"purpose": request.purpose, "model": self.model.name(), "input_tokens": tokens_in,
                         "output_tokens": tokens_out, "ok": result.is_ok()});
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.ledger)
        {
            let _ = writeln!(file, "{row}");
        }
        result
    }
}

#[cfg(test)]
mod scripted {
    use super::*;
    use lesscode_model::repair::repair_request;

    #[test]
    fn it_answers_the_seams_repair_request_with_the_shipped_pack() {
        let packs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../native/packs");
        let shipped = std::fs::read_to_string(packs.join("java.json")).unwrap();
        let model = Scripted { packs };
        let request = CallRequest {
            purpose: "author".into(),
            system: "guide".into(),
            user: "Example packs:\n\n{}\n\nWrite the pack for: java".into(),
            max_output_tokens: 100,
            temperature: None,
            structure: None,
        };
        let first = model.chat(&request).unwrap().text;
        assert_ne!(first, shipped);
        let repair = repair_request(&request, &first, &["a reason".into()]);
        assert_eq!(model.chat(&repair).unwrap().text, shipped);
    }
}
