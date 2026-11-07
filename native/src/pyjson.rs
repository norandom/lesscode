//! Python-compatible JSON model, parser and writer for digest parity (S2).
//!
//! Reproduces the exact byte shapes of the frozen deconvolution apparatus:
//! `json.dumps(v, sort_keys=True, separators=(",", ":"))` (e1b_oracle.py canon),
//! `json.dumps(v, sort_keys=True)` with default separators (e4_oracle.py digests)
//! and the golden/receipt writers using `indent` with `allow_nan=True`. Number
//! tokens are preserved verbatim on parse, so re-emission is byte-identical by
//! construction; Python `repr` rules are implemented for Rust-constructed floats.

use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// Order- and token-preserving JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    /// Integer literal with digits verbatim (arbitrary precision preserved).
    Int(String),
    /// Float or NaN/Infinity literal with the token exactly as emitted.
    Float(String),
    Str(String),
    Arr(Vec<J>),
    /// Object pairs in stored order; writers sort pairs only on demand.
    Obj(Vec<(String, J)>),
}

impl J {
    pub fn str(s: impl Into<String>) -> J {
        J::Str(s.into())
    }
    pub fn int(i: i64) -> J {
        J::Int(i.to_string())
    }
    /// Formats via Python `repr` rules (`-0.0` kept, `NaN`/`Infinity` tokens).
    pub fn f64(x: f64) -> J {
        J::Float(py_repr_f64(x))
    }
    pub fn obj(pairs: Vec<(&str, J)>) -> J {
        J::Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
}

/// Python `repr` formatting for floats, including `NaN`/`Infinity` tokens.
pub fn py_repr_f64(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity" } else { "-Infinity" }.into();
    }
    let neg = x.is_sign_negative();
    let a = x.abs();
    if a == 0.0 {
        return if neg { "-0.0" } else { "0.0" }.into();
    }
    // Rust LowerExp gives the shortest round-trip digits; reshape to Python rules:
    // fixed notation iff -4 <= exp10 < 16, otherwise e-notation with an explicit
    // exponent sign and at least two exponent digits (1e-07, 1e+16, 1e+100).
    let sci = format!("{a:e}");
    let (mant, exp) = sci
        .split_once('e')
        .expect("LowerExp always has an exponent");
    let exp: i32 = exp.parse().expect("valid exponent");
    let mut digits = mant.chars().filter(|c| *c != '.').collect::<String>();
    let trailing = digits.len() - digits.trim_end_matches('0').len();
    digits.truncate(digits.len() - trailing);
    let sign = if neg { "-" } else { "" };
    if (-4..16).contains(&exp) {
        let body = if exp >= 0 {
            let int_len = (exp + 1) as usize;
            if digits.len() > int_len {
                let (int_part, frac) = digits.split_at(int_len);
                format!("{int_part}.{frac}")
            } else {
                let mut int_part = digits.clone();
                int_part.push_str(&"0".repeat(int_len - digits.len()));
                int_part.push_str(".0");
                int_part
            }
        } else {
            format!("0.{}{digits}", "0".repeat((-exp - 1) as usize))
        };
        format!("{sign}{body}")
    } else {
        let mantissa = if digits.len() > 1 {
            format!("{}.{}", &digits[..1], &digits[1..])
        } else {
            digits
        };
        format!(
            "{sign}{mantissa}e{}{:02}",
            if exp < 0 { '-' } else { '+' },
            exp.abs()
        )
    }
}

/// Emission shapes matching the apparatus' `json.dumps` calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fmt {
    /// `separators=(",", ":")` — e1b_oracle.py canonical observation form.
    Canon,
    /// Default separators `(", ", ": ")` — e4_oracle.py original-sensitivity digests.
    DefaultCompact,
    /// `indent=width` golden/receipt form; key separator `": "`, item separator
    /// `","` + newline + `width * depth` spaces; `sort_keys` optional.
    Indent { width: usize, sort_keys: bool },
}

fn escape_py(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7e => {
                let cp = c as u32;
                if cp <= 0xffff {
                    let _ = write!(out, "\\u{cp:04x}");
                } else {
                    let c = cp - 0x10000;
                    let _ = write!(
                        out,
                        "\\u{:04x}\\u{:04x}",
                        0xd800 + (c >> 10),
                        0xdc00 + (c & 0x3ff)
                    );
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn scalar(v: &J, out: &mut String) {
    match v {
        J::Null => out.push_str("null"),
        J::Bool(true) => out.push_str("true"),
        J::Bool(false) => out.push_str("false"),
        J::Int(token) | J::Float(token) => out.push_str(token),
        J::Str(s) => escape_py(s, out),
        _ => unreachable!("scalar called on a container"),
    }
}

/// Serializes `v` under `fmt` into `out` exactly as the Python writers would.
pub fn write(v: &J, fmt: Fmt, out: &mut String) {
    write_at(v, fmt, out, 0);
}

fn write_at(v: &J, fmt: Fmt, out: &mut String, depth: usize) {
    let (item_sep, key_sep, indent) = match fmt {
        Fmt::Canon => (",", ":", 0usize),
        Fmt::DefaultCompact => (", ", ": ", 0),
        Fmt::Indent { width, .. } => (",", ": ", width),
    };
    match v {
        J::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(item_sep);
                }
                if let Fmt::Indent { .. } = fmt {
                    out.push('\n');
                    push_indent(out, indent, depth + 1);
                }
                write_at(item, fmt, out, depth + 1);
            }
            if let Fmt::Indent { .. } = fmt {
                out.push('\n');
                push_indent(out, indent, depth);
            }
            out.push(']');
        }
        J::Obj(pairs) => {
            if pairs.is_empty() {
                out.push_str("{}");
                return;
            }
            let sorted = if matches!(fmt, Fmt::Canon | Fmt::DefaultCompact)
                || matches!(
                    fmt,
                    Fmt::Indent {
                        sort_keys: true,
                        ..
                    }
                ) {
                let mut sorted = pairs.clone();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                Some(sorted)
            } else {
                None
            };
            let pairs: &[(String, J)] = sorted.as_ref().map_or(pairs.as_slice(), |s| s.as_slice());
            out.push('{');
            for (i, (key, value)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push_str(item_sep);
                }
                if let Fmt::Indent { .. } = fmt {
                    out.push('\n');
                    push_indent(out, indent, depth + 1);
                }
                escape_py(key, out);
                out.push_str(key_sep);
                write_at(value, fmt, out, depth + 1);
            }
            if let Fmt::Indent { .. } = fmt {
                out.push('\n');
                push_indent(out, indent, depth);
            }
            out.push('}');
        }
        _ => scalar(v, out),
    }
}

fn push_indent(out: &mut String, width: usize, depth: usize) {
    for _ in 0..width * depth {
        out.push(' ');
    }
}

/// Parses Python-emitted JSON, including `NaN`/`Infinity` literals, preserving
/// object order and number tokens. Lone surrogates fail closed.
pub fn parse(input: &str) -> Result<J, String> {
    let mut p = Parser {
        b: input.as_bytes(),
        i: 0,
    };
    p.ws();
    let v = p.value(0)?;
    p.ws();
    if p.i != p.b.len() {
        return Err(format!("trailing bytes at offset {}", p.i));
    }
    Ok(v)
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }
    fn lit(&mut self, token: &str) -> bool {
        if self.b[self.i..].starts_with(token.as_bytes()) {
            self.i += token.len();
            true
        } else {
            false
        }
    }
    fn value(&mut self, depth: usize) -> Result<J, String> {
        if depth > 512 {
            return Err("nesting deeper than 512 levels".into());
        }
        match self.b.get(self.i) {
            None => Err("unexpected end of input".into()),
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => Ok(J::Str(self.string()?)),
            Some(b't') if self.lit("true") => Ok(J::Bool(true)),
            Some(b'f') if self.lit("false") => Ok(J::Bool(false)),
            Some(b'n') if self.lit("null") => Ok(J::Null),
            Some(b'N') if self.lit("NaN") => Ok(J::Float("NaN".into())),
            Some(b'I') if self.lit("Infinity") => Ok(J::Float("Infinity".into())),
            Some(b'-') if self.b[self.i..].starts_with(b"-Infinity") => {
                self.i += "-Infinity".len();
                Ok(J::Float("-Infinity".into()))
            }
            Some(c) if c.is_ascii_digit() || *c == b'-' => self.number(),
            Some(c) => Err(format!("unexpected byte {:#04x} at offset {}", c, self.i)),
        }
    }
    fn object(&mut self, depth: usize) -> Result<J, String> {
        self.i += 1; // '{'
        let mut pairs = Vec::new();
        self.ws();
        if self.b.get(self.i) == Some(&b'}') {
            self.i += 1;
            return Ok(J::Obj(pairs));
        }
        loop {
            self.ws();
            if self.b.get(self.i) != Some(&b'"') {
                return Err(format!("expected key string at offset {}", self.i));
            }
            let key = self.string()?;
            self.ws();
            if self.b.get(self.i) != Some(&b':') {
                return Err(format!("expected ':' at offset {}", self.i));
            }
            self.i += 1;
            self.ws();
            pairs.push((key, self.value(depth + 1)?));
            self.ws();
            match self.b.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(J::Obj(pairs));
                }
                _ => return Err(format!("expected ',' or '}}' at offset {}", self.i)),
            }
        }
    }
    fn array(&mut self, depth: usize) -> Result<J, String> {
        self.i += 1; // '['
        let mut items = Vec::new();
        self.ws();
        if self.b.get(self.i) == Some(&b']') {
            self.i += 1;
            return Ok(J::Arr(items));
        }
        loop {
            self.ws();
            items.push(self.value(depth + 1)?);
            self.ws();
            match self.b.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(J::Arr(items));
                }
                _ => return Err(format!("expected ',' or ']' at offset {}", self.i)),
            }
        }
    }
    fn number(&mut self) -> Result<J, String> {
        let start = self.i;
        if self.b.get(self.i) == Some(&b'-') {
            self.i += 1;
        }
        let mut float = false;
        loop {
            match self.b.get(self.i) {
                Some(c) if c.is_ascii_digit() => self.i += 1,
                Some(b'.') => {
                    self.i += 1;
                    float = true;
                }
                Some(b'e' | b'E') => {
                    self.i += 1;
                    float = true;
                    if matches!(self.b.get(self.i), Some(b'+' | b'-')) {
                        self.i += 1;
                    }
                }
                _ => break,
            }
        }
        let token = std::str::from_utf8(&self.b[start..self.i]).expect("ASCII bytes");
        if token == "-" {
            return Err("bare minus is not a number".into());
        }
        Ok(if float {
            J::Float(token.into())
        } else {
            J::Int(token.into())
        })
    }
    fn string(&mut self) -> Result<String, String> {
        self.i += 1; // opening quote
        let mut out = String::new();
        loop {
            match self.b.get(self.i) {
                None => return Err("unterminated string".into()),
                Some(b'"') => {
                    self.i += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.i += 1;
                    match self.b.get(self.i) {
                        Some(b'"') => out.push('"'),
                        Some(b'\\') => out.push('\\'),
                        Some(b'/') => out.push('/'),
                        Some(b'b') => out.push('\u{08}'),
                        Some(b'f') => out.push('\u{0c}'),
                        Some(b'n') => out.push('\n'),
                        Some(b'r') => out.push('\r'),
                        Some(b't') => out.push('\t'),
                        Some(b'u') => {
                            self.i += 1;
                            let first = self.hex4()?;
                            let cp = if (0xd800..0xdc00).contains(&first)
                                && self.b.get(self.i) == Some(&b'\\')
                                && self.b.get(self.i + 1) == Some(&b'u')
                            {
                                self.i += 2;
                                let second = self.hex4()?;
                                if !(0xdc00..0xe000).contains(&second) {
                                    return Err(format!(
                                        "invalid low surrogate at offset {}",
                                        self.i
                                    ));
                                }
                                0x10000 + ((first - 0xd800) << 10) + (second - 0xdc00)
                            } else if (0xd800..0xe000).contains(&first) {
                                return Err(format!("lone surrogate at offset {}", self.i));
                            } else {
                                first
                            };
                            out.push(char::from_u32(cp).ok_or("invalid code point")?);
                            continue; // hex4 already advanced past its digits
                        }
                        other => {
                            return Err(format!("invalid escape {other:?} at offset {}", self.i))
                        }
                    }
                    self.i += 1;
                }
                Some(&c) if c < 0x20 => {
                    return Err(format!("raw control byte {:#04x} at offset {}", c, self.i))
                }
                Some(_) => {
                    // Copy one UTF-8 scalar; input is a &str so boundaries are valid.
                    let s = self.b[self.i..]
                        .iter()
                        .position(|b| *b == b'"' || *b == b'\\')
                        .expect("string is terminated");
                    let chunk_end = (self.i + s).min(self.b.len());
                    let chunk = std::str::from_utf8(&self.b[self.i..chunk_end])
                        .map_err(|_| "invalid UTF-8 in string")?;
                    out.push_str(chunk);
                    self.i = chunk_end;
                }
            }
        }
    }
    fn hex4(&mut self) -> Result<u32, String> {
        if self.i + 4 > self.b.len() {
            return Err("truncated \\u escape".into());
        }
        let hex = std::str::from_utf8(&self.b[self.i..self.i + 4]).expect("ASCII bytes");
        let v = u32::from_str_radix(hex, 16)
            .map_err(|_| format!("bad \\u escape at offset {}", self.i))?;
        self.i += 4;
        Ok(v)
    }
}

/// SHA-256 over bytes, lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Serializes under `fmt` and hashes the LF body, as the apparatus' recorders do.
pub fn digest_of(v: &J, fmt: Fmt) -> String {
    let mut body = String::new();
    write(v, fmt, &mut body);
    sha256_hex(body.as_bytes())
}
