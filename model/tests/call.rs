//! Requirement 2 and the parts of requirement 1 that concern a call, against
//! a local stub endpoint (task 1.1 of the spec model-seam).

mod stub_endpoint;

use lesscode_core::regenerator::Endpoint;
use lesscode_model::{CallRequest, Stop, send};
use stub_endpoint::{chat_reply, chat_reply_in_parts, message_reply, ok, start};

const KEY: &str = "test-key-0123456789";

fn request() -> CallRequest {
    CallRequest {
        purpose: "test".into(),
        system: "Answer briefly.".into(),
        user: "Say ok.".into(),
        max_output_tokens: 100,
        temperature: None,
        structure: None,
    }
}

fn endpoint(style: &str, address: &str) -> Endpoint {
    Endpoint::new(style, address, "stub-model", KEY)
}

#[test]
fn a_local_address_is_called_like_any_other_and_the_reply_is_reported() {
    let stub = start(vec![ok(chat_reply("ok", "stop"))]);
    let reply = send(
        &endpoint("openai", &format!("{}/v1", stub.address)),
        &request(),
    )
    .unwrap();
    assert_eq!(reply.text, "ok");
    assert_eq!(reply.stop, Stop::Complete);
    assert_eq!((reply.input_tokens, reply.output_tokens), (11, 7));
    assert_eq!(reply.reasoning_tokens, Some(5));
    assert_eq!(reply.cached_input_tokens, Some(4));
    assert_eq!(reply.sent.model, "stub-model");
    assert!(reply.sent.host.starts_with("127.0.0.1:"));
    let seen = stub.seen();
    assert_eq!(seen.len(), 1);
    assert!(
        seen[0].line.starts_with("POST /v1/chat/completions "),
        "{}",
        seen[0].line
    );
    assert_eq!(seen[0].json()["model"], "stub-model");
}

#[test]
fn only_stated_settings_go_on_the_wire() {
    let stub = start(vec![ok(chat_reply("ok", "stop"))]);
    let address = format!("{}/v1", stub.address);

    send(&endpoint("openai", &address), &request()).unwrap();
    let plain = stub.seen()[0].json();
    let object = plain.as_object().unwrap();
    for unasked in [
        "temperature",
        "top_p",
        "reasoning_effort",
        "response_format",
        "seed",
    ] {
        assert!(
            !object.contains_key(unasked),
            "{unasked} was sent unasked: {plain}"
        );
    }
    let limit = object
        .get("max_tokens")
        .or_else(|| object.get("max_completion_tokens"))
        .expect("the output limit is sent");
    assert_eq!(limit, 100);

    let mut stated = endpoint("openai", &address);
    stated.reasoning = Some("high".into());
    let mut asked = request();
    asked.temperature = Some(0.3);
    let reply = send(&stated, &asked).unwrap();
    let body = stub.seen()[1].json();
    assert_eq!(body["temperature"], 0.3, "{body}");
    assert_eq!(body["reasoning_effort"], "high", "{body}");
    assert_eq!(reply.sent.temperature, Some(0.3));
    assert_eq!(reply.sent.reasoning_level.as_deref(), Some("high"));
}

#[test]
fn text_in_parts_is_joined_and_the_reasoning_is_left_out() {
    let stub = start(vec![ok(chat_reply_in_parts(&["{\"a\": ", "1}"]))]);
    let reply = send(
        &endpoint("openai", &format!("{}/v1", stub.address)),
        &request(),
    )
    .unwrap();
    assert_eq!(reply.text, "{\"a\": 1}");
    assert!(!reply.text.contains("reasoning"));
}

#[test]
fn a_reply_cut_off_at_the_limit_is_reported_as_cut_off() {
    let stub = start(vec![ok(chat_reply("half an ans", "length"))]);
    let reply = send(
        &endpoint("openai", &format!("{}/v1", stub.address)),
        &request(),
    )
    .unwrap();
    assert_eq!(reply.stop, Stop::CutOff);
    assert_eq!(reply.text, "half an ans");
}

#[test]
fn the_second_style_gets_its_version_segment_when_the_address_has_none() {
    for suffix in ["", "/v1", "/v1/"] {
        let stub = start(vec![ok(message_reply("ok"))]);
        let reply = send(
            &endpoint("anthropic", &format!("{}{suffix}", stub.address)),
            &request(),
        )
        .unwrap();
        assert_eq!(reply.text, "ok");
        assert_eq!(reply.stop, Stop::Complete);
        assert_eq!((reply.input_tokens, reply.output_tokens), (9, 3));
        let line = stub.seen()[0].line.clone();
        assert!(
            line.starts_with("POST /v1/messages "),
            "address suffix {suffix:?}: {line}"
        );
    }
}

#[test]
fn an_unknown_style_is_refused_before_any_call() {
    let stub = start(vec![ok(chat_reply("ok", "stop"))]);
    let error = send(&endpoint("carrier-pigeon", &stub.address), &request()).unwrap_err();
    assert!(
        format!("{error:?}").contains("openai, anthropic"),
        "{error:?}"
    );
    assert!(stub.seen().is_empty());
}

/// Requirements 1.8 and 2.8: the seam's source names no model and holds no
/// address. The names below are the first cases of requirement 2.7.
#[test]
fn the_source_names_no_model_and_holds_no_address() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut checked = 0;
    for entry in std::fs::read_dir(source).unwrap().flatten() {
        let text = std::fs::read_to_string(entry.path())
            .unwrap()
            .to_lowercase();
        for word in [
            "http://", "https://", "gpt-", "claude", "glm", "mistral", "opus", "sonnet",
        ] {
            assert!(
                !text.contains(word),
                "{} contains {word:?}",
                entry.path().display()
            );
        }
        checked += 1;
    }
    assert!(checked >= 4, "only {checked} source files were read");
}

/// Requirement 2.6: a reply in a stated structure, or the report that it has not.
#[test]
fn a_reply_says_whether_it_has_the_structure_asked_for() {
    let schema = serde_json::json!({"type": "object", "required": ["files", "change_log"]});
    let good = r#"{"files": [], "change_log": "none"}"#;
    let fenced = format!("```json\n{good}\n```");
    let stub = start(vec![
        ok(chat_reply(good, "stop")),
        ok(chat_reply(&fenced, "stop")),
        ok(chat_reply(r#"{"files": []}"#, "stop")),
        ok(chat_reply("Here is what I would do.", "stop")),
        ok(chat_reply(good, "stop")),
    ]);
    let address = endpoint("openai", &format!("{}/v1", stub.address));
    let mut asked = request();
    asked.structure = Some(schema);
    let outcomes: Vec<Option<bool>> = (0..4)
        .map(|_| send(&address, &asked).unwrap().structured)
        .collect();
    assert_eq!(outcomes, [Some(true), Some(true), Some(false), Some(false)]);
    // Nothing about the structure goes on the wire, and without a structure nothing is said.
    assert!(
        stub.seen()
            .iter()
            .all(|seen| seen.json().get("response_format").is_none())
    );
    assert_eq!(send(&address, &request()).unwrap().structured, None);
}
