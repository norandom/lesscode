//! A local stub service for the seam's tests: it answers from a script and
//! keeps what it was sent, so a test can look at the request as it was on
//! the wire. No test of this crate calls a real endpoint.
#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

/// One scripted answer: status, extra header lines (each ending in `\r\n`),
/// body. The last entry of a script repeats.
pub type Answer = (u16, &'static str, String);

/// What the stub received: the request line and the body.
#[derive(Clone, Debug)]
pub struct Seen {
    pub line: String,
    pub head: String,
    pub body: String,
}

impl Seen {
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).expect("the request body is JSON")
    }
}

pub struct Stub {
    /// The address to put into an endpoint, without a version segment.
    pub address: String,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl Stub {
    pub fn seen(&self) -> Vec<Seen> {
        self.seen.lock().expect("the list of requests").clone()
    }
}

/// A reply in the first request style, with the text as one string.
pub fn chat_reply(text: &str, finish: &str) -> String {
    serde_json::json!({
        "id": "stub", "object": "chat.completion", "created": 0, "model": "stub-model",
        "choices": [{"index": 0, "message": {"role": "assistant", "content": text}, "finish_reason": finish}],
        "usage": {"prompt_tokens": 11, "completion_tokens": 7, "total_tokens": 18,
                  "prompt_tokens_details": {"cached_tokens": 4},
                  "completion_tokens_details": {"reasoning_tokens": 5}},
    })
    .to_string()
}

/// The same with the text delivered in parts, the model's reasoning first.
pub fn chat_reply_in_parts(parts: &[&str]) -> String {
    let mut content = vec![serde_json::json!({
        "type": "thinking", "thinking": [{"type": "text", "text": "reasoning that is not the answer"}]
    })];
    content.extend(
        parts
            .iter()
            .map(|part| serde_json::json!({"type": "text", "text": part})),
    );
    serde_json::json!({
        "id": "stub", "object": "chat.completion", "created": 0, "model": "stub-model",
        "choices": [{"index": 0, "message": {"role": "assistant", "content": content}, "finish_reason": "stop"}],
        "usage": {"prompt_tokens": 11, "completion_tokens": 7, "total_tokens": 18},
    })
    .to_string()
}

/// A reply in the second request style.
pub fn message_reply(text: &str) -> String {
    serde_json::json!({
        "id": "stub", "type": "message", "role": "assistant", "model": "stub-model",
        "content": [{"type": "text", "text": text}],
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 9, "output_tokens": 3},
    })
    .to_string()
}

pub fn ok(body: String) -> Answer {
    (200, "", body)
}

/// Serves the script on a local port until the test process ends.
pub fn start(script: Vec<Answer>) -> Stub {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a local port");
    let address = format!("http://{}", listener.local_addr().expect("an address"));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let list = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 4096];
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
            let head = String::from_utf8_lossy(&buffer[..body_start]).into_owned();
            let length: usize = head
                .to_lowercase()
                .lines()
                .find_map(|l| l.strip_prefix("content-length:").map(str::to_owned))
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
                let mut list = list.lock().expect("the list of requests");
                list.push(Seen {
                    line: head.lines().next().unwrap_or_default().to_owned(),
                    head: head.clone(),
                    body: String::from_utf8_lossy(&buffer[body_start..]).into_owned(),
                });
                (list.len() - 1).min(script.len() - 1)
            };
            let (status, headers, body) = &script[index];
            let reply = format!(
                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\n{headers}content-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(reply.as_bytes());
        }
    });
    Stub { address, seen }
}
