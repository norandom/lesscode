r"""Ask a model to write the data pack for one language, check it, allow one repair.

    uv run --no-sync python author_pack.py --language LANG --examples LANG [LANG ...] --guide FILE --out-dir DIR
        [--scope NAME --scopes-dir DIR] [--max-output-tokens N] [--dry-run] [--endpoint-env FILE]

The guide is the system message. The user message shows the example packs as
they are in tables/ (never AD_TABLES_DIR) and names the language. The reply
must be the pack as one JSON object, with annotate.py's two leniencies
(surrounding whitespace, one whole-reply code fence). The pack is checked with
pack_check.check_pack; when it is not admitted, exactly one repair call follows
with the first reply as an assistant turn and the failed checks. Endpoint
configuration, request building, retries, redaction and reply extraction are
annotate.py's.

Writes attempt{N}-request.json, -response.json, -reply.txt and -check.json and
author.json into the output directory, the admitted pack as {language}.json,
and one ledger.jsonl line per real call in its parent. The API key is never
printed or written.

Exit codes: 0 admitted; 2 not admitted after the allowed attempts; 3
configuration or usage error; 4 request failed after the allowed HTTP tries.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.parse
from datetime import datetime, timezone
from pathlib import Path

import annotate as a
import pack_check as pc

HERE = Path(__file__).parent
DEFAULT_TABLES = HERE / "tables"
TIMEOUT_S = 600  # annotate.py's default --timeout-s
PACK_USER_TEMPLATE = """Example packs:

{examples}

Write the pack for: {language}"""
REPAIR_TEMPLATE = "The pack was checked and not accepted:\n\n{feedback}\n\nReturn the corrected pack as a JSON object only."


def fail(message: str) -> int:
    print(f"author_pack: {message}", file=sys.stderr)
    return 3


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def plain_name(name: str) -> bool:
    return bool(name) and Path(name).name == name and "/" not in name and "\\" not in name


def example_packs(names: list[str]) -> dict[str, bytes]:
    """The exact bytes of tables/{name}.json in the default tables directory."""
    packs = {}
    for name in names:
        path = DEFAULT_TABLES / f"{name}.json"
        if not plain_name(name) or not path.is_file():
            raise FileNotFoundError(f"no example pack tables/{name}.json")
        packs[name] = path.read_bytes()
    return packs


def user_message(language: str, names: list[str], packs: dict[str, bytes]) -> str:
    examples = "\n\n".join(f"{name}.json:\n{packs[name].decode('utf-8')}" for name in names)
    return PACK_USER_TEMPLATE.format(examples=examples, language=language)


def pack_text(reply: str) -> tuple[str, list[str]]:
    """The reply without what annotate.parse_reply tolerates, and which of the two applied."""
    _, leniency = a.parse_reply(reply)
    body = reply.strip()
    if "code_fence" in leniency:
        body = a.FENCE.fullmatch(body).group(1).strip()
    return body, leniency


def main(argv=None, *, environ=os.environ, opener=a.open_url, sleep=time.sleep) -> int:
    parser = argparse.ArgumentParser(description="Ask a model to write the data pack for one language.")
    parser.add_argument("--language", required=True)
    parser.add_argument("--examples", nargs="+", required=True)
    parser.add_argument("--guide", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--scope")
    parser.add_argument("--scopes-dir", type=Path)
    parser.add_argument("--max-output-tokens", type=int, default=4000)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--endpoint-env", type=Path)
    try:
        args = parser.parse_args(argv)
    except SystemExit as exc:  # argparse exits 2 on usage errors; 2 means not admitted here
        return 3 if exc.code == 2 else exc.code
    started_utc, started = utc_now(), time.monotonic()
    out, language = args.out_dir, args.language
    if (out / "author.json").exists():
        return fail(f"refusing to overwrite {out / 'author.json'}")
    if not plain_name(language):
        return fail(f"not a plain language name: {language}")
    if (args.scope is None) != (args.scopes_dir is None):
        return fail("--scope and --scopes-dir go together")
    scope = controls = None
    if args.scope is not None:
        try:
            scope, controls = pc.load_scope_controls(args.scope, args.scopes_dir)
        except KeyError:
            return fail(f"unknown scope or no controls: {args.scope}")
        if scope.language != language:
            return fail(f"scope {scope.name} is {scope.language}, not {language}")
    cfg, error = a.load_config(a.env_path(args.endpoint_env, environ))
    if error:
        return fail(error)
    try:
        guide_bytes = args.guide.read_bytes()
        guide = guide_bytes.decode("utf-8")
        packs = example_packs(args.examples)
        user = user_message(language, args.examples, packs)
    except (OSError, UnicodeDecodeError) as exc:
        return fail(f"cannot read input: {type(exc).__name__}: {exc}")
    secret = cfg[a.KEY]

    def redact(text: str) -> str:
        return text.replace(secret, a.REDACTED)

    def save(name: str, content) -> None:
        text = content if isinstance(content, str) else json.dumps(content, indent=1, ensure_ascii=False) + "\n"
        (out / name).write_text(redact(text), encoding="utf-8", newline="\n")

    def save_request(n: int, url: str, headers: dict, body: dict) -> None:
        save(f"attempt{n}-request.json", {"url": url, "headers": {k: a.REDACTED if k.lower() in a.SECRET_HEADERS else val
                                                                 for k, val in headers.items()}, "body": body})

    record = {
        "language": language, "scope": args.scope, "examples": list(args.examples),
        "guide_path": str(args.guide.resolve()), "guide_sha256": a.sha(guide_bytes),
        "pack_user_template_sha256": a.sha(PACK_USER_TEMPLATE.encode("utf-8")),
        "repair_template_sha256": a.sha(REPAIR_TEMPLATE.encode("utf-8")),
        "example_packs_sha256": {name: a.sha(data) for name, data in packs.items()},
        "user_message_sha256": a.sha(user.encode("utf-8")), "api_style": cfg[a.STYLE],
        "base_url_host": urllib.parse.urlsplit(cfg[a.BASE]).hostname, "requested_model": cfg[a.NAME],
        "returned_model": None, "max_output_tokens": args.max_output_tokens, "attempts": [], "admitted": None,
        "pack_sha256": None, "wall_seconds": None, "started_utc": started_utc, "dry_run": args.dry_run,
        "exit_code": None, "error": None,
    }
    out.mkdir(parents=True, exist_ok=True)
    request = a.build_request(cfg, guide, user, args.max_output_tokens)
    save_request(1, *request)
    if args.dry_run:
        record.update(exit_code=0, wall_seconds=round(time.monotonic() - started, 3))
        save("author.json", record)
        print(f"{language}: dry run, system {len(guide)} chars, user {len(user)} chars -> {out}")
        return 0
    save("author.json", record)  # marks the directory as used before any connection is opened

    def attempt(n: int, request: tuple[str, dict, dict]) -> tuple[int, str | None, str | None, dict | None]:
        """One model call: (code, reply text, pack text, check result); code 0, 2 or 4."""
        call_utc = utc_now()
        sent = a.send(*request, TIMEOUT_S, opener, sleep)
        save(f"attempt{n}-response.json", {"attempts": sent})
        row = {"attempt": n, "started_utc": call_utc, "http_attempts": len(sent), "input_tokens": None,
               "output_tokens": None, "stop_reason": None, "returned_model": None, "cut_off": None,
               "leniency": None, "admitted": False, "failed_checks": None, "error": sent[-1]["error"]}
        code, reply, text, check = 4, None, None, None
        if sent[-1]["error"] is None:
            try:
                result = a.extract(cfg[a.STYLE], sent[-1]["body"])
            except (ValueError, KeyError, IndexError, TypeError, AttributeError) as exc:
                row["error"] = f"unreadable response envelope: {type(exc).__name__}: {exc}"
            else:
                reply = result.pop("text")
                save(f"attempt{n}-reply.txt", reply)
                text, leniency = pack_text(reply)
                check = pc.check_pack(text, language, scope, controls)
                save(f"attempt{n}-check.json", check)
                cut_off = result["stop_reason"] == a.CUT_OFF[cfg[a.STYLE]]
                admitted = check["admitted"] and not cut_off
                row.update(result, cut_off=cut_off, leniency=leniency, admitted=admitted,
                           failed_checks=[c["name"] for c in check["checks"] if not c["pass"]])
                code = 0 if admitted else 2
        record["attempts"].append(row)
        record.update(returned_model=row["returned_model"] or record["returned_model"], error=row["error"])
        save("author.json", record)
        ledger = {"utc": call_utc, "scope": f"pack:{language}", "out_dir": out.resolve().name, "model": cfg[a.NAME],
                  "input_tokens": row["input_tokens"], "output_tokens": row["output_tokens"],
                  "attempts": len(sent), "exit_code": code}
        with (out.resolve().parent / "ledger.jsonl").open("a", encoding="utf-8", newline="\n") as fh:
            fh.write(redact(json.dumps(ledger)) + "\n")
        return code, reply, text, check

    code, reply, text, check = attempt(1, request)
    if code == 2:
        history = [{"role": "user", "content": user}, {"role": "assistant", "content": reply}]
        repair = REPAIR_TEMPLATE.format(feedback=pc.feedback_for_model(check))
        request = a.build_request(cfg, guide, repair, args.max_output_tokens, history)
        save_request(2, *request)
        code, reply, text, check = attempt(2, request)
    if code == 0:
        data = redact(text if text.endswith("\n") else text + "\n").encode("utf-8")
        (out / f"{language}.json").write_bytes(data)
        record["pack_sha256"] = a.sha(data)
    record.update(admitted=code == 0, exit_code=code, wall_seconds=round(time.monotonic() - started, 3))
    save("author.json", record)
    failed = [row["failed_checks"] for row in record["attempts"]]
    print(f"{language}: exit {code}, admitted {code == 0}, {len(record['attempts'])} call(s), "
          f"failed checks per call {failed} -> {out}")
    return code


if __name__ == "__main__":
    raise SystemExit(main())
