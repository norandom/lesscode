//! Prototype gate for the model seam: the turn that `lesscode-core regenerate`
//! sends, through an existing model client library (genai) in place of the
//! hand-written request. The endpoint comes from an endpoint file, read by
//! the core's own loader; nothing about a vendor is written here.
//!
//!     model-seam-prototype ENV_FILE SYSTEM_TXT USER_TXT
//!
//! Prints one JSON line: stop reason, tokens, whether the reply is the JSON
//! object the turn asks for, and its first characters.

use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest, ReasoningEffort};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};
use lesscode_core::regenerator::load_endpoint;
use serde_json::{Value, json};
use std::path::Path;
use std::time::Instant;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let [_, env_file, system, user] = args.as_slice() else {
        return Err("usage: model-seam-prototype ENV_FILE SYSTEM_TXT USER_TXT".into());
    };
    let ep = load_endpoint(Some(Path::new(env_file)))?;
    let kind = if ep.style == "anthropic" {
        AdapterKind::Anthropic
    } else {
        AdapterKind::OpenAI
    };
    let (base, key) = (
        format!("{}/", ep.base_url.trim_end_matches('/')),
        ep.api_key().to_owned(),
    );
    let resolver = ServiceTargetResolver::from_resolver_fn(
        move |target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
            Ok(ServiceTarget {
                endpoint: Endpoint::from_owned(base.clone()),
                auth: AuthData::from_single(key.clone()),
                model: ModelIden::new(kind, target.model.model_name),
            })
        },
    );
    let client = Client::builder()
        .with_service_target_resolver(resolver)
        .build()?;

    // Only what the endpoint file states is sent: no temperature, and a
    // reasoning level only when the file names one.
    let mut options = ChatOptions::default().with_max_tokens(64_000);
    if let Some(level) = &ep.reasoning {
        options = options.with_reasoning_effort(level.parse::<ReasoningEffort>()?);
    }
    let request = ChatRequest::default()
        .with_system(std::fs::read_to_string(system)?)
        .append_message(ChatMessage::user(std::fs::read_to_string(user)?));

    let started = Instant::now();
    let line = match client.exec_chat(&ep.model, request, Some(&options)).await {
        Err(error) => json!({"ok": false, "model": ep.model, "error": error.to_string()}),
        Ok(reply) => {
            let text = reply.first_text().unwrap_or_default().to_owned();
            let body = text
                .trim()
                .trim_start_matches("```json")
                .trim_matches('`')
                .trim();
            let parsed: Option<Value> = serde_json::from_str(body).ok();
            json!({
                "ok": true, "model": ep.model,
                "seconds": started.elapsed().as_secs(),
                "stop_reason": format!("{:?}", reply.stop_reason),
                "input_tokens": reply.usage.prompt_tokens,
                "output_tokens": reply.usage.completion_tokens,
                "reasoning_seen": reply.reasoning_content.is_some(),
                "reply_chars": text.chars().count(),
                "reply_is_json_object": parsed.as_ref().is_some_and(Value::is_object),
                "keys": parsed.as_ref().and_then(Value::as_object).map(|o| o.keys().cloned().collect::<Vec<_>>()),
                "reply_start": text.chars().take(160).collect::<String>(),
            })
        }
    };
    println!("{line}");
    Ok(())
}
