# S2(b): generate Python-side parity fixtures for the Rust pyjson formatter.
# Uses exactly the apparatus formatting modes (e1b_oracle.py L48/L385, e4_oracle.py L98/L508,
# receipt_atomic.py L62). Expected digests are sha256 over the emitted LF body.
import hashlib
import json
from pathlib import Path

OUT = Path(__file__).resolve().parents[4] / "native" / "tests" / "fixtures" / "pyjson"

# Edge values: negative zero, exponent boundaries of Python repr, NaN/Infinity,
# arbitrary-precision ints, non-ASCII + astral + control chars, nesting, empties.
VALUE = {
    "b": True, "n": None, "f": False,
    "i": 12345678901234567890123456789,
    "neg0": -0.0, "zero": 0.0, "one": 1.0, "hundred": 100.0,
    "tiny": 1e-07, "tiny_edge": 1e-4, "big_edge": 1e16, "big": 1.5e23,
    "nan": float("nan"), "inf": float("inf"), "ninf": float("-inf"),
    "s_ascii": "plain", "s_latin": "caf\xe9", "s_astral": "\U0001F600\U0001D11E",
    "s_ctl": "a\x01b\x1fc", "s_quote": "q\"q\\q\n\t\r\b\fd",
    "empty_obj": {}, "empty_arr": [], "arr": [1, [2, [3, {"k": "v"}]]],
    "obj": {"z": 1, "a": {"y": [1.5, -0.0], "m": {}}, "\xe9": 2},
}

GOLDEN_LIKE = {"meta": {"alphabet": ["op_a", "op_b"], "counts": {"n": 3, "-0.0": -0.0}},
               "sequences": [{"name": "s1", "steps": [{"op": "op_a", "v": 1e-07}]}]}

CASES = {
    # mode -> (python emission kwargs, value)
    "canon_compact": (dict(sort_keys=True, separators=(",", ":")), VALUE),
    "default_compact": (dict(sort_keys=True), VALUE),
    "golden_indent0": (dict(sort_keys=True, indent=0, allow_nan=True), GOLDEN_LIKE),
    "receipt_indent1": (dict(indent=1), GOLDEN_LIKE),
}

def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    manifest = []
    for name, (kwargs, value) in CASES.items():
        body = json.dumps(value, **kwargs)
        path = OUT / f"{name}.json"
        path.write_text(body, encoding="utf-8", newline="\n")
        digest = hashlib.sha256(body.encode()).hexdigest()
        manifest.append({"case": name, "kwargs": {k: str(v) for k, v in kwargs.items()},
                         "file": path.name, "sha256_lf_body": digest, "bytes": len(body.encode())})
    (OUT / "expected.json").write_text(
        json.dumps({"generator": "s2_make_fixtures.py", "python_format": "json.dumps", "cases": manifest},
                   indent=1) + "\n", encoding="utf-8", newline="\n")
    for m in manifest:
        print(f"{m['case']}: {m['sha256_lf_body']}  {m['bytes']} bytes")

if __name__ == "__main__":
    main()
