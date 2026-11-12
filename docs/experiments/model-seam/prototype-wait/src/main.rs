//! Prototype gate P3 for the model seam: a refusal that asks to come back
//! later is waited out, and before each wait the user is told for how long.
//!
//!     model-seam-prototype-wait stub
//!         five scripted cases against a local stub endpoint; exits non-zero
//!         when a case does not behave as expected
//!     model-seam-prototype-wait live ENV_FILE SYSTEM_TXT USER_TXT COUNT
//!         COUNT calls at once against a real endpoint, each waiting as needed
//!
//! The rule under test has no entry per endpoint: a wait the endpoint states
//! in a header, else one it states in its message, else a first step; and
//! never less than twice the wait that has just failed.

use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest, ReasoningEffort};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};
use lesscode_core::regenerator::{Endpoint as Configured, load_endpoint};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Policy {
    /// The first wait when the endpoint states none.
    first: f64,
    /// The waits of one call may add up to this much.
    total: f64,
}

// The fields are shown through `Debug` only.
#[allow(dead_code)]
#[derive(Debug)]
enum Failure {
    Refused {
        status: u16,
        said: String,
    },
    WaitLimit {
        waited: f64,
        limit: f64,
        attempts: u32,
    },
    Other(String),
}

struct Answer {
    text: String,
    /// "stop", "cut_off" or the endpoint's own word.
    stop: String,
    input_tokens: i64,
    output_tokens: i64,
    waits: Vec<(f64, &'static str)>,
    seconds: f64,
}

/// What a caller may set for one call; nothing else is sent.
struct Settings {
    max_output_tokens: u32,
    temperature: Option<f64>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            max_output_tokens: 64_000,
            temperature: None,
        }
    }
}

/// Seconds the endpoint asks for, and where it said so.
fn stated_wait(retry_after: Option<&str>, body: &str) -> Option<(f64, &'static str)> {
    if let Some(seconds) = retry_after.and_then(|v| v.trim().parse::<f64>().ok()) {
        return Some((seconds, "header"));
    }
    let lower = body.to_lowercase();
    let rest = lower.split("retry after ").nth(1)?;
    let number: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    number.parse().ok().map(|seconds| (seconds, "message"))
}

fn shortened(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(300)
        .collect()
}

fn client_for(ep: &Configured) -> Result<Client, String> {
    let kind = if ep.style == "anthropic" {
        AdapterKind::Anthropic
    } else {
        AdapterKind::OpenAI
    };
    let trimmed = ep.base_url.trim_end_matches('/');
    let base = if ep.style == "anthropic" && !trimmed.ends_with("/v1") {
        format!("{trimmed}/v1/")
    } else {
        format!("{trimmed}/")
    };
    let key = ep.api_key().to_owned();
    let resolver = ServiceTargetResolver::from_resolver_fn(
        move |target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
            Ok(ServiceTarget {
                endpoint: Endpoint::from_owned(base.clone()),
                auth: AuthData::from_single(key.clone()),
                model: ModelIden::new(kind, target.model.model_name),
            })
        },
    );
    Client::builder()
        .with_service_target_resolver(resolver)
        .build()
        .map_err(|e| e.to_string())
}

fn host_of(ep: &Configured) -> String {
    let after = ep.base_url.split("://").nth(1).unwrap_or(&ep.base_url);
    after.split('/').next().unwrap_or(after).to_owned()
}

/// One call. A refusal with "too many requests" or "service unavailable" is
/// waited out; the user is told before each wait.
fn call(
    ep: &Configured,
    system: &str,
    user: &str,
    policy: &Policy,
    tag: &str,
    settings: &Settings,
) -> Result<Answer, Failure> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Failure::Other(e.to_string()))?;
    let client = client_for(ep).map_err(Failure::Other)?;
    let mut options = ChatOptions::default().with_max_tokens(settings.max_output_tokens);
    if let Some(temperature) = settings.temperature {
        options = options.with_temperature(temperature);
    }
    if let Some(level) = &ep.reasoning {
        let level = level
            .parse::<ReasoningEffort>()
            .map_err(|e| Failure::Other(e.to_string()))?;
        options = options.with_reasoning_effort(level);
    }
    let host = host_of(ep);
    let started = Instant::now();
    let (mut waits, mut waited, mut previous, mut attempts) = (Vec::new(), 0.0_f64, 0.0_f64, 0_u32);
    loop {
        attempts += 1;
        let request = ChatRequest::default()
            .with_system(system)
            .append_message(ChatMessage::user(user));
        let error = match runtime.block_on(client.exec_chat(&ep.model, request, Some(&options))) {
            Ok(reply) => {
                use genai::chat::StopReason;
                let stop = match &reply.stop_reason {
                    Some(StopReason::Completed(_)) => "stop".to_owned(),
                    Some(StopReason::MaxTokens(_)) => "cut_off".to_owned(),
                    Some(other) => format!("{other:?}"),
                    None => "unknown".to_owned(),
                };
                return Ok(Answer {
                    text: reply.first_text().unwrap_or_default().to_owned(),
                    stop,
                    input_tokens: reply.usage.prompt_tokens.unwrap_or(0).into(),
                    output_tokens: reply.usage.completion_tokens.unwrap_or(0).into(),
                    waits,
                    seconds: started.elapsed().as_secs_f64(),
                });
            }
            Err(error) => error,
        };
        let web = match &error {
            genai::Error::WebModelCall { webc_error, .. }
            | genai::Error::WebAdapterCall { webc_error, .. } => webc_error,
            other => return Err(Failure::Other(shortened(&other.to_string()))),
        };
        let Some(status) = web.status().map(|s| s.as_u16()) else {
            return Err(Failure::Other(shortened(&web.to_string())));
        };
        let body = match web {
            genai::webc::Error::ResponseFailedStatus { body, .. } => body.as_str(),
            _ => "",
        };
        if status != 429 && status != 503 {
            return Err(Failure::Refused {
                status,
                said: shortened(body),
            });
        }
        let header = web
            .headers()
            .and_then(|h| h.get("retry-after"))
            .and_then(|v| v.to_str().ok());
        let stated = stated_wait(header, body);
        let (base, source) = stated.unwrap_or((policy.first, "none stated"));
        let next = base.max(previous * 2.0);
        if waited + next > policy.total {
            return Err(Failure::WaitLimit {
                waited,
                limit: policy.total,
                attempts,
            });
        }
        eprintln!(
            "[{tag}] {host} asks to come back later (attempt {attempts}); waiting about {next:.0} s \
             ({source}), {waited:.0} s waited so far"
        );
        std::thread::sleep(Duration::from_secs_f64(next));
        waits.push((next, source));
        waited += next;
        previous = next;
    }
}

// --- the stub endpoint ---------------------------------------------------------

const OK_BODY: &str = r#"{"id":"stub","object":"chat.completion","created":0,"model":"stub","choices":[{"index":0,"message":{"role":"assistant","content":"ok"},"finish_reason":"stop"}],"usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"#;

/// A scripted reply: status, extra header line, body. The last entry repeats.
type Script = Vec<(u16, &'static str, &'static str)>;

/// Serves the script on a local port; returns the address and a counter of
/// requests seen.
fn stub(script: Script) -> (String, Arc<Mutex<usize>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a local port");
    let address = format!("http://{}/v1", listener.local_addr().expect("an address"));
    let seen = Arc::new(Mutex::new(0_usize));
    let counter = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 4096];
            // Read the head, then as many body bytes as it announces.
            let body_start = loop {
                let Ok(n) = stream.read(&mut chunk) else {
                    break None;
                };
                if n == 0 {
                    break None;
                }
                buffer.extend_from_slice(&chunk[..n]);
                if let Some(at) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                    break Some(at + 4);
                }
            };
            let Some(body_start) = body_start else {
                continue;
            };
            let head = String::from_utf8_lossy(&buffer[..body_start]).to_lowercase();
            let length: usize = head
                .lines()
                .find_map(|l| l.strip_prefix("content-length:"))
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0);
            while buffer.len() < body_start + length {
                let Ok(n) = stream.read(&mut chunk) else {
                    break;
                };
                if n == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..n]);
            }
            let index = {
                let mut count = counter.lock().expect("the counter");
                *count += 1;
                (*count - 1).min(script.len() - 1)
            };
            let (status, header, body) = script[index];
            let reply = format!(
                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\n{header}content-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(reply.as_bytes());
        }
    });
    (address, seen)
}

const SAID_ONE: &str = r#"{"error":{"message":"Upstream rate limit persisted; retry after 1 seconds","type":"rate_limit_error"}}"#;
const SAID_NOTHING: &str = r#"{"error":{"message":"slow down"}}"#;
const BAD_REQUEST: &str = r#"{"error":{"message":"the request is not valid here"}}"#;

fn stub_cases() -> bool {
    let policy = Policy {
        first: 1.0,
        total: 6.0,
    };
    // name, script, expected waits (None: the call must fail), expected failure
    let cases: Vec<(&str, Script, Option<Vec<f64>>, &str)> = vec![
        (
            "wait stated in a header",
            vec![
                (429, "retry-after: 2\r\n", SAID_NOTHING),
                (200, "", OK_BODY),
            ],
            Some(vec![2.0]),
            "",
        ),
        (
            "wait stated in the message",
            vec![(429, "", SAID_ONE), (200, "", OK_BODY)],
            Some(vec![1.0]),
            "",
        ),
        (
            "no wait stated, refused twice",
            vec![
                (429, "", SAID_NOTHING),
                (503, "", SAID_NOTHING),
                (200, "", OK_BODY),
            ],
            Some(vec![1.0, 2.0]),
            "",
        ),
        (
            "refused for good: the limit ends it",
            vec![(429, "", SAID_ONE)],
            None,
            "WaitLimit",
        ),
        (
            "refused for another reason: not waited out",
            vec![(400, "", BAD_REQUEST)],
            None,
            "Refused",
        ),
    ];
    let mut all = true;
    for (name, script, expected, failure) in cases {
        let (address, seen) = stub(script);
        let ep = Configured::new("openai", &address, "stub", "not-a-key");
        let started = Instant::now();
        let result = call(&ep, "system", "user", &policy, "stub", &Settings::default());
        let (ok, shown) = match (&result, &expected) {
            (Ok(answer), Some(waits)) => {
                let got: Vec<f64> = answer.waits.iter().map(|w| w.0).collect();
                (
                    got == *waits && answer.text == "ok",
                    json!({"waits": answer.waits.iter().map(|w| json!([w.0, w.1])).collect::<Vec<_>>(), "text": answer.text}),
                )
            }
            (Err(error), None) => {
                let text = format!("{error:?}");
                (text.starts_with(failure), json!({"failure": text}))
            }
            (Ok(answer), None) => (false, json!({"unexpected_answer": answer.text})),
            (Err(error), Some(_)) => (false, json!({"unexpected_failure": format!("{error:?}")})),
        };
        all &= ok;
        println!(
            "{}",
            json!({"case": name, "as_expected": ok, "requests_seen": *seen.lock().expect("the counter"),
                   "seconds": (started.elapsed().as_secs_f64() * 10.0).round() / 10.0, "result": shown})
        );
    }
    all
}

fn live(env_file: &str, system: &str, user: &str, count: usize) -> Result<bool, String> {
    let ep = load_endpoint(Some(Path::new(env_file)))?;
    let (system, user) = (
        std::fs::read_to_string(system).map_err(|e| e.to_string())?,
        std::fs::read_to_string(user).map_err(|e| e.to_string())?,
    );
    let policy = Policy {
        first: 10.0,
        total: 600.0,
    };
    let lines: Vec<Value> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..count)
            .map(|n| {
                let (ep, system, user, policy) = (&ep, &system, &user, &policy);
                let tag = format!("call {n}");
                scope.spawn(move || {
                    match call(ep, system, user, policy, &tag, &Settings::default()) {
                        Ok(answer) => {
                            json!({"call": n, "ok": true, "seconds": answer.seconds.round(),
                        "waits": answer.waits.iter().map(|w| json!([w.0, w.1])).collect::<Vec<_>>(),
                        "reply_chars": answer.text.chars().count()})
                        }
                        Err(error) => {
                            json!({"call": n, "ok": false, "failure": format!("{error:?}")})
                        }
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("a call does not panic"))
            .collect()
    });
    for line in &lines {
        println!("{line}");
    }
    Ok(lines.iter().all(|l| l["ok"] == true))
}

/// The process protocol of gate P4: one request as JSON on standard input,
/// one reply as JSON on standard output, notices and the reason of a failure
/// on standard error. The endpoint is named by its file, never by its key.
fn protocol_call(env_file: &str) -> Result<bool, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    let request: Value = serde_json::from_str(&input).map_err(|e| format!("request: {e}"))?;
    if request["protocol"] != 1 {
        return Err("request: protocol 1 expected".into());
    }
    let text = |key: &str| request[key].as_str().unwrap_or_default().to_owned();
    let settings = Settings {
        max_output_tokens: request["max_output_tokens"].as_u64().unwrap_or(64_000) as u32,
        temperature: request["temperature"].as_f64(),
    };
    let ep = load_endpoint(Some(Path::new(env_file)))?;
    let policy = Policy {
        first: 10.0,
        total: 600.0,
    };
    let reply = match call(
        &ep,
        &text("system"),
        &text("user"),
        &policy,
        "model",
        &settings,
    ) {
        Ok(answer) => json!({"protocol": 1, "ok": true, "reply": {
            "text": answer.text, "stop": answer.stop,
            "input_tokens": answer.input_tokens, "output_tokens": answer.output_tokens,
            "seconds": (answer.seconds * 10.0).round() / 10.0,
            "waits": answer.waits.iter().map(|w| json!([w.0, w.1])).collect::<Vec<_>>(),
            "sent": {"host": host_of(&ep), "model": ep.model,
                     "max_output_tokens": settings.max_output_tokens,
                     "temperature": settings.temperature, "reasoning_level": ep.reasoning},
        }}),
        Err(failure) => {
            let reason = format!("{failure:?}");
            eprintln!("[model] the call failed: {reason}");
            json!({"protocol": 1, "ok": false, "error": {"reason": reason}})
        }
    };
    println!("{reply}");
    Ok(reply["ok"] == true)
}

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let passed = match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [_, "call", env_file] => match protocol_call(env_file) {
            Ok(true) => return std::process::ExitCode::SUCCESS,
            Ok(false) => return std::process::ExitCode::from(3),
            Err(message) => Err(message),
        },
        [_, "stub"] => Ok(stub_cases()),
        [_, "live", env_file, system, user, count] => {
            live(env_file, system, user, count.parse().unwrap_or(1))
        }
        _ => Err(
            "usage: model-seam-prototype-wait stub | live ENV_FILE SYSTEM_TXT USER_TXT COUNT | call ENV_FILE"
                .into(),
        ),
    };
    match passed {
        Ok(true) => std::process::ExitCode::SUCCESS,
        Ok(false) => std::process::ExitCode::from(1),
        Err(message) => {
            eprintln!("{message}");
            std::process::ExitCode::from(2)
        }
    }
}
