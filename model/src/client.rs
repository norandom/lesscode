//! One request to an endpoint through the client library, and its reply
//! read in whatever shape it arrives. Nothing here names a vendor or a model:
//! the request style picks the library's adapter, and address, model and key
//! are data from the endpoint file.

use crate::error::{CallError, shown};
use crate::request::{CallRequest, Reply, Sent, Stop};
use crate::structure::has_structure;
use genai::adapter::AdapterKind;
use genai::chat::{
    ChatMessage, ChatOptions, ChatRequest, ChatResponse, ReasoningEffort, StopReason,
};
use genai::resolver::{AuthData, Endpoint as Address, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};
use lesscode_core::regenerator::Endpoint;
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

/// The host part of the endpoint's address, for messages and records.
pub fn host_of(endpoint: &Endpoint) -> String {
    let rest = endpoint
        .base_url
        .split_once("://")
        .map_or(endpoint.base_url.as_str(), |(_, rest)| rest);
    rest.split('/').next().unwrap_or(rest).to_owned()
}

/// The address the library is given. The second request style names its
/// version segment in the address; an endpoint file may leave it out.
fn address_of(endpoint: &Endpoint) -> String {
    let trimmed = endpoint.base_url.trim_end_matches('/');
    if endpoint.style == "anthropic" && !trimmed.ends_with("/v1") {
        format!("{trimmed}/v1/")
    } else {
        format!("{trimmed}/")
    }
}

fn client_for(endpoint: &Endpoint) -> Result<Client, CallError> {
    let kind = match endpoint.style.as_str() {
        "openai" => AdapterKind::OpenAI,
        "anthropic" => AdapterKind::Anthropic,
        other => {
            return Err(CallError::Config(format!(
                "request style {other:?} is not known; known styles: openai, anthropic"
            )));
        }
    };
    let (address, key) = (address_of(endpoint), endpoint.api_key().to_owned());
    let resolver = ServiceTargetResolver::from_resolver_fn(
        move |target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
            Ok(ServiceTarget {
                endpoint: Address::from_owned(address.clone()),
                auth: AuthData::from_single(key.clone()),
                model: ModelIden::new(kind, target.model.model_name),
            })
        },
    );
    Client::builder()
        .with_service_target_resolver(resolver)
        .build()
        .map_err(|e| CallError::Config(shown(&e.to_string(), endpoint.api_key())))
}

/// The settings that go on the wire: the output limit always, the rest only
/// when the request or the endpoint states it.
fn options_for(endpoint: &Endpoint, request: &CallRequest) -> Result<ChatOptions, CallError> {
    let mut options = ChatOptions::default().with_max_tokens(request.max_output_tokens);
    if let Some(temperature) = request.temperature {
        options = options.with_temperature(temperature);
    }
    if let Some(level) = &endpoint.reasoning {
        let level = level
            .parse::<ReasoningEffort>()
            .map_err(|_| CallError::Config(format!("reasoning level {level:?} is not known")))?;
        options = options.with_reasoning_effort(level);
    }
    Ok(options)
}

/// What a failed attempt was: not reached, refused, or answered unreadably.
/// Text of the endpoint or the transport passes through `shown` with the key.
fn failure(host: String, key: &str, error: genai::Error) -> CallError {
    let web = match &error {
        genai::Error::WebModelCall { webc_error, .. }
        | genai::Error::WebAdapterCall { webc_error, .. } => webc_error,
        other => {
            return CallError::Unreadable {
                host,
                detail: shown(&other.to_string(), key),
            };
        }
    };
    let Some(status) = web.status() else {
        let detail = shown(&web.to_string(), key);
        return match web {
            genai::webc::Error::Reqwest(_) => CallError::Unreachable { host, detail },
            _ => CallError::Unreadable { host, detail },
        };
    };
    let said = match web {
        genai::webc::Error::ResponseFailedStatus { body, .. } => shown(body, key),
        _ => String::new(),
    };
    let retry_after = web
        .headers()
        .and_then(|headers| headers.get("retry-after"))
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    CallError::Refused {
        host,
        status: status.as_u16(),
        said,
        retry_after,
    }
}

fn count(tokens: Option<i32>) -> u64 {
    tokens.unwrap_or(0).max(0) as u64
}

/// One request on its way: sent once, its answer not yet read. Dropping it
/// gives the request up.
pub struct Attempt {
    answer: Pin<Box<dyn Future<Output = Result<Reply, CallError>>>>,
    runtime: tokio::runtime::Runtime,
}

impl Attempt {
    /// Waits for the answer for at most `step`. `None`: it is not there yet,
    /// and the request stays on its way.
    pub fn answer_within(&mut self, step: Duration) -> Option<Result<Reply, CallError>> {
        let answer = self.answer.as_mut();
        self.runtime
            .block_on(async { tokio::time::timeout(step, answer).await.ok() })
    }

    /// Waits for the answer however long it takes.
    pub fn answer(self) -> Result<Reply, CallError> {
        self.runtime.block_on(self.answer)
    }
}

/// Starts one attempt. A refusal comes back from the attempt as
/// `CallError::Refused` with what the endpoint said; waiting it out and
/// deciding how long an answer may take are the caller's part.
pub fn begin(endpoint: &Endpoint, request: &CallRequest) -> Result<Attempt, CallError> {
    let client = client_for(endpoint)?;
    let options = options_for(endpoint, request)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CallError::Config(e.to_string()))?;
    let mut chat = ChatRequest::default();
    if !request.system.is_empty() {
        chat = chat.with_system(request.system.as_str());
    }
    let chat = chat.append_message(ChatMessage::user(request.user.as_str()));
    let sent = Sent {
        host: host_of(endpoint),
        model: endpoint.model.clone(),
        max_output_tokens: request.max_output_tokens,
        temperature: request.temperature,
        reasoning_level: endpoint.reasoning.clone(),
    };
    let (key, structure) = (endpoint.api_key().to_owned(), request.structure.clone());
    let answer = Box::pin(async move {
        let started = Instant::now();
        let reply = client
            .exec_chat(&sent.model, chat, Some(&options))
            .await
            .map_err(|error| failure(sent.host.clone(), &key, error))?;
        let reply = read(reply, sent);
        Ok(Reply {
            structured: structure
                .as_ref()
                .map(|schema| has_structure(&reply.text, schema)),
            seconds: started.elapsed().as_secs_f64(),
            ..reply
        })
    });
    Ok(Attempt { answer, runtime })
}

/// One attempt, awaited to its end.
pub fn send(endpoint: &Endpoint, request: &CallRequest) -> Result<Reply, CallError> {
    begin(endpoint, request)?.answer()
}

/// The reply as the seam reports it, whatever shape it arrived in.
fn read(reply: ChatResponse, sent: Sent) -> Reply {
    let stop = match &reply.stop_reason {
        Some(StopReason::Completed(_)) => Stop::Complete,
        Some(StopReason::MaxTokens(_)) => Stop::CutOff,
        Some(StopReason::ContentFilter(_)) => Stop::Refused,
        Some(StopReason::ToolCall(word))
        | Some(StopReason::StopSequence(word))
        | Some(StopReason::Other(word)) => Stop::Other(word.clone()),
        None => Stop::Other("not stated".to_owned()),
    };
    let usage = &reply.usage;
    let text = reply.texts().concat();
    Reply {
        structured: None,
        text,
        stop,
        input_tokens: count(usage.prompt_tokens),
        output_tokens: count(usage.completion_tokens),
        reasoning_tokens: usage
            .completion_tokens_details
            .as_ref()
            .and_then(|d| d.reasoning_tokens)
            .map(|n| count(Some(n))),
        cached_input_tokens: usage
            .prompt_tokens_details
            .as_ref()
            .and_then(|d| d.cached_tokens)
            .map(|n| count(Some(n))),
        seconds: 0.0,
        waits: Vec::new(),
        sent,
    }
}
