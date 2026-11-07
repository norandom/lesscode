"""Probabilistic single-turn/two-tier regenerator controller (ProbabilisticRegenerator).

Language-agnostic by contract: ``TurnRequest.language`` drives prompt labels
and code fences, and every Python-specific analysis step (syntax, surface,
measures) is delegated behind the extractor seams in ``surface`` and
``measures``; this module never imports ``ast``.

Preserves the recorded single-turn contracts of the frozen ``regen_turn.py``
(deconvolution-loop task 4, R26 amendments 1-4): Contract-B surface input,
preregistered turn parameters, content hashes over exact UTF-8 LF bytes, and
case-distinct receipt labels. The receipt writer is internalized here (the
frozen ``receipt_atomic.py`` protocol) and additionally emits
``receipt.json.sha256`` plus ``bindings.json`` so ``lesscode-core admission``
verifies a run directly.
"""
from __future__ import annotations

import argparse
import dataclasses
import datetime
import decimal
import hashlib
import json
import os
import shutil
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Sequence
from urllib.parse import urlsplit

__all__ = [
    "EXIT_COPY_MISMATCH",
    "EXIT_FAILED_TURN",
    "EXIT_INPUT_TOO_LARGE",
    "EXIT_OK",
    "EXIT_OUT_OF_SCOPE",
    "EXIT_USAGE",
    "CopyMismatch",
    "DuplicateReceiptLabelError",
    "Endpoint",
    "FeedbackBoundaryError",
    "HoldoutOverlapError",
    "InputTooLarge",
    "ReceiptWriter",
    "ScopeError",
    "TurnRequest",
    "TurnResult",
    "build_prompt",
    "hash_set",
    "load_endpoint",
    "main",
    "parse_result",
    "prepare_candidate",
    "request_kwargs",
    "sha256_file",
    "sha256_text",
    "write_candidate",
]

# Exit codes preserved from the frozen regen_turn.py contract (see Error Handling).
EXIT_OK = 0
EXIT_USAGE = 2
EXIT_FAILED_TURN = 3
EXIT_COPY_MISMATCH = 4
EXIT_INPUT_TOO_LARGE = 5
EXIT_OUT_OF_SCOPE = 6

DEFAULT_ENV = Path.home() / ".lesscode" / "model.env"
ENV_KEYS = ("LESSCODE_MODEL_API_STYLE", "LESSCODE_MODEL_BASE_URL", "LESSCODE_MODEL_NAME", "LESSCODE_MODEL_API_KEY")
MAX_INPUT_TOKENS = 120_000
FEEDBACK_HEADING = "## Feedback from the previous round"
FEEDBACK_FIELDS = ("### First divergence", "### Measure gaps against the targets",
                   "### Surviving faults", "### Realised counterexamples")
FEEDBACK_KEYS = {"failed": ("first_divergence", "measure_gaps"),
                 "accepted_scored": ("first_divergence", "measure_gaps", "surviving_faults", "counterexamples")}
SEED_NOTE = "endpoint exposes no seed; determinism not claimed"
HELD_OUT_MARKERS = ("heldout", "held_out", "holdout")


class ScopeError(ValueError):
    """A returned path is outside the scope list."""


class CopyMismatch(RuntimeError):
    """The candidate copy's scope files differ from the frozen ones."""


class InputTooLarge(RuntimeError):
    """The counted input exceeds MAX_INPUT_TOKENS."""


class HoldoutOverlapError(ValueError):
    """Optimizer training scope intersects the held-out evaluation scope."""


class DuplicateReceiptLabelError(ValueError):
    """Case-insensitive collision in receipt labels (R26 amendment 4)."""


class FeedbackBoundaryError(ValueError):
    """Feedback payload carries held-out sequence names (Req 7.5 boundary)."""


def _obj(**props) -> dict:
    return {"type": "object", "additionalProperties": False, "required": list(props), "properties": props}


_STR, _STRS = {"type": "string"}, {"type": "array", "items": {"type": "string"}}
SCHEMA = _obj(files={"type": "array", "items": _obj(path=_STR, content=_STR)}, structure=_obj(pure=_STRS, shell=_STRS),
              change_log=_STR)

LANGUAGE_LABELS = {"python": "Python", "rust": "Rust", "typescript": "TypeScript"}


def _system_prompt(language: str) -> str:
    label = LANGUAGE_LABELS.get(language, language.capitalize())
    return f"""You are the regenerator in a preregistered software experiment. You rewrite a small set of {label}
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
   or not) as pure core or effect shell in "structure", using "<file>::{LANGUAGE_LABELS.get('python', 'python')}::<qualname>" entries.
5. Meet the measure targets in the user message.
6. If feedback from a previous round is present, treat it as the only information about that round.
Answer with exactly one JSON object and nothing else: {{"files": [{{"path": "<scope path>", "content":
"<full file text>"}}], "structure": {{"pure": [...], "shell": [...]}}, "change_log": "<what changed, per
file, and what is deliberately not preserved>"}}.
"""


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


@dataclasses.dataclass
class TurnRequest:
    """One request behind a fixed signature; ``language`` drives prompt labels and fences (Req 1.1)."""

    scope_root: str
    scope_files: list[str]
    surface_path: str
    definitions: str
    targets: dict
    contract: str
    acceptance_command: str
    feedback: dict | None = None
    language: str = "python"

    @classmethod
    def load(cls, path: Path) -> "TurnRequest":
        return cls(**json.loads(Path(path).read_text(encoding="utf-8")))

    def path(self, p: str) -> Path:
        return Path(p) if Path(p).is_absolute() else Path(self.scope_root) / p


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
        return {"style": self.style, "host": self.host, "model": self.model, "model_sha256": sha256_text(self.model)}

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
    """Fixed system brief plus a user message holding the inputs verbatim; fences follow req.language."""
    system = _system_prompt(req.language)
    root = Path(req.scope_root)
    parts = ["## Scope files (the only files that may change)"]
    for rel in req.scope_files:
        parts += [f"### {rel}", _fence((root / rel).read_text(encoding="utf-8"), req.language)]
    parts += ["## Derived public surface (public-surface.json)",
              _fence(req.path(req.surface_path).read_text(encoding="utf-8").strip(), "json"),
              "## Definitions", req.definitions, "## Measure targets", _fence(json.dumps(req.targets, indent=1), "json"),
              "## Structural contract", req.contract, "## Acceptance command (run by the loop, not by you)",
              _fence(req.acceptance_command, "")]
    if req.feedback is not None:
        parts.append(_feedback_section(req.feedback))
    return system, "\n\n".join(parts) + "\n"


def request_kwargs(ep: Endpoint, system: str, user: str, max_tokens: int, effort: str,
                   temperature: float | None = None, display: str | None = None) -> dict:
    kw = {"model": ep.model, "max_tokens": max_tokens, "system": system,
          "messages": [{"role": "user", "content": user}],
          "output_config": {"effort": effort, "format": {"type": "json_schema", "schema": SCHEMA}}}
    if temperature is not None:
        kw["temperature"] = temperature
    if display:
        kw["thinking"] = {"type": "adaptive", "display": display}
    return kw


def hash_set(kwargs: dict, response_json: str, model_returned: str) -> dict:
    """The receipt hash set (req 6.8); kwargs and response never contain the key."""
    return {"system_sha256": sha256_text(kwargs["system"]),
            "user_sha256": sha256_text(kwargs["messages"][0]["content"]),
            "request_sha256": sha256_text(json.dumps(kwargs, sort_keys=True, ensure_ascii=False)),
            "response_sha256": sha256_text(response_json),
            "model_sent": kwargs["model"], "model_returned": model_returned,
            "seed": kwargs.get("seed"), "seed_note": SEED_NOTE}


def _openai_call(ep: Endpoint, system: str, user: str, max_tokens: int, effort: str,
                 temperature: float | None = None) -> tuple[str, dict]:
    """One OpenAI-compatible chat completion via stdlib urllib (no SDK dependency)."""
    body = {"model": ep.model, "max_tokens": max_tokens, "temperature": 0 if temperature is None else temperature,
            "messages": [{"role": "system", "content": system}, {"role": "user", "content": user}],
            "response_format": {"type": "json_schema", "json_schema": {"name": "regenerate", "schema": SCHEMA}}}
    request = urllib.request.Request(
        ep.base_url.rstrip("/") + "/chat/completions",
        data=json.dumps(body).encode("utf-8"),
        headers={"Authorization": f"Bearer {ep.api_key()}", "Content-Type": "application/json"},
        method="POST",
    )
    t0 = time.perf_counter()
    try:
        with urllib.request.urlopen(request, timeout=3600) as resp:
            payload = json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        raise RuntimeError(f"openai transport: HTTP {e.code}") from e
    except (urllib.error.URLError, OSError) as e:
        raise RuntimeError(f"openai transport: {e}") from e
    choice = payload["choices"][0]
    text = choice["message"]["content"]
    meta = {"mode": "openai-json-http", "stop_reason": choice.get("finish_reason"),
            "elapsed_s": round(time.perf_counter() - t0, 2), "usage": payload.get("usage"),
            "max_tokens": max_tokens, "effort": effort, "temperature": body["temperature"]}
    return text, meta


def call(ep: Endpoint, system: str, user: str, max_tokens: int = 64000, effort: str = "high",
         temperature: float | None = None, display: str | None = None) -> tuple[str, dict]:
    """One request; returns (response_text, meta with the receipt hash set).

    The anthropic style streams through the lazily imported SDK; the openai
    style uses stdlib urllib. Raises InputTooLarge before any generation for
    the anthropic path (the openai-compatible arm has no token-counting call;
    the loop enforces the cap on the prompt text length).
    """
    if ep.style == "anthropic":
        import anthropic  # lazy: only the anthropic style pays for the SDK

        client = anthropic.Anthropic(api_key=ep.api_key(), base_url=ep.base_url,
                                     timeout=anthropic.Timeout(3600.0, connect=30.0))
        kwargs = request_kwargs(ep, system, user, max_tokens, effort, temperature, display)
        count = client.messages.count_tokens(
            model=kwargs["model"], system=system, messages=kwargs["messages"]
        ).input_tokens
        if count > MAX_INPUT_TOKENS:
            raise InputTooLarge(f"input {count} tokens exceeds {MAX_INPUT_TOKENS}", count)
        t0, prog = time.perf_counter(), {"events": 0, "last_event": None, "last_event_s": None}
        try:
            with client.messages.stream(**kwargs) as stream:
                for event in stream:
                    prog.update(events=prog["events"] + 1, last_event=event.type,
                                last_event_s=round(time.perf_counter() - t0, 2))
                msg = stream.get_final_message()
        except Exception as e:  # transport failure mid-stream; progress travels with the exception
            e.progress = prog
            raise
        text = next(b.text for b in msg.content if b.type == "text")
        meta = {"mode": "json_schema+stream", "anthropic_version": anthropic.__version__,
                "counted_input_tokens": count, "thinking_display": display, "stream": prog,
                "elapsed_s": round(time.perf_counter() - t0, 2), "stop_reason": msg.stop_reason,
                "stop_details": msg.stop_details.model_dump() if getattr(msg, "stop_details", None) else None,
                "usage": msg.usage.model_dump(), "max_tokens": max_tokens, "effort": effort,
                "temperature": temperature,
                **hash_set(kwargs, msg.to_json(), msg.model)}
        return text, meta
    text, meta = _openai_call(ep, system, user, max_tokens, effort, temperature)
    kwargs = request_kwargs(ep, system, user, max_tokens, effort, temperature, display)
    meta |= hash_set(kwargs, text, ep.model)
    return text, meta


def parse_result(text: str, meta: dict) -> TurnResult:
    data = json.loads(text)
    files, structure, log = data["files"], data["structure"], data["change_log"]
    if not (isinstance(files, list)
            and all(isinstance(f.get("path"), str) and isinstance(f.get("content"), str) for f in files)
            and isinstance(structure.get("pure"), list) and isinstance(structure.get("shell"), list)
            and isinstance(log, str)):
        raise ValueError("response JSON does not match the result shape")
    return TurnResult(files=files, structure=structure, change_log=log, meta=meta)


def prepare_candidate(frozen: Path, cand: Path, scope_files: list[str]) -> dict:
    """Fresh isolated copy of the frozen tree if absent; the scope files must still equal the frozen ones (exit 4)."""
    if not cand.exists():
        shutil.copytree(frozen, cand, symlinks=False,
                        ignore=shutil.ignore_patterns(".git", "__pycache__", ".pytest_cache", ".venv"))
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
        written.append({"path": Path(f["path"]).as_posix(), "bytes": len(data),
                        "sha256": hashlib.sha256(data).hexdigest(), "crlf": crlf})
    return written


def _save(dirpath: Path, name: str, text: str) -> str:
    """Write text as UTF-8 LF bytes with a sha256 sidecar; the digest is over the stored bytes (R26 amendment 3)."""
    dirpath.mkdir(parents=True, exist_ok=True)
    (dirpath / name).write_text(text, encoding="utf-8", newline="\n")
    (dirpath / f"{name}.sha256").write_text(f"{sha256_text(text)}  {name}\n", encoding="utf-8", newline="\n")
    return sha256_text(text)


class ReceiptWriter:
    """Internalized atomic receipt writer (.lock sidecar protocol) with admission bindings.

    Every update rewrites the receipt, refreshes ``receipt.json.sha256`` and
    ``bindings.json`` (``{"bindings": [{"artifact", "sidecars"}]}``) so that
    ``lesscode-core admission <dir> <dir>/bindings.json <out>`` verifies the
    receipt and every bound turn artifact byte-exactly. Labels are validated
    case-insensitively unique before execution (R26 amendment 4).
    """

    STALE_S = 120.0
    TIMEOUT_S = 300.0

    def __init__(self, receipt_path: Path):
        self.path = Path(receipt_path)
        self.base = self.path.parent
        self._labels: set[str] = set()
        if self.path.exists():
            for note in self._load().get("notes", []):
                self._labels.add(note["key"].lower())

    def _load(self) -> dict:
        if not self.path.exists():
            return {"facts": [], "notes": []}
        return json.loads(self.path.read_text(encoding="utf-8"))

    def _acquire(self, lock: Path) -> None:
        deadline = time.perf_counter() + self.TIMEOUT_S
        while True:
            try:
                lock.touch()
                return
            except FileExistsError:
                if time.time() - lock.stat().st_mtime > self.STALE_S:
                    lock.unlink(missing_ok=True)  # stale holder: take over
                    continue
                if time.perf_counter() > deadline:
                    raise TimeoutError(f"receipt lock held beyond {self.TIMEOUT_S}s: {lock}")
                time.sleep(0.05)

    def _write(self, data: dict, bound: list[dict]) -> None:
        self.path.write_text(json.dumps(data, indent=1), encoding="utf-8", newline="\n")
        self.base.mkdir(parents=True, exist_ok=True)
        digest = sha256_file(self.path)
        (self.base / "receipt.json.sha256").write_text(f"{digest}  receipt.json\n", encoding="utf-8", newline="\n")
        bindings = {"bindings": [{"artifact": "receipt.json", "sidecars": ["receipt.json.sha256"]}, *bound]}
        (self.base / "bindings.json").write_text(
            json.dumps(bindings, indent=1), encoding="utf-8", newline="\n"
        )

    def _append(self, collection: str, entry: dict, bound: list[dict] | None = None) -> None:
        self.base.mkdir(parents=True, exist_ok=True)
        lock = self.path.with_name(self.path.name + ".lock")
        self._acquire(lock)
        try:
            data = self._load()
            data.setdefault(collection, []).append(entry)
            self._write(data, bound or self._current_bindings())
        finally:
            lock.unlink(missing_ok=True)

    def _current_bindings(self) -> list[dict]:
        if not (self.base / "bindings.json").exists():
            return []
        return [
            b
            for b in json.loads((self.base / "bindings.json").read_text(encoding="utf-8"))["bindings"]
            if b["artifact"] != "receipt.json"
        ]

    def add_fact(self, fact: dict, bound: list[dict] | None = None) -> None:
        self._append("facts", {"time_utc": _now(), **fact}, bound)

    def add_note(self, key: str, value: dict, bound: list[dict] | None = None) -> None:
        if key.lower() in self._labels:
            raise DuplicateReceiptLabelError(f"receipt label collision (case-insensitive): {key!r}")
        self._labels.add(key.lower())
        self._append("notes", {"time_utc": _now(), "key": key, "value": value}, bound)

    def add_bound_artifact(self, artifact: Path) -> None:
        rel = artifact.resolve().relative_to(self.base.resolve()).as_posix()
        sidecars = [Path(rel).name + ".sha256"]
        bindings = [
            b
            for b in self._current_bindings()
            if b["artifact"] != rel
        ]
        bindings.append({"artifact": rel, "sidecars": sidecars})
        self._append = self._append  # noqa: PLW0127 - keep signature stable
        data = self._load()
        self._acquire(self.path.with_name(self.path.name + ".lock"))
        try:
            self._write(data, bindings)
        finally:
            lock = self.path.with_name(self.path.name + ".lock")
            lock.unlink(missing_ok=True)


def _now() -> str:
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def evaluate_tier2_runtime(
    exit_code: int,
    first_divergence: str | None,
    results: dict,
) -> RewardEvaluation:
    """Tier-2 RuntimeOracleGate over the recording arm's three-part outcome.

    ``exit_code`` is the arm verdict (0 accepted, 1 kill/divergence, 2 invalid
    arm, 124 timeout). The 0.60-0.85 partial-credit band applies the pinned
    formula ``R = 0.60 + 0.25 * (0.5*g + 0.5*s)`` (g = seed-group pass
    fraction, s = sequence pass fraction before first divergence), rounded
    half-up to two decimals; a missing fraction key counts as 0, never as a
    pass. Infrastructure outcomes (2/124/unknown) stay ``runtime_infrastructure``
    and are never classified as a kill or an equivalence pass (Req 5.4, 5.6).
    """
    if exit_code != 0 and exit_code != 1:
        return RewardEvaluation(0.50, 2, True, "runtime_infrastructure",
                                {"exit_code": exit_code, "first_divergence": first_divergence})
    if exit_code == 1:
        g = results.get("seed_group_pass_fraction", 0) or 0
        s = results.get("sequence_pass_fraction", 0) or 0
        gd = min(max(decimal.Decimal(str(g)), decimal.Decimal(0)), decimal.Decimal(1))
        sd = min(max(decimal.Decimal(str(s)), decimal.Decimal(0)), decimal.Decimal(1))
        step = decimal.Decimal("0.01")
        raw = decimal.Decimal("0.60") + decimal.Decimal("0.25") * (
            decimal.Decimal("0.5") * gd + decimal.Decimal("0.5") * sd
        )
        score = float(raw.quantize(step, rounding=decimal.ROUND_HALF_UP))
        return RewardEvaluation(score, 2, True, "kill",
                                {"first_divergence": first_divergence, "formula": {"g": g, "s": s}})
    if results.get("measure_gaps"):
        return RewardEvaluation(0.90, 2, True, "gaps", {"measure_gaps": results["measure_gaps"]})
    if results.get("surviving_faults") or results.get("counterexamples"):
        return RewardEvaluation(0.95, 2, True, "scored_faults",
                                {"surviving_faults": results.get("surviving_faults", []),
                                 "counterexamples": results.get("counterexamples", [])})
    return RewardEvaluation(1.00, 2, True, None, {})


def enforce_feedback_boundary(feedback: dict, held_out_names: Sequence[str]) -> None:
    """Feedback carries divergence identities and measure gaps only — never held-out sequence names.

    The whole payload is serialized and scanned for every held-out name before
    the controller may dispatch the next rollout (Req 7.5 separation; audited
    by ``ctrl_feedback_leak``).
    """
    blob = json.dumps(feedback, ensure_ascii=False)
    for name in held_out_names:
        if name and name in blob:
            raise FeedbackBoundaryError(
                f"feedback payload carries held-out sequence name {name!r}; refusing to dispatch"
            )


def verify_holdout_disjoint(train_scopes: set[str], eval_scopes: set[str]) -> None:
    """Training and held-out evaluation scopes must not intersect (Req 7.5)."""
    if overlap := sorted(train_scopes & eval_scopes):
        raise HoldoutOverlapError(f"optimizer training scope intersects held-out evaluation scope: {overlap}")


@dataclasses.dataclass
class RefineBudget:
    """Bounded turn/rollout/retry budget; every field is preregistered in the receipt."""

    max_turns: int = 4
    max_static_retries: int = 3
    n_rollouts_per_turn: int = 1
    max_input_tokens_per_call: int = MAX_INPUT_TOKENS
    max_total_tokens: int = 600_000
    no_progress_limit: int = 2
    threshold: float = 1.0


@dataclasses.dataclass
class RolloutRecord:
    """One recorded rollout (BestOfN element); hashes and diagnostics come from the turn."""

    label: str
    turn: int
    rollout_id: int
    attempt: int
    score: float
    tier: int
    failure_kind: str | None
    hashes: dict
    diagnostics: dict

    def as_dict(self) -> dict:
        return dataclasses.asdict(self)


@dataclasses.dataclass
class RefineOutcome:
    """Validity, compression and adoption stay separate; adoption is never claimed (Req 6.7)."""

    status: str  # "accepted" | "exhausted"
    best: RolloutRecord
    validity: bool
    compression_met: bool
    adopted: bool
    unresolved_obligations: list[str]
    rollout_history: list[dict]
    usage: dict


def run_refine_loop(
    req: TurnRequest,
    cand_root: Path,
    frozen_root: Path,
    receipt_path: Path,
    base_label: str,
    budget: RefineBudget,
    runtime_evaluator=None,
    call_fn=None,
    held_out_names: Sequence[str] = (),
    static_targets: dict | None = None,
) -> RefineOutcome:
    """Stdlib Refine/BestOfN controller: outer turns x BestOfN rollouts x inner static retries.

    The first attempt of the run resumes an existing candidate tree by hash
    identity (exit 4 on mismatch); every later attempt restores ``cand_root``
    from ``frozen_root`` before the next candidate write. Static failures
    never launch the runtime oracle; tier 2 runs only after a tier-1 pass.
    Turn feedback is boundary-checked against ``held_out_names`` before the
    next prompt is built. No-progress and token budgets stop on outer turns;
    exhaustion retains the best trajectory.
    """
    surface_report = json.loads(req.path(req.surface_path).read_text(encoding="utf-8"))
    writer = ReceiptWriter(receipt_path)
    call = call_fn or _default_call_fn
    history: list[dict] = []
    best: RolloutRecord | None = None
    usage_total = {"input_tokens": 0, "output_tokens": 0, "calls": 0}
    no_progress = 0
    resolved_obligations: set[str] = set()
    status = "exhausted"
    first_attempt = True
    base_params = {"max_tokens": 64000, "effort": "high", "temperature": 0.0, "display": None}

    for turn in range(1, budget.max_turns + 1):
        turn_best: RolloutRecord | None = None
        turn_feedback: dict | None = None
        for rollout in range(1, budget.n_rollouts_per_turn + 1):
            attempt_params = dict(base_params)
            record: RolloutRecord | None = None
            attempts: list[dict] = []
            label = f"{base_label}-t{turn}-r{rollout}"
            for attempt in range(1, budget.max_static_retries + 1):
                if first_attempt:
                    prepare_candidate(frozen_root, cand_root, req.scope_files)  # resume by hash identity
                    first_attempt = False
                else:
                    shutil.rmtree(cand_root, ignore_errors=True)
                    prepare_candidate(frozen_root, cand_root, req.scope_files)
                working_req = dataclasses.replace(req, feedback=turn_feedback)
                system, user = build_prompt(working_req)
                text, meta = None, {}
                try:
                    text, meta = call(system, user, attempt_params)
                    if meta.get("stop_reason") not in ("end_turn", "stop"):
                        outcome = {"ok": False, "kind": "stop_reason", "detail": meta.get("stop_reason")}
                    else:
                        try:
                            result = parse_result(text, meta)
                            written = write_candidate(result, cand_root, req.scope_files)
                        except ScopeError as e:
                            outcome = {"ok": False, "kind": "out_of_scope", "detail": str(e)}
                        except (ValueError, KeyError, TypeError, StopIteration) as e:
                            outcome = {"ok": False, "kind": "invalid_json", "detail": str(e)}
                        else:
                            outcome = {"ok": True, "result": result, "written": written}
                except Exception as e:  # transport failure before or during the call
                    outcome = {"ok": False, "kind": "transport", "detail": f"{type(e).__name__}: {e}"}
                usage = meta.get("usage") or {}
                usage_total["input_tokens"] += int(usage.get("input_tokens", 0) or 0)
                usage_total["output_tokens"] += int(usage.get("output_tokens", 0) or 0)
                usage_total["calls"] += 1
                reward = evaluate_tier1_static(req, outcome, cand_root, surface_report, static_targets)
                if reward.passed_tier1 and reward.score >= 0.50 and runtime_evaluator is not None:
                    exit_code, divergence, results = runtime_evaluator(cand_root, {})
                    reward = evaluate_tier2_runtime(exit_code, divergence, results)
                record = RolloutRecord(
                    label=label, turn=turn, rollout_id=rollout, attempt=attempt,
                    score=reward.score, tier=reward.tier, failure_kind=reward.failure_kind,
                    hashes={k: v for k, v in meta.items() if k.endswith("_sha256")},
                    diagnostics=reward.diagnostics,
                )
                attempts.append({
                    "attempt": attempt, "preregistered_params": attempt_params,
                    "reward": reward.as_receipt_value(), "diagnostics": reward.diagnostics,
                    **{k: meta[k] for k in ("stop_reason", "usage") if k in meta},
                })
                history.append(record.as_dict())
                if reward.score >= 0.50:
                    break  # the static repair chain ends on the first tier-1 pass
                turn_feedback = _static_feedback(reward)
                adaptation = next_rollout_params(reward, attempt - 1)
                if adaptation:
                    attempt_params.update({k: v for k, v in adaptation.items() if k in base_params})
                    if adaptation.get("inject_missing_symbols"):
                        turn_feedback["first_divergence"] += " missing: " + ", ".join(adaptation["inject_missing_symbols"])
            if attempts:
                writer.add_note(f"{label}-turn", {"attempts": attempts})
            if record is not None and (turn_best is None or record.score > turn_best.score):
                turn_best = record
        if turn_best is None:
            continue
        improved = best is None or turn_best.score > best.score
        discharged = _discharged_obligations(turn_best, resolved_obligations)
        no_progress = 0 if (improved or discharged) else no_progress + 1
        if improved:
            best = turn_best
        resolved_obligations |= discharged
        if best.score >= budget.threshold:
            status = "accepted"
            break
        if no_progress >= budget.no_progress_limit:
            break
        if usage_total["input_tokens"] + usage_total["output_tokens"] >= budget.max_total_tokens:
            break
        if runtime_evaluator is not None:
            turn_feedback = _turn_feedback_from(best)
            if turn_feedback is not None:
                enforce_feedback_boundary(turn_feedback, held_out_names)
    if best is None:
        raise RuntimeError("no rollout recorded; budget too small")
    unresolved = sorted(o for o in _known_obligations(best) if o not in resolved_obligations)
    return RefineOutcome(
        status=status,
        best=best,
        validity=best.score >= budget.threshold,
        compression_met=not static_targets,
        adopted=False,
        unresolved_obligations=unresolved,
        rollout_history=history,
        usage=usage_total,
    )


def _static_feedback(reward: RewardEvaluation) -> dict:
    parts = [f"static:{reward.failure_kind}"]
    detail = reward.diagnostics.get("detail")
    if detail:
        parts.append(str(detail))
    parts.extend((reward.diagnostics.get("errors") or [])[:3])
    return {"kind": "failed", "first_divergence": " ".join(parts)[:2000],
            "measure_gaps": reward.diagnostics.get("gaps") or {}}


def _turn_feedback_from(record: RolloutRecord) -> dict | None:
    if record.failure_kind is None and not record.diagnostics:
        return None
    divergence = record.diagnostics.get("first_divergence") or record.failure_kind or record.label
    return {"kind": "failed", "first_divergence": str(divergence)[:2000],
            "measure_gaps": record.diagnostics.get("measure_gaps") or {}}


def _known_obligations(record: RolloutRecord) -> set[str]:
    obligations: set[str] = set()
    if record.failure_kind:
        obligations.add(record.failure_kind)
    if record.diagnostics.get("first_divergence"):
        obligations.add(str(record.diagnostics["first_divergence"]))
    obligations.update(name for _, name in record.diagnostics.get("missing_symbols") or [])
    obligations.update((record.diagnostics.get("gaps") or {}).keys())
    return obligations


def _discharged_obligations(record: RolloutRecord, resolved: set[str]) -> set[str]:
    return {o for o in _known_obligations(record) - resolved if record.failure_kind != o}


def _default_call_fn(system: str, user: str, params: dict) -> tuple[str, dict]:
    ep = load_endpoint()
    return call(ep, system, user, max_tokens=params.get("max_tokens", 64000),
                effort=params.get("effort", "high"), temperature=params.get("temperature"),
                display=params.get("display"))


@dataclasses.dataclass
class RewardEvaluation:
    """Two-tier reward: tier 1 is static and offline, tier 2 wraps the runtime oracle."""

    score: float
    tier: int
    passed_tier1: bool
    failure_kind: str | None
    diagnostics: dict

    def as_receipt_value(self) -> dict:
        return {"score": self.score, "tier": self.tier, "passed_tier1": self.passed_tier1,
                "accepted": self.score >= 1.0, "failure_kind": self.failure_kind}


_TIER1_FAILURE_SCORES = {"transport": 0.00, "stop_reason": 0.00, "invalid_json": 0.10, "out_of_scope": 0.20}


def evaluate_tier1_static(
    req: TurnRequest,
    turn_outcome: dict,
    cand_root: Path,
    surface_report: dict,
    static_targets: dict | None = None,
) -> RewardEvaluation:
    """The Tier-1 static ladder: score rungs 0.00-0.50, offline, zero runtime calls.

    ``turn_outcome`` is ``{"ok": False, "kind": transport|stop_reason|invalid_json|out_of_scope, ...}``
    for a failed call/parse/write, or ``{"ok": True, "result": TurnResult, "written": [...]}`` after a
    successful candidate write. Syntax delegates to ``surface.check_syntax``, the surface rung to
    ``surface.verify_candidate_surface``, the structure rung to ``measures.measure_written`` with the
    candidate's declared structure, and the static-targets rung to ``measures.compute_measure_gaps``.
    """
    if not turn_outcome.get("ok"):
        kind = turn_outcome.get("kind", "transport")
        score = _TIER1_FAILURE_SCORES.get(kind, 0.00)
        return RewardEvaluation(score=score, tier=1, passed_tier1=False, failure_kind=kind,
                                diagnostics={"detail": turn_outcome.get("detail")})
    from . import surface  # language seam: all AST knowledge stays behind surface/measures

    errors = surface.check_syntax(cand_root, req.scope_files, language=req.language)
    if errors:
        return RewardEvaluation(0.30, 1, False, "syntax_error", {"errors": errors})
    verification = surface.verify_candidate_surface(cand_root, surface_report)
    if not verification["passed"]:
        return RewardEvaluation(0.40, 1, False, "surface", {k: verification[k] for k in
                                ("missing_symbols", "kind_mismatches", "local_rebindings")})
    declared = (turn_outcome.get("result").structure if turn_outcome.get("result") is not None else None)
    from . import measures

    written = measures.measure_written(cand_root, req.scope_files, structure=declared)
    if declared:
        structure_block = written.get("structure", {})
        if not structure_block.get("pass", True):
            why = {k: v["why"] for k, v in structure_block.get("pure", {}).items() if not v["pass"]}
            sites = [s for s in structure_block.get("write_sites", []) if not s["pass"]]
            return RewardEvaluation(0.45, 1, False, "structure",
                                    {"why": [w for ws in why.values() for w in ws], "write_sites": sites})
    gaps = measures.compute_measure_gaps(written, None, {"total": {}}, static_targets or {})
    if gaps:
        return RewardEvaluation(0.49, 1, False, "static_targets", {"gaps": gaps})
    return RewardEvaluation(0.50, 1, True, None, {})


_TEMPERATURE_STEPS = (0.0, 0.2, 0.4)


def next_rollout_params(prev: RewardEvaluation, attempt: int) -> dict:
    """Adaptive schedule for the next inner rollout; empty when nothing needs adapting."""
    if prev.passed_tier1 and prev.score >= 0.50:
        return {}
    params: dict = {}
    if prev.score == 0.00:  # transport or thinking-budget stop: cheaper effort, concise output directive
        params["effort"] = "medium"
        params["directive"] = (
            "Be concise: emit the JSON object directly, without extended internal monologue, within the budget."
        )
    if 0.10 <= prev.score <= 0.49:  # bounded temperature steps across static repair attempts
        params["temperature"] = _TEMPERATURE_STEPS[min(attempt, len(_TEMPERATURE_STEPS) - 1)]
    if prev.failure_kind == "surface" and prev.diagnostics.get("missing_symbols"):
        params["inject_missing_symbols"] = sorted({name for _, name in prev.diagnostics["missing_symbols"]})
    return params


def cmd_prompt(a: argparse.Namespace) -> int:
    system, user = build_prompt(TurnRequest.load(Path(a.request)))
    out = Path(a.out)
    hashes = {"system_sha256": _save(out, "system.txt", system), "user_sha256": _save(out, "user.txt", user),
              "system_chars": len(system), "user_chars": len(user)}
    (out / "prompt-hashes.json").write_text(json.dumps(hashes, indent=1), encoding="utf-8")
    print(json.dumps(hashes, indent=1))
    return EXIT_OK


def cmd_run(a: argparse.Namespace) -> int:
    receipt, label, cand = Path(a.receipt), a.label, Path(a.cand)
    writer = ReceiptWriter(receipt)
    req = TurnRequest.load(Path(a.request))
    turn_dir = Path(a.turns) / label
    system, user = build_prompt(req)
    prompts = {"system.txt": _save(turn_dir, "system.txt", system), "user.txt": _save(turn_dir, "user.txt", user)}
    ep = load_endpoint(Path(a.model_env)) if a.model_env else load_endpoint()
    writer.add_fact(
        {"key": f"{label}-inputs",
         "apparatus": {"regenerator.py": sha256_file(Path(__file__))},
         "request": {str(a.request): sha256_file(Path(a.request))},
         "surface": sha256_file(req.path(req.surface_path)),
         "frozen_scope": prepare_candidate(Path(a.frozen), cand, req.scope_files),
         "candidate_root": str(cand),
         "endpoint": ep.public(),
         "feedback_kind": (req.feedback or {}).get("kind"),
         "language": req.language,
         "preregistered_params": {"max_tokens": a.max_tokens, "effort": a.effort,
                                  "temperature": a.temperature, "rollout_id": a.rollout_id, "seed": None},
         "prompts": prompts},
        bound=[{"artifact": _rel_under(turn_dir, receipt, "system.txt"), "sidecars": ["system.txt.sha256"]},
               {"artifact": _rel_under(turn_dir, receipt, "user.txt"), "sidecars": ["user.txt.sha256"]}],
    )

    def failed(reason: str, code: int, **extra) -> int:  # failed turns stay in the receipt; no retry here
        writer.add_note(f"{label}-failed", {"reason": reason[:2000], "outcome": "failed", "exit": code, **extra})
        return code

    try:
        text, meta = call(ep, system, user, max_tokens=a.max_tokens, effort=a.effort,
                          temperature=a.temperature, display=a.thinking_display)
    except InputTooLarge as e:
        return failed(f"aborted before generation: {e.args[0]}", EXIT_INPUT_TOO_LARGE)
    except Exception as e:  # API or transport failure
        return failed(f"{type(e).__name__}: {e}", EXIT_FAILED_TURN,
                      stream=getattr(e, "progress", None), thinking_display=a.thinking_display)
    meta["response_sha256_saved"] = _save(turn_dir, "response.json", text)
    writer.add_note(f"{label}-turn", meta,
                    bound=[{"artifact": _rel_under(turn_dir, receipt, "response.json"),
                            "sidecars": ["response.json.sha256"]}])
    if meta["stop_reason"] not in ("end_turn", "stop"):
        return failed(f"stop_reason {meta['stop_reason']}", EXIT_FAILED_TURN)
    try:
        result = parse_result(text, meta)
        written = write_candidate(result, cand, req.scope_files)
    except (ValueError, KeyError, TypeError, AttributeError, StopIteration) as e:
        return failed(f"{type(e).__name__}: {e}", EXIT_OUT_OF_SCOPE if isinstance(e, ScopeError) else EXIT_FAILED_TURN)
    summary = {"files": written, "structure": result.structure, "change_log": result.change_log,
               "written_utc": _now()}
    digest = _save(turn_dir, "result.json", json.dumps(summary, indent=1, ensure_ascii=False))
    writer.add_note(f"{label}-candidate", {"root": str(cand), "files": written, "result_sha256": digest,
                                           "structure_counts": {k: len(v) for k, v in result.structure.items()}},
                    bound=[{"artifact": _rel_under(turn_dir, receipt, "result.json"),
                            "sidecars": ["result.json.sha256"]}])
    print(f"{label}: end_turn, {len(written)} files written to {cand}")
    return EXIT_OK


def _rel_under(turn_dir: Path, receipt: Path, name: str) -> str:
    return turn_dir.resolve().relative_to(receipt.parent.resolve()).as_posix() + "/" + name


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(
        prog="lesscode_baseline.regenerator",
        description="Single-turn/two-tier regenerator controller (Slice A; slice-B adoption gated by G1-R).",
    )
    sub = p.add_subparsers(dest="command", required=True)
    sp = sub.add_parser("prompt", help="build and hash the prompt for a request")
    sp.add_argument("--request", required=True)
    sp.add_argument("--out", required=True)
    sp.set_defaults(fn=cmd_prompt)
    sp = sub.add_parser("run", help="one recorded turn: prompt, call, write, receipt")
    sp.add_argument("--request", required=True)
    sp.add_argument("--cand", required=True)
    sp.add_argument("--receipt", required=True)
    sp.add_argument("--label", required=True)
    sp.add_argument("--frozen", required=True)
    sp.add_argument("--turns", required=True)
    sp.add_argument("--model-env", default=None)
    sp.add_argument("--max-tokens", type=int, default=64000)
    sp.add_argument("--effort", default="high")
    sp.add_argument("--temperature", type=float, default=None)
    sp.add_argument("--rollout-id", type=int, default=1)
    sp.add_argument("--thinking-display", default=None, choices=["summarized"])
    sp.set_defaults(fn=cmd_run)
    sp = sub.add_parser("refine", help="multi-turn Refine/BestOfN controller (task 5.4)")
    sp.set_defaults(fn=lambda a: (print("refine: implemented by task 5.4", file=sys.stderr), EXIT_USAGE)[1])
    a = p.parse_args(argv)
    try:
        return a.fn(a)
    except CopyMismatch as e:
        print(e, file=sys.stderr)
        return EXIT_COPY_MISMATCH
    except DuplicateReceiptLabelError as e:
        print(e, file=sys.stderr)
        return EXIT_USAGE
    except HoldoutOverlapError as e:
        print(e, file=sys.stderr)
        return EXIT_USAGE


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
