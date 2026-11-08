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
