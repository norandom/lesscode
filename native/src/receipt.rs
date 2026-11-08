//! Order-preserving JSON receipts (design.md, Receipt Module): the shared
//! `json.dumps(indent=1)`-parity writer, moved from `admission.rs` task 2.1.

use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;

/// Receipt-domain JSON value type, extracted to `jsonval.rs` in the
/// task-9.3 defect-7 split (R37); re-exported so receipt-builder paths
/// (`receipt::J`) are unchanged.
pub use crate::jsonval::J;

const HEX: &[u8; 16] = b"0123456789abcdef";

/// Lowercase hex SHA-256 over the bytes as stored.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let mut out = String::with_capacity(64);
    for byte in hasher.finalize() {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn indent(depth: usize, out: &mut String) {
    for _ in 0..depth {
        out.push(' ');
    }
}

/// Python `json.dumps` string escaping (`ensure_ascii`): short escapes for
/// `"`, `\`, \b, \t, \n, \f, \r; `\u00xx` (lowercase hex) for other control
/// characters and DEL; `\uxxxx` (surrogate pairs above BMP) for non-ASCII.
pub(crate) fn write_json_str(s: &str, out: &mut String) {
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

/// Python `json.dumps(value, indent=1)`-compatible emission for the receipt
/// domain: one space per nesting level, `": "` after keys, `",\n"` between
/// items, empty containers inline, insertion order preserved.
fn write_value(value: &J, depth: usize, out: &mut String) {
    match value {
        J::Null => out.push_str("null"),
        J::Bool(true) => out.push_str("true"),
        J::Bool(false) => out.push_str("false"),
        J::Int(n) => out.push_str(&n.to_string()),
        // Python `json.dumps` float repr: shortest round-trip, integral
        // values keep one decimal ("1.0"); shares are rounded ratios only.
        J::Float(x) if x.fract() == 0.0 && x.is_finite() => {
            let _ = write!(out, "{x:.1}");
        }
        J::Float(x) if x.is_finite() => {
            let _ = write!(out, "{x}");
        }
        J::Float(_) => out.push_str("null"),
        J::Str(s) => write_json_str(s, out),
        J::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                indent(depth + 1, out);
                write_value(item, depth + 1, out);
            }
            out.push('\n');
            indent(depth, out);
            out.push(']');
        }
        J::Obj(pairs) => {
            if pairs.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            for (i, (key, val)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                indent(depth + 1, out);
                write_json_str(key, out);
                out.push_str(": ");
                write_value(val, depth + 1, out);
            }
            out.push('\n');
            indent(depth, out);
            out.push('}');
        }
    }
}

/// Writes the receipt with LF endings plus its `.sha256` sidecar (sha256sum
/// format, digest over the bytes as stored).
/// Render a value with the `json.dumps(value, indent=1)`-compatible writer
/// (stdout use; receipts use `write_receipt`).
pub fn dumps(value: &J) -> String {
    let mut out = String::new();
    write_value(value, 0, &mut out);
    out
}

pub fn write_receipt(path: &Path, value: &J) -> io::Result<()> {
    let mut body = String::new();
    write_value(value, 0, &mut body);
    body.push('\n');
    fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))?;
    fs::write(path, body.as_bytes())?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let sidecar = format!("{}  {}\n", sha256_hex(body.as_bytes()), name);
    fs::write(path.with_extension("json.sha256"), sidecar.as_bytes())?;
    Ok(())
}
