"""Tests for annotate.py against a fake endpoint on 127.0.0.1. No model call, no external host.

    uv run --no-sync python test_annotate.py
"""

from __future__ import annotations

import contextlib
import io
import json
import os
import shutil
import socket
import tempfile
import threading
import unittest
import urllib.error
import urllib.request
import uuid
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path

os.environ["NO_PROXY"] = "127.0.0.1,localhost"  # loopback requests must never go through a proxy

import annotate as a  # noqa: E402
import validator as v  # noqa: E402

HERE = Path(__file__).parent
GUIDE = HERE / "guide-v0.md"
DEV = Path(r"D:\BuildArtifacts\lesscode\annotation-driver\dev")
KEY = "sk-test-SECRET-" + uuid.uuid4().hex
ROOT: Path
FROZEN_TEMPLATE = (
    'The file below is shown with line numbers. The numbers and the "| " separator are not part of the file; '
    "anchors must be copied from the text after the separator.\n\nFile: {path}\n{numbered}"
)
LINES = [
    "def check(limit, used):",
    "    café = used + 1  # non-ASCII, leading spaces",
    "\tif café > limit:",
    '        raise ValueError("über")',
    "    return café",
    "",
    "  ",
    'x = {"k": 1}',
    "y = 2",
    "z = 3",
    "end = True",
]
ANNOTATIONS = [{"file": "pkg/mod.py", "start_line": 3, "end_line": 3, "kind": "branch_condition",
                "anchor": "café > limit", "reason": "callers over the limit would not be refused"}]
CALL_KEYS = {
    "scope", "scope_revision", "api_style", "base_url_host", "requested_model", "returned_model", "guide_path",
    "guide_sha256", "user_template_sha256", "user_message_sha256", "scope_files_sha256", "max_output_tokens",
    "attempts", "input_tokens", "output_tokens", "stop_reason", "parse", "leniency", "annotations",
    "wall_seconds", "started_utc", "dry_run", "exit_code", "error",
}


def setUpModule():
    global ROOT
    DEV.mkdir(parents=True, exist_ok=True)
    ROOT = Path(tempfile.mkdtemp(dir=DEV, prefix="test-annotate-"))
    proxies = urllib.request.getproxies()
    assert "http" not in proxies or urllib.request.proxy_bypass("127.0.0.1"), "loopback would use a proxy"


def tearDownModule():
    shutil.rmtree(ROOT, ignore_errors=True)


def anthropic_reply(text: str, stop: str = "end_turn") -> dict:
    half = len(text) // 2
    return {"id": "msg_1", "type": "message", "role": "assistant", "model": "test-model-returned",
            "content": [{"type": "thinking", "thinking": "not part of the reply"},
                        {"type": "text", "text": text[:half]}, {"type": "text", "text": text[half:]}],
            "stop_reason": stop, "usage": {"input_tokens": 1234, "output_tokens": 56}}


def openai_reply(text: str, finish: str = "stop") -> dict:
    return {"id": "c1", "object": "chat.completion", "model": "test-model-returned",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": text}, "finish_reason": finish}],
            "usage": {"prompt_tokens": 1234, "completion_tokens": 56, "total_tokens": 1290}}


def env_text(style: str, base: str) -> str:
    return (f"# endpoint for tests\nLESSCODE_MODEL_API_STYLE = {style}\nLESSCODE_MODEL_BASE_URL={base}  # fake\n"
            f"LESSCODE_MODEL_NAME=test-model\nLESSCODE_MODEL_API_KEY={KEY}\n")


def closed_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Base(unittest.TestCase):
    def setUp(self):
        case = ROOT / self._testMethodName
        self.dir, self.cfg = case / "work", case / "cfg"
        clone = self.dir / "clone"
        (clone / "pkg").mkdir(parents=True)
        self.cfg.mkdir(parents=True)
        (clone / "pkg" / "mod.py").write_bytes(("\r\n".join(LINES) + "\r\n").encode("utf-8"))
        (clone / "b.py").write_bytes(b"print(1)")
        self.scope = v.Scope("fake-scope", "python", clone, ("pkg/mod.py",), ("tests/",))
        self.sleeps: list[float] = []
        self.seen: list[dict] = []

    def serve(self, *replies) -> str:
        """Start a fake endpoint that answers with (status, payload) pairs in order."""
        queue, seen = list(replies), self.seen

        class Handler(BaseHTTPRequestHandler):
            def do_POST(self):
                raw = self.rfile.read(int(self.headers["Content-Length"]))
                seen.append({"path": self.path, "headers": {k.lower(): v for k, v in self.headers.items()},
                             "body": json.loads(raw)})
                status, payload = queue.pop(0)
                data = (payload if isinstance(payload, str) else json.dumps(payload)).encode("utf-8")
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)

            def log_message(self, *args):
                pass

        server = HTTPServer(("127.0.0.1", 0), Handler)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        return f"http://127.0.0.1:{server.server_port}"

    def call(self, base: str, *, style="anthropic", name="call", extra=(), opener=a.open_url, env=None, environ=None):
        env_file = self.cfg / f"{name}.env"
        env_file.write_text(env if env is not None else env_text(style, base), encoding="utf-8", newline="\n")
        out = self.dir / name
        argv = ["--scope", self.scope.name, "--scopes-dir", str(self.dir), "--guide", str(GUIDE),
                "--out-dir", str(out), "--endpoint-env", str(env_file), *extra]
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(buf):
            code = a.main(argv, scope=self.scope, revision="rev-0", opener=opener, sleep=self.sleeps.append,
                          environ=environ if environ is not None else {})
        output = buf.getvalue()
        self.assertNotIn(KEY, output)
        for path in self.dir.rglob("*"):
            if path.is_file():
                self.assertNotIn(KEY.encode(), path.read_bytes(), path)
        return code, out, output

    def call_json(self, out: Path) -> dict:
        return json.loads((out / "call.json").read_text(encoding="utf-8"))

    def reply_case(self, text: str, name: str, *, stop="end_turn", style="anthropic") -> tuple[int, Path, dict]:
        reply = anthropic_reply(text, stop) if style == "anthropic" else openai_reply(text, stop)
        code, out, _ = self.call(self.serve((200, reply)), style=style, name=name)
        return code, out, self.call_json(out)


class Styles(Base):
    def check_files(self, out: Path, text: str, secret_header: str) -> dict:
        for name in ("request.json", "response.json", "reply.txt", "annotations.json", "call.json"):
            self.assertTrue((out / name).is_file(), name)
        self.assertEqual((out / "reply.txt").read_text(encoding="utf-8", newline=""), text)
        self.assertEqual(json.loads((out / "annotations.json").read_text(encoding="utf-8")), ANNOTATIONS)
        request = json.loads((out / "request.json").read_text(encoding="utf-8"))
        self.assertEqual(request["headers"][secret_header], "<redacted>")
        self.assertEqual(request["body"], self.seen[0]["body"])
        call = self.call_json(out)
        self.assertEqual(set(call), CALL_KEYS)
        self.assertEqual((call["parse"], call["leniency"], call["annotations"]), ("ok", [], 1))
        self.assertEqual((call["input_tokens"], call["output_tokens"], call["attempts"]), (1234, 56, 1))
        self.assertEqual((call["requested_model"], call["returned_model"]), ("test-model", "test-model-returned"))
        self.assertEqual((call["base_url_host"], call["scope_revision"], call["dry_run"]), ("127.0.0.1", "rev-0", False))
        self.assertEqual(call["guide_sha256"], a.sha(GUIDE.read_bytes()))
        self.assertEqual(call["user_message_sha256"], a.sha(a.user_message(self.scope).encode("utf-8")))
        self.assertEqual(call["scope_files_sha256"], {"pkg/mod.py": a.sha((self.scope.root / "pkg/mod.py").read_bytes())})
        return call

    def test_anthropic(self):
        text = json.dumps(ANNOTATIONS)
        code, out, output = self.call(self.serve((200, anthropic_reply(text))))
        self.assertEqual(code, 0, output)
        req = self.seen[0]
        self.assertEqual(req["path"], "/v1/messages")
        self.assertEqual(req["headers"]["x-api-key"], KEY)
        self.assertEqual(req["headers"]["anthropic-version"], "2023-06-01")
        self.assertEqual(req["headers"]["content-type"], "application/json")
        self.assertNotIn("authorization", req["headers"])
        self.assertEqual(req["body"], {"model": "test-model", "max_tokens": 16000,
                                       "system": GUIDE.read_text(encoding="utf-8"),
                                       "messages": [{"role": "user", "content": a.user_message(self.scope)}]})
        call = self.check_files(out, text, "x-api-key")
        self.assertEqual((call["api_style"], call["stop_reason"]), ("anthropic", "end_turn"))

    def test_openai(self):
        text = json.dumps(ANNOTATIONS)
        code, out, output = self.call(self.serve((200, openai_reply(text))) + "/v1/", style="openai",
                                      extra=["--max-output-tokens", "777"])
        self.assertEqual(code, 0, output)
        req = self.seen[0]
        self.assertEqual(req["path"], "/v1/chat/completions")
        self.assertEqual(req["headers"]["authorization"], f"Bearer {KEY}")
        self.assertNotIn("x-api-key", req["headers"])
        self.assertEqual(req["body"], {"model": "test-model", "max_tokens": 777, "messages": [
            {"role": "system", "content": GUIDE.read_text(encoding="utf-8")},
            {"role": "user", "content": a.user_message(self.scope)}]})
        call = self.check_files(out, text, "Authorization")
        self.assertEqual((call["api_style"], call["stop_reason"]), ("openai", "stop"))

    def test_base_url_variants(self):
        for n, suffix in enumerate(("", "/", "/v1", "/v1/")):
            self.seen.clear()
            code, _, output = self.call(self.serve((200, anthropic_reply("[]"))) + suffix, name=f"base{n}")
            self.assertEqual(code, 0, output)
            self.assertEqual(self.seen[0]["path"], "/v1/messages", suffix)


class Retries(Base):
    def attempts(self, out: Path) -> list:
        return [x["status"] for x in json.loads((out / "response.json").read_text(encoding="utf-8"))["attempts"]]

    def test_429_then_success(self):
        code, out, _ = self.call(self.serve((429, {"error": "slow down"}), (200, anthropic_reply("[]"))))
        self.assertEqual((code, self.call_json(out)["attempts"], self.attempts(out)), (0, 2, [429, 200]))
        self.assertEqual(self.sleeps, [10])

    def test_400_no_retry(self):
        code, out, _ = self.call(self.serve((400, {"error": "bad request"})))
        call = self.call_json(out)
        self.assertEqual((code, call["attempts"], self.attempts(out), self.sleeps), (4, 1, [400], []))
        self.assertIsNone(call["parse"])
        self.assertIn("400", call["error"])
        self.assertFalse((out / "reply.txt").exists())

    def test_two_500s(self):
        code, out, _ = self.call(self.serve((500, "oops"), (500, "oops")))
        self.assertEqual((code, self.call_json(out)["attempts"], self.attempts(out)), (4, 2, [500, 500]))
        self.assertEqual((len(self.seen), self.sleeps), (2, [10]))

    def test_connection_error_twice(self):
        calls = []

        def refuse(request, timeout):
            calls.append(request.full_url)
            raise urllib.error.URLError("connection refused")

        code, out, _ = self.call("http://127.0.0.1:9", opener=refuse)
        self.assertEqual((code, len(calls), self.call_json(out)["attempts"], self.sleeps), (4, 2, 2, [10]))

    def test_timeout_then_success(self):
        calls = []

        def flaky(request, timeout):
            calls.append(timeout)
            if len(calls) == 1:
                raise TimeoutError("timed out")
            return a.open_url(request, timeout)

        code, out, _ = self.call(self.serve((200, anthropic_reply("[]"))), opener=flaky, extra=["--timeout-s", "30"])
        self.assertEqual((code, calls, self.attempts(out)), (0, [30.0, 30.0], [None, 200]))

    def test_unreadable_envelope(self):
        code, out, _ = self.call(self.serve((200, "not json")))
        call = self.call_json(out)
        self.assertEqual((code, call["attempts"], call["parse"]), (4, 1, None))
        self.assertIn("envelope", call["error"])


class Replies(Base):
    def test_variants(self):
        bare = json.dumps(ANNOTATIONS)
        cases = [
            (bare, 0, "ok", []),
            ("\n  " + bare + "\n", 0, "ok", ["whitespace"]),
            ("```json\n" + bare + "\n```", 0, "ok", ["code_fence"]),
            ("```\n" + bare + "\n```", 0, "ok", ["code_fence"]),
            ("\n```json\n" + bare + "\n```\n", 0, "ok", ["whitespace", "code_fence"]),
            ("Here are the marks:\n" + bare, 2, "parse_failed", []),
            (bare + "\nDone.", 2, "parse_failed", []),
            ('{"annotations": ' + bare + "}", 2, "parse_failed", []),
            ("[1, 2]", 2, "parse_failed", []),
            ('[{"start_line": NaN}]', 2, "parse_failed", []),
            ("```json\n" + bare + "\n```\n```json\n[]\n```", 2, "parse_failed", ["code_fence"]),
        ]
        for n, (text, code_expected, parse, leniency) in enumerate(cases):
            with self.subTest(text=text[:40]):
                code, out, call = self.reply_case(text, f"r{n}")
                self.assertEqual((code, call["parse"], call["leniency"]), (code_expected, parse, leniency))
                self.assertEqual((out / "annotations.json").exists(), parse == "ok")
                self.assertEqual((out / "reply.txt").read_text(encoding="utf-8", newline=""), text)

    def test_cut_off(self):
        code, out, call = self.reply_case('[{"file": "pkg', "cut", stop="max_tokens")
        self.assertEqual((code, call["parse"], call["stop_reason"]), (2, "parse_failed", "max_tokens"))
        code, out, call = self.reply_case("[]", "cut-parsed", stop="max_tokens")
        self.assertEqual((code, call["parse"]), (2, "ok"))
        code, out, call = self.reply_case("[]", "cut-openai", stop="length", style="openai")
        self.assertEqual((code, call["stop_reason"]), (2, "length"))


class Local(Base):
    def test_dry_run_opens_no_connection(self):
        base = f"http://127.0.0.1:{closed_port()}"
        code, out, output = self.call(base, extra=["--dry-run"])  # real opener: a connection attempt would fail
        self.assertEqual(code, 0, output)
        self.assertIn("dry run", output)
        call = self.call_json(out)
        self.assertEqual((call["dry_run"], call["attempts"], call["exit_code"]), (True, 0, 0))
        self.assertEqual(json.loads((out / "request.json").read_text(encoding="utf-8"))["url"], base + "/v1/messages")
        self.assertEqual(sorted(p.name for p in out.iterdir()), ["call.json", "request.json"])
        calls = []
        code, _, _ = self.call(base, name="dry2", extra=["--dry-run"], opener=lambda *x: calls.append(x))
        self.assertEqual((code, calls), (0, []))
        self.assertFalse((self.dir / "ledger.jsonl").exists())

    def test_refuses_existing_call(self):
        out = self.dir / "call"
        out.mkdir()
        (out / "call.json").write_text("{}", encoding="utf-8")
        base = self.serve()
        code, _, output = self.call(base)
        self.assertEqual(code, 3)
        self.assertIn("refusing", output)
        self.assertEqual((out / "call.json").read_text(encoding="utf-8"), "{}")
        self.assertEqual((sorted(p.name for p in out.iterdir()), self.seen), (["call.json"], []))
        code, _, _ = self.call(base, extra=["--dry-run"])
        self.assertEqual(code, 3)

    def test_config_errors(self):
        full = env_text("anthropic", "http://127.0.0.1:9")
        for n, key in enumerate(("LESSCODE_MODEL_API_STYLE", "LESSCODE_MODEL_BASE_URL",
                                 "LESSCODE_MODEL_NAME", "LESSCODE_MODEL_API_KEY")):
            env = "".join(line + "\n" for line in full.splitlines() if not line.startswith(key))
            code, out, output = self.call("", name=f"missing{n}", env=env)
            self.assertEqual(code, 3)
            self.assertIn(key, output)
            self.assertFalse(out.exists())
        code, _, output = self.call("", name="empty-key", env=full.replace(KEY, ""))
        self.assertEqual(code, 3)
        self.assertIn("LESSCODE_MODEL_API_KEY", output)
        code, _, output = self.call("", name="style", env=full.replace("anthropic", "SECRET-STYLE-VALUE"))
        self.assertEqual(code, 3)
        self.assertIn("LESSCODE_MODEL_API_STYLE", output)
        self.assertNotIn("SECRET-STYLE-VALUE", output)

    def test_config_path_and_usage(self):
        self.assertEqual(a.env_path(Path("x.env"), {"LESSCODE_MODEL_ENV": "y.env"}), Path("x.env"))
        self.assertEqual(a.env_path(None, {"LESSCODE_MODEL_ENV": "y.env", "USERPROFILE": "u"}), Path("y.env"))
        self.assertEqual(a.env_path(None, {"USERPROFILE": "u"}), Path("u") / ".lesscode" / "model.env")
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(buf):
            missing_file = a.main(["--scope", "s", "--scopes-dir", ".", "--guide", str(GUIDE),
                                   "--out-dir", str(self.dir / "o")], scope=self.scope, revision="r",
                                  environ={"LESSCODE_MODEL_ENV": str(self.cfg / "absent.env")})
            usage = a.main(["--scopes-dir", "."])
        self.assertEqual((missing_file, usage), (3, 3))
        self.assertIn("not found", buf.getvalue())

    def test_user_message_format(self):
        self.assertEqual(a.USER_TEMPLATE, FROZEN_TEMPLATE)
        intro = FROZEN_TEMPLATE.split("\n\n")[0]
        prefixes = [f"    {n}| " for n in range(1, 10)] + ["   10| ", "   11| "]
        body = "\n".join(p + line for p, line in zip(prefixes, LINES))
        message = a.user_message(self.scope)
        self.assertEqual(message, intro + "\n\nFile: pkg/mod.py\n" + body)
        self.assertEqual(message, FROZEN_TEMPLATE.format(path="pkg/mod.py", numbered=body))
        self.assertIn("\n    2|     café = used + 1", message)
        self.assertIn("\n    3| \tif café > limit:", message)
        self.assertIn("\n    7|   \n", message)
        self.assertNotIn("\r", message)
        two = v.Scope("two", "python", self.scope.root, ("pkg/mod.py", "b.py"), ())
        message = a.user_message(two)
        self.assertEqual(message.count(intro), 1)
        self.assertEqual(message, intro + "\n\nFile: pkg/mod.py\n" + body + "\n\nFile: b.py\n    1| print(1)")

    def test_ledger(self):
        ledger = self.dir / "ledger.jsonl"
        self.call(self.serve((200, anthropic_reply("[]"))), name="first")
        self.call(f"http://127.0.0.1:{closed_port()}", name="dry", extra=["--dry-run"])
        self.call(self.serve((400, "no")), name="second")
        rows = [json.loads(line) for line in ledger.read_text(encoding="utf-8").splitlines()]
        self.assertEqual([(r["out_dir"], r["exit_code"], r["attempts"]) for r in rows], [("first", 0, 1), ("second", 4, 1)])
        self.assertEqual(set(rows[0]), {"utc", "scope", "out_dir", "model", "input_tokens", "output_tokens",
                                        "attempts", "exit_code"})
        self.assertEqual((rows[0]["model"], rows[0]["input_tokens"], rows[0]["output_tokens"]), ("test-model", 1234, 56))
        self.assertNotIn(b"\r", ledger.read_bytes())


if __name__ == "__main__":
    unittest.main(verbosity=2)
