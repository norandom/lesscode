//! Whether a reply's text has the structure a caller asked for.
//!
//! The check is made here, on the text that came back. No response format is
//! sent to the endpoint: whether an endpoint accepts one cannot be known
//! without a rule per endpoint, and an unknown setting is refused more often
//! than ignored (R102).

use serde_json::Value;

/// The text without one code fence around the whole of it. Models wrap a
/// JSON reply in a fence on some calls and not on others.
pub fn unfenced(text: &str) -> &str {
    let body = text.trim();
    body.strip_prefix("```")
        .and_then(|rest| rest.strip_suffix("```"))
        .map(|inner| {
            // An optional language word on the fence's first line.
            let after_word = inner.trim_start_matches(|c: char| c.is_ascii_alphanumeric());
            if after_word.starts_with(['\n', '\r']) {
                after_word.trim()
            } else {
                inner.trim()
            }
        })
        .unwrap_or(body)
}

/// Whether `value` satisfies `schema`, for the part of JSON Schema a request
/// for a structure uses: `type`, `required`, `properties` and `items`.
/// Anything else in a schema is not checked.
pub fn satisfies(value: &Value, schema: &Value) -> bool {
    let type_fits = match schema.get("type").and_then(Value::as_str) {
        None => true,
        Some("object") => value.is_object(),
        Some("array") => value.is_array(),
        Some("string") => value.is_string(),
        Some("boolean") => value.is_boolean(),
        Some("null") => value.is_null(),
        Some("integer") => value.is_i64() || value.is_u64(),
        Some("number") => value.is_number(),
        Some(_) => false,
    };
    let required_present = schema
        .get("required")
        .and_then(Value::as_array)
        .is_none_or(|names| {
            names
                .iter()
                .filter_map(Value::as_str)
                .all(|name| value.get(name).is_some())
        });
    let properties_fit = schema
        .get("properties")
        .and_then(Value::as_object)
        .is_none_or(|properties| {
            properties
                .iter()
                .all(|(name, inner)| value.get(name).is_none_or(|found| satisfies(found, inner)))
        });
    let items_fit = match (schema.get("items"), value.as_array()) {
        (Some(inner), Some(items)) => items.iter().all(|item| satisfies(item, inner)),
        _ => true,
    };
    type_fits && required_present && properties_fit && items_fit
}

/// Whether the reply's text, with or without a code fence, is JSON of the
/// structure asked for.
pub fn has_structure(text: &str, schema: &Value) -> bool {
    serde_json::from_str::<Value>(unfenced(text)).is_ok_and(|value| satisfies(&value, schema))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn one_fence_around_the_whole_text_is_removed() {
        assert_eq!(unfenced("```json\n{\"a\": 1}\n```"), "{\"a\": 1}");
        assert_eq!(unfenced("```\n{}\n```\n"), "{}");
        assert_eq!(unfenced(" {\"a\": \"```\"} "), "{\"a\": \"```\"}");
        assert_eq!(unfenced("```{\"a\": 1}```"), "{\"a\": 1}");
    }

    #[test]
    fn type_required_properties_and_items_are_checked() {
        let schema = json!({
            "type": "object", "required": ["files", "note"],
            "properties": {
                "files": {"type": "array", "items": {"type": "object", "required": ["path"]}},
                "note": {"type": "string"},
            },
        });
        assert!(satisfies(
            &json!({"files": [{"path": "a"}], "note": "x", "more": 1}),
            &schema
        ));
        assert!(!satisfies(&json!({"files": [{"path": "a"}]}), &schema));
        assert!(!satisfies(&json!({"files": [{}], "note": "x"}), &schema));
        assert!(!satisfies(&json!({"files": "a", "note": "x"}), &schema));
        assert!(!satisfies(&json!([1]), &schema));
    }
}
