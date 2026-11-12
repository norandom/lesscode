//! Prototype gate P1 for the model seam: the same turn as `../prototype`,
//! through a second client library (`llm`), for comparison.
//!
//!     model-seam-prototype-llm ENV_FILE SYSTEM_TXT USER_TXT

use lesscode_core::regenerator::load_endpoint;
use llm::builder::{LLMBackend, LLMBuilder};
use llm::chat::{ChatMessage, ReasoningEffort};
use serde_json::{Value, json};
use std::path::Path;
use std::time::Instant;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let [_, env_file, system, user] = args.as_slice() else {
        return Err("usage: model-seam-prototype-llm ENV_FILE SYSTEM_TXT USER_TXT".into());
    };
    let ep = load_endpoint(Some(Path::new(env_file)))?;
    let trimmed = ep.base_url.trim_end_matches('/');
    let (backend, base) = if ep.style == "anthropic" {
        let with_version = if trimmed.ends_with("/v1") {
            format!("{trimmed}/")
        } else {
            format!("{trimmed}/v1/")
        };
        (LLMBackend::Anthropic, with_version)
    } else {
        // The crate's OpenAI backend speaks only the Responses API, which the other
        // OpenAI-style endpoints do not serve. Its Mistral backend is the one that
        // sends plain chat completions to a given address.
        (LLMBackend::Mistral, format!("{trimmed}/"))
    };
    let mut builder = LLMBuilder::new()
        .backend(backend)
        .api_key(ep.api_key())
        .base_url(base)
        .model(&ep.model)
        .max_tokens(64_000)
        .timeout_seconds(3600)
        .system(std::fs::read_to_string(system)?);
    if let Some(level) = &ep.reasoning {
        builder = builder.reasoning_effort(match level.as_str() {
            "low" => ReasoningEffort::Low,
            "medium" => ReasoningEffort::Medium,
            _ => ReasoningEffort::High,
        });
    }
    let client = builder.build()?;
    let messages = vec![
        ChatMessage::user()
            .content(std::fs::read_to_string(user)?)
            .build(),
    ];

    let started = Instant::now();
    let line = match client.chat(&messages).await {
        Err(error) => json!({"ok": false, "model": ep.model, "error": error.to_string()}),
        Ok(reply) => {
            let text = reply.text().unwrap_or_default();
            let body = text
                .trim()
                .trim_start_matches("```json")
                .trim_matches('`')
                .trim();
            let parsed: Option<Value> = serde_json::from_str(body).ok();
            let usage = reply.usage();
            json!({
                "ok": true, "model": ep.model,
                "seconds": started.elapsed().as_secs(),
                "input_tokens": usage.as_ref().map(|u| u.prompt_tokens),
                "output_tokens": usage.as_ref().map(|u| u.completion_tokens),
                "reasoning_seen": reply.thinking().is_some(),
                "reply_chars": text.chars().count(),
                "reply_is_json_object": parsed.as_ref().is_some_and(Value::is_object),
                "keys": parsed.as_ref().and_then(Value::as_object).map(|o| o.keys().cloned().collect::<Vec<_>>()),
                "reply_start": text.chars().take(120).collect::<String>(),
            })
        }
    };
    println!("{line}");
    Ok(())
}
