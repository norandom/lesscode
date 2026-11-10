//! The model behind the optimiser: a live endpoint or a scripted stand-in,
//! metered against a budget. The API key never leaves `Endpoint`.

use lesscode_core::regenerator::{Endpoint, ureq_post};
use serde_json::{Value, json};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Reply {
    pub text: String,
    pub stop_reason: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

pub trait Model: Send + Sync {
    fn name(&self) -> String;
    fn chat(&self, system: &str, user: &str, max_tokens: u64) -> Result<Reply, String>;
}

pub struct Live(pub Endpoint);

impl Live {
    fn request(
        &self,
        system: &str,
        user: &str,
        max_tokens: u64,
    ) -> (String, Vec<(String, String)>, Value) {
        let ep = &self.0;
        let base = ep.base_url.trim_end_matches('/');
        let json_type = ("content-type".to_owned(), "application/json".to_owned());
        let turn = json!({"role": "user", "content": user});
        if ep.style == "anthropic" {
            let url = if base.ends_with("/v1") {
                format!("{base}/messages")
            } else {
                format!("{base}/v1/messages")
            };
            let headers = vec![
                ("x-api-key".to_owned(), ep.api_key().to_owned()),
                ("anthropic-version".to_owned(), "2023-06-01".to_owned()),
                json_type,
            ];
            let mut body = json!({"model": ep.model, "max_tokens": max_tokens, "messages": [turn]});
            if !system.is_empty() {
                body["system"] = json!(system);
            }
            (url, headers, body)
        } else {
            let headers = vec![
                (
                    "Authorization".to_owned(),
                    format!("Bearer {}", ep.api_key()),
                ),
                json_type,
            ];
            let mut messages = vec![turn];
            if !system.is_empty() {
                messages.insert(0, json!({"role": "system", "content": system}));
            }
            let body = json!({"model": ep.model, "max_tokens": max_tokens, "messages": messages});
            (format!("{base}/chat/completions"), headers, body)
        }
    }

    fn read(&self, raw: &[u8]) -> Result<Reply, String> {
        let data: Value =
            serde_json::from_slice(raw).map_err(|e| format!("unreadable response: {e}"))?;
        let count = |v: &Value| v.as_u64().unwrap_or(0);
        if self.0.style == "anthropic" {
            let blocks = data["content"].as_array().cloned().unwrap_or_default();
            let text: String = blocks
                .iter()
                .filter(|b| b["type"] == "text")
                .filter_map(|b| b["text"].as_str())
                .collect();
            Ok(Reply {
                text,
                stop_reason: data["stop_reason"].as_str().unwrap_or("").to_owned(),
                input_tokens: count(&data["usage"]["input_tokens"]),
                output_tokens: count(&data["usage"]["output_tokens"]),
            })
        } else {
            let choice = &data["choices"][0];
            Ok(Reply {
                text: choice["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_owned(),
                stop_reason: choice["finish_reason"].as_str().unwrap_or("").to_owned(),
                input_tokens: count(&data["usage"]["prompt_tokens"]),
                output_tokens: count(&data["usage"]["completion_tokens"]),
            })
        }
    }
}

impl Model for Live {
    fn name(&self) -> String {
        self.0.model.clone()
    }

    /// One request; a transport failure is retried once after ten seconds.
    fn chat(&self, system: &str, user: &str, max_tokens: u64) -> Result<Reply, String> {
        let (url, headers, body) = self.request(system, user, max_tokens);
        let bytes = body.to_string().into_bytes();
        let raw = ureq_post(&url, &headers, &bytes).or_else(|_| {
            std::thread::sleep(std::time::Duration::from_secs(10));
            ureq_post(&url, &headers, &bytes)
        })?;
        self.read(&raw)
    }
}

const SCRIPTED_HINT: &str = "Check every field against the examples before answering.";

/// A stand-in that makes no call. It answers a reflection request with the
/// current guide plus one sentence, and an authoring request with the shipped
/// pack of the language when the guide has that sentence and with a weakened
/// pack when it has not. For checking the plumbing only.
pub struct Scripted {
    pub packs: PathBuf,
}

impl Model for Scripted {
    fn name(&self) -> String {
        "scripted".into()
    }

    fn chat(&self, system: &str, user: &str, _max_tokens: u64) -> Result<Reply, String> {
        let asked = user
            .starts_with("Example packs:")
            .then(|| user.rsplit_once("Write the pack for: "))
            .flatten();
        let text = match asked {
            Some((_, language)) => {
                let shipped =
                    std::fs::read_to_string(self.packs.join(format!("{}.json", language.trim())))
                        .map_err(|e| e.to_string())?;
                if system.contains(SCRIPTED_HINT) {
                    shipped
                } else {
                    // Without the hint the stand-in forgets how failures are rewritten.
                    let mut weak: Value =
                        serde_json::from_str(&shipped).map_err(|e| e.to_string())?;
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
            stop_reason: "end_turn".into(),
            input_tokens: 0,
            output_tokens: 0,
        })
    }
}

/// USD per million tokens (input, output) at the vendor's list price.
pub fn list_price(model: &str) -> Option<(f64, f64)> {
    match model {
        "claude-opus-5-5" => Some((4.0, 20.0)),
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

/// A model with a ledger and two stops: this run's maximum and the project's.
pub struct Metered {
    pub model: Box<dyn Model>,
    pub price: (f64, f64),
    pub max_usd: f64,
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
            project_room_usd,
            reserve_usd,
            ledger,
            state: Mutex::new((0.0, 0.0, 0)),
        }
    }

    pub fn spent(&self) -> (f64, u64) {
        let state = self.state.lock().unwrap();
        (state.0, state.2)
    }

    /// One call, refused when it could pass a stop. Every call is written
    /// to the ledger with its purpose.
    pub fn chat(
        &self,
        purpose: &str,
        system: &str,
        user: &str,
        max_tokens: u64,
    ) -> Result<Reply, String> {
        {
            let mut state = self.state.lock().unwrap();
            let committed = state.0 + state.1 + self.reserve_usd;
            if committed > self.max_usd || committed > self.project_room_usd {
                return Err("not run: the budget would be passed".into());
            }
            state.1 += self.reserve_usd;
        }
        let result = self.model.chat(system, user, max_tokens);
        let mut state = self.state.lock().unwrap();
        state.1 -= self.reserve_usd;
        state.2 += 1;
        let (tokens_in, tokens_out) = result
            .as_ref()
            .map(|r| (r.input_tokens, r.output_tokens))
            .unwrap_or((0, 0));
        state.0 += cost(self.price, tokens_in, tokens_out);
        let row = json!({"purpose": purpose, "model": self.model.name(), "input_tokens": tokens_in,
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
