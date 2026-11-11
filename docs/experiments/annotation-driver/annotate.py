r"""Send one scope's source to a model endpoint under the annotation guide.

    uv run --no-sync python annotate.py --scope NAME --scopes-dir DIR --guide FILE --out-dir DIR
        [--max-output-tokens N] [--timeout-s N] [--dry-run] [--endpoint-env FILE]

The endpoint comes from a KEY=VALUE file: ``--endpoint-env``, else the
environment variable LESSCODE_MODEL_ENV, else %USERPROFILE%\.lesscode\model.env.
Writes request.json, response.json, reply.txt, annotations.json and call.json
into the output directory and appends one line per real call to ledger.jsonl
in its parent. The API key is never printed or written.

Exit codes: 0 call made and parsed; 2 parse_failed or reply cut off;
3 configuration error; 4 request failed after the allowed attempts.
"""

from __future__ import annotations

import argparse
import hashlib
import http.client
import json
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

import validator as v

HERE = Path(__file__).parent
STYLE, BASE, NAME, KEY = "LESSCODE_MODEL_API_STYLE", "LESSCODE_MODEL_BASE_URL", "LESSCODE_MODEL_NAME", "LESSCODE_MODEL_API_KEY"
CUT_OFF = {"anthropic": "max_tokens", "openai": "length"}
SECRET_HEADERS = ("x-api-key", "authorization")
REDACTED = "<redacted>"
RETRY_WAIT_S = 10
USER_TEMPLATE = """The file below is shown with line numbers. The numbers and the "| " separator are not part of the file; anchors must be copied from the text after the separator.

File: {path}
{numbered}"""
FENCE = re.compile(r"```(?:json)?\n(.*)\n```", re.DOTALL)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def fail(message: str) -> int:
    print(f"annotate: {message}", file=sys.stderr)
    return 3


def env_path(arg: Path | None, environ) -> Path:
    if arg is not None:
        return arg
    if environ.get("LESSCODE_MODEL_ENV"):
        return Path(environ["LESSCODE_MODEL_ENV"])
    return Path(environ.get("USERPROFILE") or Path.home()) / ".lesscode" / "model.env"


def load_config(path: Path) -> tuple[dict | None, str | None]:
    """The endpoint settings, or an error message that names a key but never a value."""
    if not path.is_file():
        return None, f"model env file not found: {path}"
    cfg = {}
    for line in path.read_text(encoding="utf-8-sig").splitlines():
        line = line.split("#", 1)[0].strip()
        if "=" in line:
            key, value = line.split("=", 1)
            cfg[key.strip()] = value.strip()
    for key in (STYLE, BASE, NAME, KEY):
        if not cfg.get(key):
            return None, f"missing {key} in {path}"
    if cfg[STYLE] not in CUT_OFF:
        return None, f"{STYLE} must be one of: {', '.join(CUT_OFF)}"
    return cfg, None


def numbered(text: str) -> str:
    lines = text.split("\n")
    if lines[-1] == "":
        lines.pop()
    return "\n".join(f"{n:>5}| {line}" for n, line in enumerate(lines, 1))


def user_message(scope: v.Scope) -> str:
    intro, block = USER_TEMPLATE.split("\n\n", 1)
    blocks = [block.format(path=f, numbered=numbered(scope.source(f))) for f in scope.files]
    return intro + "\n\n" + "\n\n".join(blocks)


def build_request(cfg: dict, guide: str, user: str, max_tokens: int,
                  history: list[dict] | tuple = ()) -> tuple[str, dict, dict]:
    """URL, headers and body; ``history`` holds earlier turns ({"role", "content"}) placed before ``user``."""
    base, model, key = cfg[BASE].removesuffix("/"), cfg[NAME], cfg[KEY]
    turns = [*history, {"role": "user", "content": user}]
    if cfg[STYLE] == "anthropic":
        url = base + ("/messages" if base.endswith("/v1") else "/v1/messages")
        headers = {"x-api-key": key, "anthropic-version": "2023-06-01", "content-type": "application/json"}
        body = {"model": model, "max_tokens": max_tokens, "system": guide, "messages": turns}
    else:
        url = base + "/chat/completions"
        headers = {"Authorization": f"Bearer {key}", "content-type": "application/json"}
        body = {"model": model, "max_tokens": max_tokens,
                "messages": [{"role": "system", "content": guide}, *turns]}
    return url, headers, body


def open_url(request: urllib.request.Request, timeout: float):
    """The only place that opens a network connection."""
    return urllib.request.urlopen(request, timeout=timeout)


def send(url: str, headers: dict, body: dict, timeout: float, opener=open_url, sleep=time.sleep) -> list[dict]:
    """At most two attempts; the second only after 429, 5xx or a timeout or connection error."""
    data = json.dumps(body).encode("utf-8")
    attempts = []
    for n in (1, 2):
        status, text, error, retry = None, None, None, False
        started = time.monotonic()
        try:
            with opener(urllib.request.Request(url, data=data, headers=headers, method="POST"), timeout) as resp:
                status, text = resp.status, resp.read().decode("utf-8", "replace")
        except urllib.error.HTTPError as exc:
            status, text, error = exc.code, exc.read().decode("utf-8", "replace"), str(exc)
            retry = exc.code == 429 or 500 <= exc.code < 600
        except (OSError, http.client.HTTPException) as exc:
            error, retry = f"{type(exc).__name__}: {exc}", True
        attempts.append({"attempt": n, "status": status, "body": text, "error": error,
                         "seconds": round(time.monotonic() - started, 3)})
        if not retry or n == 2:
            break
        sleep(RETRY_WAIT_S)
    return attempts


def extract(style: str, raw: str) -> dict:
    data = json.loads(raw)
    usage = data.get("usage") or {}
    if style == "anthropic":
        text = "".join(b["text"] for b in data["content"] if b.get("type") == "text")
        return {"text": text, "input_tokens": usage.get("input_tokens"), "output_tokens": usage.get("output_tokens"),
                "stop_reason": data.get("stop_reason"), "returned_model": data.get("model")}
    choice = data["choices"][0]
    content = choice["message"]["content"] or ""
    if isinstance(content, list):  # reasoning and text parts (Mistral): the reply is the text parts
        content = "".join(p.get("text", "") for p in content if isinstance(p, dict) and p.get("type") == "text")
    return {"text": content, "input_tokens": usage.get("prompt_tokens"),
            "output_tokens": usage.get("completion_tokens"), "stop_reason": choice.get("finish_reason"),
            "returned_model": data.get("model")}


def _reject_constant(name: str):
    raise ValueError(f"non-standard JSON constant {name}")


def parse_reply(text: str) -> tuple[list | None, list[str]]:
    """The annotation array and the deviations accepted: surrounding whitespace, one wrapping code fence."""
    leniency = []
    body = text.strip()
    if body != text:
        leniency.append("whitespace")
    fence = FENCE.fullmatch(body)
    if fence:
        leniency.append("code_fence")
        body = fence.group(1)
        if body != body.strip() and "whitespace" not in leniency:
            leniency.append("whitespace")
        body = body.strip()
    try:
        data = json.loads(body, parse_constant=_reject_constant)
    except ValueError:
        return None, leniency
    if not isinstance(data, list) or not all(isinstance(item, dict) for item in data):
        return None, leniency
    return data, leniency


def main(argv=None, *, scope: v.Scope | None = None, revision: str | None = None,
         environ=os.environ, opener=open_url, sleep=time.sleep) -> int:
    parser = argparse.ArgumentParser(description="Send one scope to a model endpoint under the annotation guide.")
    parser.add_argument("--scope", required=True)
    parser.add_argument("--scopes-dir", type=Path, required=True)
    parser.add_argument("--guide", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--max-output-tokens", type=int, default=16000)
    parser.add_argument("--timeout-s", type=float, default=600)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--endpoint-env", type=Path)
    try:
        args = parser.parse_args(argv)
    except SystemExit as exc:  # argparse exits 2 on usage errors; 2 means parse_failed here
        return 3 if exc.code == 2 else exc.code
    started_utc = datetime.now(timezone.utc).isoformat(timespec="seconds")
    started = time.monotonic()
    if scope is None:
        scopes = v.load_scopes(args.scopes_dir)
        if args.scope not in scopes:
            return fail(f"unknown scope {args.scope}")
        scope = scopes[args.scope]
    if revision is None:
        manifest = json.loads((HERE / "scopes.json").read_text(encoding="utf-8"))
        revision = next(s["revision"] for s in manifest["scopes"] if s["name"] == scope.name)
    out = args.out_dir
    if (out / "call.json").exists():
        return fail(f"refusing to overwrite {out / 'call.json'}")
    cfg, error = load_config(env_path(args.endpoint_env, environ))
    if error:
        return fail(error)
    try:
        guide_bytes = args.guide.read_bytes()
        guide, user = guide_bytes.decode("utf-8"), user_message(scope)
        file_sha = {f: sha((scope.root / f).read_bytes()) for f in scope.files}
    except (OSError, UnicodeDecodeError) as exc:
        return fail(f"cannot read input: {type(exc).__name__}: {exc}")
    url, headers, body = build_request(cfg, guide, user, args.max_output_tokens)
    secret = cfg[KEY]

    def save(name: str, content) -> None:
        text = content if isinstance(content, str) else json.dumps(content, indent=1, ensure_ascii=False) + "\n"
        (out / name).write_text(text.replace(secret, REDACTED), encoding="utf-8", newline="\n")

    call = {
        "scope": scope.name, "scope_revision": revision, "api_style": cfg[STYLE],
        "base_url_host": urllib.parse.urlsplit(cfg[BASE]).hostname, "requested_model": cfg[NAME],
        "returned_model": None, "guide_path": str(args.guide.resolve()), "guide_sha256": sha(guide_bytes),
        "user_template_sha256": sha(USER_TEMPLATE.encode("utf-8")), "user_message_sha256": sha(user.encode("utf-8")),
        "scope_files_sha256": file_sha, "max_output_tokens": args.max_output_tokens, "attempts": 0,
        "input_tokens": None, "output_tokens": None, "stop_reason": None, "parse": None, "leniency": None,
        "annotations": None, "wall_seconds": None, "started_utc": started_utc, "dry_run": args.dry_run,
        "exit_code": None, "error": None,
    }
    out.mkdir(parents=True, exist_ok=True)
    save("request.json", {"url": url, "headers": {k: REDACTED if k.lower() in SECRET_HEADERS else val
                                                  for k, val in headers.items()}, "body": body})
    if args.dry_run:
        call.update(exit_code=0, wall_seconds=round(time.monotonic() - started, 3))
        save("call.json", call)
        print(f"{scope.name}: dry run, system {len(guide)} chars, user {len(user)} chars -> {out}")
        return 0
    save("call.json", call)  # marks the directory as used before any connection is opened
    attempts = send(url, headers, body, args.timeout_s, opener, sleep)
    save("response.json", {"attempts": attempts})
    code, last = 4, attempts[-1]
    call["error"] = last["error"]
    if last["error"] is None:
        try:
            result = extract(cfg[STYLE], last["body"])
        except (ValueError, KeyError, IndexError, TypeError, AttributeError) as exc:
            call["error"] = f"unreadable response envelope: {type(exc).__name__}: {exc}"
        else:
            text = result.pop("text")
            save("reply.txt", text)
            annotations, leniency = parse_reply(text)
            if annotations is not None:
                save("annotations.json", annotations)
            cut_off = result["stop_reason"] == CUT_OFF[cfg[STYLE]]
            code = 0 if annotations is not None and not cut_off else 2
            call.update(result, parse="ok" if annotations is not None else "parse_failed", leniency=leniency,
                        annotations=None if annotations is None else len(annotations))
    call.update(attempts=len(attempts), exit_code=code, wall_seconds=round(time.monotonic() - started, 3))
    save("call.json", call)
    ledger = {"utc": started_utc, "scope": scope.name, "out_dir": out.resolve().name, "model": cfg[NAME],
              "input_tokens": call["input_tokens"], "output_tokens": call["output_tokens"],
              "attempts": len(attempts), "exit_code": code}
    with (out.resolve().parent / "ledger.jsonl").open("a", encoding="utf-8", newline="\n") as fh:
        fh.write(json.dumps(ledger).replace(secret, REDACTED) + "\n")
    print(f"{scope.name}: exit {code}, parse {call['parse']}, {call['annotations']} annotations, "
          f"{call['input_tokens']} in / {call['output_tokens']} out tokens, {len(attempts)} attempts -> {out}")
    return code


if __name__ == "__main__":
    raise SystemExit(main())
