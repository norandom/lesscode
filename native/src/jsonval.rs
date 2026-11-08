//! Minimal order-preserving JSON value for the fixed receipt schema,
//! extracted from `receipt.rs` in the task-9.3 defect-7 split (R37):
//! value type here, `json.dumps(indent=1)`-parity writer in `receipt.rs`.

/// Minimal order-preserving JSON value for the fixed receipt schema.
#[derive(Debug, Clone, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    Int(i64),
    /// Finite floats only (shares/ratios); formatted like Python `json.dumps`.
    Float(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

// Eq holds by contract: `Float` carries finite rounded ratios only, never NaN.
impl Eq for J {}

impl J {
    pub fn str(s: impl Into<String>) -> J {
        J::Str(s.into())
    }

    pub fn int(n: i64) -> J {
        J::Int(n)
    }

    pub fn obj(pairs: Vec<(&str, J)>) -> J {
        J::Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    pub fn arr(items: Vec<J>) -> J {
        J::Arr(items)
    }
}

use std::fmt::Write as _;

/// Object field lookup (first matching key; insertion order preserved).
pub fn j_get<'a>(v: &'a J, key: &str) -> Option<&'a J> {
    match v {
        J::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
        _ => None,
    }
}

/// String value, `""` for non-strings (missing key reads empty).
pub fn j_str(v: &J) -> &str {
    match v {
        J::Str(s) => s,
        _ => "",
    }
}

/// serde_json value to `J` (object keys arrive sorted; floats survive).
pub fn from_serde(value: &serde_json::Value) -> J {
    match value {
        serde_json::Value::Null => J::Null,
        serde_json::Value::Bool(b) => J::Bool(*b),
        serde_json::Value::Number(n) => match n.as_i64() {
            Some(i) => J::Int(i),
            None => J::Float(n.as_f64().unwrap_or(0.0)),
        },
        serde_json::Value::String(s) => J::str(s.clone()),
        serde_json::Value::Array(items) => J::arr(items.iter().map(from_serde).collect()),
        serde_json::Value::Object(map) => J::Obj(
            map.iter()
                .map(|(k, v)| (k.clone(), from_serde(v)))
                .collect(),
        ),
    }
}

/// `J` to serde_json value (maps are BTreeMaps; the HTTP body is not hashed).
pub fn to_serde(value: &J) -> serde_json::Value {
    match value {
        J::Null => serde_json::Value::Null,
        J::Bool(b) => serde_json::Value::Bool(*b),
        J::Int(n) => serde_json::Value::Number((*n).into()),
        J::Float(x) => serde_json::Number::from_f64(*x)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        J::Str(s) => serde_json::Value::String(s.clone()),
        J::Arr(items) => serde_json::Value::Array(items.iter().map(to_serde).collect()),
        J::Obj(pairs) => {
            let mut map = serde_json::Map::new();
            for (k, v) in pairs {
                map.insert(k.clone(), to_serde(v));
            }
            serde_json::Value::Object(map)
        }
    }
}

/// Python `json.dumps` string escaping; `ascii` selects `ensure_ascii`
/// (non-ASCII raw when false, `\uxxxx` otherwise). Shared kernel: the
/// receipt writer pins `ascii = true`, the regenerator prompt writers
/// pass through.
pub fn write_json_str_py(s: &str, out: &mut String, ascii: bool) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if c.is_ascii_graphic() || c == ' ' => out.push(c),
            c if !ascii => out.push(c),
            c => {
                let cp = c as u32;
                if cp <= 0xFFFF {
                    let _ = write!(out, "\\u{cp:04x}");
                } else {
                    let cp = cp - 0x10000;
                    let hi = 0xD800 + (cp >> 10);
                    let lo = 0xDC00 + (cp & 0x3FF);
                    let _ = write!(out, "\\u{hi:04x}\\u{lo:04x}");
                }
            }
        }
    }
    out.push('"');
}
