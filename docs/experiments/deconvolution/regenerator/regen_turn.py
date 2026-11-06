"""Regenerator as a recorded single-turn program (deconvolution-loop task 4; req 6.1, 6.7, 6.8, 11.2).

One turn = one request with a fixed signature (TurnRequest -> TurnResult). The endpoint comes from the file
named by LESSCODE_MODEL_ENV (default ~/.lesscode/model.env); the key is read here only, never printed or
stored. Prompt, response, model identifier and seed go to the receipt by content hash; texts to results/turns/.
  regen_turn.py prompt --request R.json --out DIR
  regen_turn.py run --request R.json --cand ROOT --receipt RECEIPT --label L [--frozen F] [--max-tokens N]
Exit: 0 ok; 2 usage; 3 failed turn (API error, stop_reason, invalid JSON); 4 candidate copy differs from the
frozen scope files; 5 input over the token cap; 6 a returned path is outside the scope.
"""
from __future__ import annotations

import argparse
import dataclasses
import datetime
import hashlib
import json
import os
import shutil
import sys
import time
from pathlib import Path
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
LESSCODE = HERE.parents[3]
EXTRAS = LESSCODE / ".runs" / "regen-extras"
FROZEN = LESSCODE / ".runs" / "e2-facdrone" / "source"
DEFAULT_ENV = Path.home() / ".lesscode" / "model.env"
ENV_KEYS = ("LESSCODE_MODEL_API_STYLE", "LESSCODE_MODEL_BASE_URL", "LESSCODE_MODEL_NAME", "LESSCODE_MODEL_API_KEY")
MAX_INPUT_TOKENS = 120_000
FEEDBACK_HEADING = "## Feedback from the previous round"
FEEDBACK_FIELDS = ("### First divergence", "### Measure gaps against the targets",
                   "### Surviving faults", "### Realised counterexamples")
FEEDBACK_KEYS = {"failed": ("first_divergence", "measure_gaps"),
                 "accepted_scored": ("first_divergence", "measure_gaps", "surviving_faults", "counterexamples")}
SEED_NOTE = "endpoint exposes no seed; determinism not claimed"


def _obj(**props) -> dict:
    return {"type": "object", "additionalProperties": False, "required": list(props), "properties": props}


_STR, _STRS = {"type": "string"}, {"type": "array", "items": {"type": "string"}}
SCHEMA = _obj(files={"type": "array", "items": _obj(path=_STR, content=_STR)}, structure=_obj(pure=_STRS, shell=_STRS),
              change_log=_STR)
SYSTEM = """You are the regenerator in a preregistered software experiment. You rewrite a small set of Python
files behind their public API so that observable behaviour is unchanged and the code is smaller and less
entangled. An acceptance arm (recorded traces, a codec harness and the original test suite) is the only
authority on behaviour; you cannot run it, so reason from the sources.

Rules, all binding:
1. Only the scope files listed in the user message may change. Return every scope file you change with its
   complete new content; omit files you leave unchanged. Never return any other path.
2. Preserve every name in the derived public surface with the same kind (function, class, constant, module)
   and the same signature and observable behaviour, including private names that tests import. Names that
   tests patch or override (kind "patch") must stay module globals that the module's own functions look up
   by name at call time.
3. Keep SQL statement text, parameters, check order and error messages unchanged unless the surface shows
   nothing observes them; the acceptance arm compares statement text and parameters.
4. Obey the structural contract in the user message: declare every function of every scope file (changed
   or not) as pure core or effect shell in "structure", using "<file>.py::<qualname>" entries.
5. Meet the measure targets in the user message.
6. If feedback from a previous round is present, treat it as the only information about that round.
Answer with exactly one JSON object and nothing else: {"files": [{"path": "<scope path>", "content":
"<full file text>"}], "structure": {"pure": [...], "shell": [...]}, "change_log": "<what changed, per
file, and what is deliberately not preserved>"}.
"""


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class ScopeError(ValueError): "A returned path is outside the scope list."
class CopyMismatch(RuntimeError): "The candidate copy's scope files differ from the frozen ones."
class InputTooLarge(RuntimeError): "The counted input exceeds MAX_INPUT_TOKENS."


@dataclasses.dataclass
class TurnRequest:
    scope_root: str
    scope_files: list[str]
    surface_path: str
    definitions: str
    targets: dict
    contract: str
    acceptance_command: str
    feedback: dict | None = None

    @classmethod
    def load(cls, path: Path) -> "TurnRequest":
        return cls(**json.loads(Path(path).read_text(encoding="utf-8")))

    def path(self, p: str) -> Path:
        return Path(p) if Path(p).is_absolute() else LESSCODE / p


@dataclasses.dataclass
class TurnResult:
    files: list[dict]
    structure: dict
    change_log: str
    meta: dict


class Endpoint:
    """Endpoint configuration; the key is reachable only through api_key()."""

    def __init__(self, style: str, base_url: str, model: str, key: str):
        self.style, self.base_url, self.model = style, base_url, model
        self.host = urlsplit(base_url).hostname or base_url
        self.api_key = lambda: key

    def public(self) -> dict:
        return {"style": self.style, "host": self.host, "model_sha256": sha256_text(self.model)}

    def __repr__(self) -> str:
        return f"Endpoint(style={self.style!r}, host={self.host!r}, model={self.model!r}, key=<redacted>)"

    __str__ = __repr__


def load_endpoint(path: Path | None = None) -> Endpoint:
    path = Path(path or os.environ.get("LESSCODE_MODEL_ENV") or DEFAULT_ENV)
    vals: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        body = line.split("#", 1)[0].strip()
        if body and "=" in body:
            k, _, v = body.partition("=")
            vals[k.strip()] = v.strip()
    if missing := [k for k in ENV_KEYS if not vals.get(k)]:
        raise ValueError(f"{path.name}: missing {missing}")
    style = vals[ENV_KEYS[0]].lower()
    if style not in ("anthropic", "openai"):
        raise ValueError(f"unknown LESSCODE_MODEL_API_STYLE {style!r}")
    return Endpoint(style, vals[ENV_KEYS[1]], vals[ENV_KEYS[2]], vals[ENV_KEYS[3]])


def _fence(text: str, lang: str) -> str:
    ticks = "```"
    while ticks in text:
        ticks += "`"
    return f"{ticks}{lang}\n{text}\n{ticks}"


def _feedback_section(fb: dict) -> str:
    keys = FEEDBACK_KEYS.get(fb.get("kind"))
    if keys is None or set(fb) - {"kind", *keys}:
        raise ValueError(f"feedback must be one of {list(FEEDBACK_KEYS)} with exactly its fields: {sorted(fb)}")
    out = [FEEDBACK_HEADING, f"Previous round outcome: {fb['kind']}."]
    for heading, key in zip(FEEDBACK_FIELDS, keys):
        v = fb.get(key)
        out += [heading, v if isinstance(v, str) else ("none" if v is None else json.dumps(v, indent=1, ensure_ascii=False))]
    return "\n".join(out)


def build_prompt(req: TurnRequest) -> tuple[str, str]:
    """Fixed system brief plus a user message holding the inputs verbatim (req 6.1, 6.7)."""
    root = req.path(req.scope_root)
    parts = ["## Scope files (the only files that may change)"]
    for rel in req.scope_files:
        parts += [f"### {rel}", _fence((root / rel).read_text(encoding="utf-8"), "python")]
    parts += ["## Derived public surface (public-surface.json)",
              _fence(req.path(req.surface_path).read_text(encoding="utf-8").strip(), "json"),
              "## Definitions", req.definitions, "## Measure targets", _fence(json.dumps(req.targets, indent=1), "json"),
              "## Structural contract", req.contract, "## Acceptance command (run by the loop, not by you)",
              _fence(req.acceptance_command, "")]
    if req.feedback is not None:
        parts.append(_feedback_section(req.feedback))
    return SYSTEM, "\n\n".join(parts) + "\n"


def request_kwargs(ep: Endpoint, system: str, user: str, max_tokens: int, effort: str, display: str | None = None) -> dict:
    kw = {"model": ep.model, "max_tokens": max_tokens, "system": system, "messages": [{"role": "user", "content": user}],
          "output_config": {"effort": effort, "format": {"type": "json_schema", "schema": SCHEMA}}}
    if display:  # adaptive is what omitting `thinking` means on this model; display only makes the stream carry it
        kw["thinking"] = {"type": "adaptive", "display": display}
    return kw


def hash_set(kwargs: dict, msg) -> dict:
    """The receipt hash set of req 6.8; kwargs never contain the key."""
    return {"system_sha256": sha256_text(kwargs["system"]), "user_sha256": sha256_text(kwargs["messages"][0]["content"]),
            "request_sha256": sha256_text(json.dumps(kwargs, sort_keys=True, ensure_ascii=False)),
            "response_sha256": sha256_text(msg.to_json()), "model_sent": kwargs["model"], "model_returned": msg.model,
            "message_id": msg.id, "request_id": getattr(msg, "_request_id", None), "seed": None, "seed_note": SEED_NOTE}


def call(ep: Endpoint, system: str, user: str, max_tokens: int = 64000, effort: str = "high",
         display: str | None = None) -> tuple[object, dict]:
    """One streamed request; returns (message, meta). Raises InputTooLarge before any generation; a transport
    failure mid-stream is re-raised with e.progress (events seen, last event type and time)."""
    if ep.style != "anthropic":
        raise NotImplementedError(f"API style {ep.style!r} not implemented")
    sys.path.append(str(EXTRAS))
    import anthropic
    client = anthropic.Anthropic(api_key=ep.api_key(), base_url=ep.base_url, timeout=anthropic.Timeout(3600.0, connect=30.0))
    kwargs = request_kwargs(ep, system, user, max_tokens, effort, display)
    count = client.messages.count_tokens(model=kwargs["model"], system=system, messages=kwargs["messages"]).input_tokens
    if count > MAX_INPUT_TOKENS:
        raise InputTooLarge(f"input {count} tokens exceeds {MAX_INPUT_TOKENS}", count)
    t0, prog = time.perf_counter(), {"events": 0, "last_event": None, "last_event_s": None}
    try:
        with client.messages.stream(**kwargs) as stream:
            for event in stream:
                prog.update(events=prog["events"] + 1, last_event=event.type, last_event_s=round(time.perf_counter() - t0, 2))
            msg = stream.get_final_message()
    except Exception as e:
        e.progress = prog
        raise
    meta = {"mode": "json_schema+stream", "anthropic_version": anthropic.__version__, "counted_input_tokens": count,
            "thinking_display": display, "stream": prog, "elapsed_s": round(time.perf_counter() - t0, 2), "stop_reason": msg.stop_reason,
            "stop_details": msg.stop_details.model_dump() if getattr(msg, "stop_details", None) else None,
            "usage": msg.usage.model_dump(), "max_tokens": max_tokens, "effort": effort, **hash_set(kwargs, msg)}
    return msg, meta


def parse_result(msg, meta: dict) -> TurnResult:
    data = json.loads(next(b.text for b in msg.content if b.type == "text"))
    files, structure, log = data["files"], data["structure"], data["change_log"]
    if not (isinstance(files, list) and all(isinstance(f.get("path"), str) and isinstance(f.get("content"), str) for f in files)
            and isinstance(structure.get("pure"), list) and isinstance(structure.get("shell"), list) and isinstance(log, str)):
        raise ValueError("response JSON does not match the result shape")
    return TurnResult(files=files, structure=structure, change_log=log, meta=meta)


def prepare_candidate(frozen: Path, cand: Path, scope_files: list[str]) -> dict:
    """Fresh isolated copy of the frozen tree if absent; the scope files must still equal the frozen ones (exit 4)."""
    if not cand.exists():
        shutil.copytree(frozen, cand, symlinks=False, ignore=shutil.ignore_patterns(".git", "__pycache__", ".pytest_cache", ".venv"))
    hashes = {rel: sha256_file(frozen / rel) for rel in scope_files}
    if bad := [rel for rel in scope_files if sha256_file(cand / rel) != hashes[rel]]:
        raise CopyMismatch(f"candidate scope files differ from frozen before writing: {bad}")
    return hashes


def write_candidate(result: TurnResult, cand: Path, scope_files: list[str]) -> list[dict]:
    """Write returned files into cand; refuse any path outside the scope before writing anything (exit 6)."""
    allowed = {Path(s).as_posix() for s in scope_files}
    if outside := [f["path"] for f in result.files if Path(f["path"]).as_posix() not in allowed]:
        raise ScopeError(f"paths outside scope: {outside}")
    written = []
    for f in result.files:
        target = cand / Path(f["path"]).as_posix()
        crlf = target.exists() and b"\r\n" in target.read_bytes()  # keep the replaced file's line-ending convention
        content = f["content"].replace("\r\n", "\n")
        data = (content.replace("\n", "\r\n") if crlf else content).encode("utf-8")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        written.append({"path": Path(f["path"]).as_posix(), "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "crlf": crlf})
    return written


def _save(dirpath: Path, name: str, text: str) -> str:
    dirpath.mkdir(parents=True, exist_ok=True)
    (dirpath / name).write_text(text, encoding="utf-8", newline="\n")  # bytes on disk == hashed text (2026-11-06 review)
    (dirpath / f"{name}.sha256").write_text(f"{sha256_text(text)}  {name}\n", encoding="utf-8", newline="\n")
    return sha256_text(text)


def cmd_prompt(a) -> int:
    system, user = build_prompt(TurnRequest.load(a.request))
    out = Path(a.out)
    hashes = {"system_sha256": _save(out, "system.txt", system), "user_sha256": _save(out, "user.txt", user),
              "system_chars": len(system), "user_chars": len(user)}
    (out / "prompt-hashes.json").write_text(json.dumps(hashes, indent=1), encoding="utf-8")
    print(json.dumps(hashes, indent=1))
    return 0


def cmd_run(a) -> int:
    sys.path.insert(0, str(HERE.parent))
    import receipt_atomic as ra
    receipt, label, cand = Path(a.receipt), a.label, Path(a.cand)
    req = TurnRequest.load(a.request)
    turn_dir = HERE / "results" / "turns" / label
    system, user = build_prompt(req)
    prompts = {"system.txt": _save(turn_dir, "system.txt", system), "user.txt": _save(turn_dir, "user.txt", user)}
    ep = load_endpoint()
    ra.add_fact(receipt, {"key": f"{label}-inputs", "apparatus": {"regen_turn.py": sha256_file(Path(__file__))},
                          "request": {str(a.request): sha256_file(Path(a.request))}, "surface": sha256_file(req.path(req.surface_path)),
                          "frozen_scope": prepare_candidate(Path(a.frozen), cand, req.scope_files), "candidate_root": str(cand),
                          "endpoint": ep.public(), "feedback_kind": (req.feedback or {}).get("kind"), "prompts": prompts})
    def failed(reason: str, code: int, **extra) -> int:  # failed turns stay in the receipt (req 11.3); no retry here
        ra.add_note(receipt, f"{label}-failed", {"reason": reason[:2000], "outcome": "failed", "exit": code, **extra})
        return code
    try:
        msg, meta = call(ep, system, user, max_tokens=a.max_tokens, effort=a.effort, display=a.thinking_display)
    except InputTooLarge as e:
        return failed(f"aborted before generation: {e.args[0]}", 5)
    except Exception as e:  # API or transport failure
        return failed(f"{type(e).__name__}: {e}", 3, stream=getattr(e, "progress", None), thinking_display=a.thinking_display)
    meta["response_sha256_saved"] = _save(turn_dir, "response.json", msg.to_json())
    ra.add_note(receipt, f"{label}-turn", meta)
    if msg.stop_reason != "end_turn":
        return failed(f"stop_reason {msg.stop_reason}", 3)
    try:
        result = parse_result(msg, meta)
        written = write_candidate(result, cand, req.scope_files)
    except (ValueError, KeyError, TypeError, AttributeError, StopIteration) as e:
        return failed(f"{type(e).__name__}: {e}", 6 if isinstance(e, ScopeError) else 3)
    summary = {"files": written, "structure": result.structure, "change_log": result.change_log,
               "written_utc": datetime.datetime.now(datetime.UTC).isoformat()}
    digest = _save(turn_dir, "result.json", json.dumps(summary, indent=1, ensure_ascii=False))
    ra.add_note(receipt, f"{label}-candidate", {"root": str(cand), "files": written, "result_sha256": digest,
                                                "structure_counts": {k: len(v) for k, v in result.structure.items()}})
    print(f"{label}: end_turn, {len(written)} files written to {cand}")
    return 0


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="cmd", required=True)
    args = {"prompt": (cmd_prompt, {"--request": {}, "--out": {}}),
            "run": (cmd_run, {"--request": {}, "--cand": {}, "--receipt": {}, "--label": {}, "--frozen": {"default": str(FROZEN)},
                              "--max-tokens": {"type": int, "default": 64000}, "--effort": {"default": "high"},
                              "--thinking-display": {"default": None, "choices": ["summarized"]}})}
    for name, (fn, opts) in args.items():
        sp = sub.add_parser(name)
        sp.set_defaults(fn=fn)
        for opt, kw in opts.items():
            sp.add_argument(opt, required="default" not in kw, **kw)
    a = p.parse_args(argv)
    try:
        return a.fn(a)
    except CopyMismatch as e:
        print(e, file=sys.stderr)
        return 4


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
