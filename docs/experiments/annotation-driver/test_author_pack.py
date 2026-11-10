"""Tests for pack_check.py, author_pack.py and the AD_TABLES_DIR override. No model call, no external host.

    uv run --no-sync python test_author_pack.py

The fake endpoint listens on 127.0.0.1. Temporary files go under
D:\\BuildArtifacts\\lesscode\\annotation-driver\\dev. The controls run against a
read-only snapshot of the scope files: a mutation batch may be writing mutants
into the real clones, so each file is copied once it has its frozen bytes.
"""

from __future__ import annotations

import contextlib
import hashlib
import io
import json
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
import unittest
import urllib.request
import uuid
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path

os.environ["NO_PROXY"] = "127.0.0.1,localhost"  # loopback requests must never go through a proxy
os.environ.pop("AD_TABLES_DIR", None)

import annotate as a  # noqa: E402
import author_pack as ap  # noqa: E402
import pack_check as pc  # noqa: E402
import validator as v  # noqa: E402

HERE = Path(__file__).parent
GUIDE = HERE / "pack-guide-v0.md"
DEV = Path(r"D:\BuildArtifacts\lesscode\annotation-driver\dev")
REAL_SCOPES = Path(r"D:\BuildArtifacts\lesscode\annotation-driver\scopes")
KEY = "sk-test-SECRET-" + uuid.uuid4().hex
# Scope files at the scopes.json revisions, as recorded by the R58 controls receipt.
FROZEN_SHA256 = {
    "rust-mmhmem-setup-update": {"src/setup/ui/update.rs":
                                 "804522e68de5d5059369418ab8190baf718eaed06c821ac007ab03661878381e"},
    "python-mvibe-ratelimit": {"mvibe_proxy/ratelimit.py":
                               "027e304481efc745ec8c04b9784d8e3efa54fd8516d197058ea4c928ffab6828"},
    "typescript-openultracode-state": {"src/state.ts":
                                       "3c9afdb17729b713455f3ee768f05e54e2511f4dadb550c9a70a4455deac7ad9"},
    "typescript-openultracode-verification": {"src/verification.ts":
                                              "9908a3ad63cba13f813ca75a7fb441ff09c377698afe9dee62a738cbdee3c8c0"},
    "java-commons-csv-lexer": {"src/main/java/org/apache/commons/csv/Lexer.java":
                               "8a57dc0759e26dc91f79942d8cc0a1eb6c3f53449fb31da4dedf4a6cb9ff0e0c"},
}
SNAPSHOT_WAIT_S = 3600
FROZEN_USER_TEMPLATE = "Example packs:\n\n{examples}\n\nWrite the pack for: {language}"
FROZEN_REPAIR_TEMPLATE = ("The pack was checked and not accepted:\n\n{feedback}\n\n"
                          "Return the corrected pack as a JSON object only.")
CHECK_NAMES = ["json", "size", "shape", "kinds_valid", "templates", "controls", "seed_mutants"]
ROOT: Path
SCOPES_DIR: Path
CONTROLS: dict


def pack(language: str) -> str:
    return (HERE / "tables" / f"{language}.json").read_bytes().decode("utf-8")


PYTHON = pack("python")
BROKEN_PYTHON = PYTHON.replace('["raise_statement", "return_statement"]', '["return_statement"]')


def snapshot_scopes(dest: Path) -> None:
    """Copy each scope file from the real clones, read only, once it has its frozen bytes."""
    manifest = json.loads((HERE / "scopes.json").read_text(encoding="utf-8"))
    for s in manifest["scopes"]:
        for f in s["files"]:
            source, target = REAL_SCOPES / s["repository"] / f, dest / s["repository"] / f
            deadline = time.monotonic() + SNAPSHOT_WAIT_S
            while True:
                data = source.read_bytes()
                if hashlib.sha256(data).hexdigest() == FROZEN_SHA256[s["name"]][f]:
                    break
                if time.monotonic() > deadline:
                    raise AssertionError(f"{source} did not return to its frozen bytes")
                time.sleep(0.05)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)


def setUpModule():
    global ROOT, SCOPES_DIR, CONTROLS
    DEV.mkdir(parents=True, exist_ok=True)
    ROOT = Path(tempfile.mkdtemp(dir=DEV, prefix="test-author-pack-"))
    (ROOT / "tmp").mkdir()
    tempfile.tempdir = str(ROOT / "tmp")  # check_pack's temporary pack directories
    SCOPES_DIR = ROOT / "scopes"
    snapshot_scopes(SCOPES_DIR)
    CONTROLS = json.loads((HERE / "controls.json").read_text(encoding="utf-8"))["scopes"]
    proxies = urllib.request.getproxies()
    assert "http" not in proxies or urllib.request.proxy_bypass("127.0.0.1"), "loopback would use a proxy"


def tearDownModule():
    tempfile.tempdir = None
    shutil.rmtree(ROOT, ignore_errors=True)


def scope_of(name: str) -> v.Scope:
    return v.load_scopes(SCOPES_DIR)[name]


def failed(result: dict) -> set[str]:
    return {c["name"] for c in result["checks"] if not c["pass"]}


def detail(result: dict, name: str) -> str:
    return next(c["detail"] for c in result["checks"] if c["name"] == name)


def anthropic_reply(text: str, stop: str = "end_turn") -> dict:
    return {"id": "msg_1", "type": "message", "role": "assistant", "model": "test-model-returned",
            "content": [{"type": "text", "text": text}], "stop_reason": stop,
            "usage": {"input_tokens": 2100, "output_tokens": 300}}


def openai_reply(text: str, finish: str = "stop") -> dict:
    return {"id": "c1", "object": "chat.completion", "model": "test-model-returned",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": text}, "finish_reason": finish}],
            "usage": {"prompt_tokens": 2100, "completion_tokens": 300}}


def closed_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class PackChecks(unittest.TestCase):
    def check(self, text: str, language: str = "python") -> dict:
        return pc.check_pack(text, language, None, None)

    def test_existing_packs_admitted(self):
        for name, spec in CONTROLS.items():
            scope = scope_of(name)
            with self.subTest(scope=name):
                result = pc.check_pack(pack(scope.language), scope.language, scope, spec)
                self.assertEqual([c["name"] for c in result["checks"]], CHECK_NAMES)
                self.assertTrue(result["admitted"], json.dumps(result, indent=1))
                self.assertEqual(pc.feedback_for_model(result), "")

    def test_not_json(self):
        result = self.check('{"language": "python",')
        self.assertFalse(result["admitted"])
        self.assertEqual(failed(result), {"json", "shape", "kinds_valid", "templates"})
        self.assertIn("not valid JSON", detail(result, "json"))
        self.assertIn("not run", detail(result, "shape"))
        result = self.check("[1, 2]")
        self.assertIn("array", detail(result, "json"))
        self.assertIn("json", failed(self.check('{"a": NaN}')))

    def test_size(self):
        lines = PYTHON.rstrip("\n").split("\n")

        def padded(n: int) -> str:
            return "\n".join(lines[:-1] + [""] * (n - len(lines)) + lines[-1:]) + "\n"

        self.assertEqual(failed(self.check(padded(40))), set())
        result = self.check(padded(41))
        self.assertEqual(failed(result), {"size"})
        self.assertIn("41 lines", detail(result, "size"))
        widen = '"language": "python",'

        def wide(n: int) -> str:
            return PYTHON.replace(widen, '"language":' + " " * (n - len(PYTHON) + 1) + '"python",')

        self.assertEqual(len(wide(2500)), 2500)
        self.assertEqual(failed(self.check(wide(2500))), set())
        result = self.check(wide(2501))
        self.assertEqual(failed(result), {"size"})
        self.assertIn("2501 characters", detail(result, "size"))

    def test_shape(self):
        missing = PYTHON.replace('  "noop_statement": "pass",\n', "")
        result = self.check(missing)
        self.assertEqual(failed(result), {"shape"})
        self.assertIn("missing key noop_statement", detail(result, "shape"))
        unknown = PYTHON.replace('"language": "python",', '"language": "python", "extra": true, "zeta": 1,')
        result = self.check(unknown)
        self.assertEqual(failed(result), {"shape"})
        self.assertIn("unknown top-level keys: extra, zeta", detail(result, "shape"))
        result = self.check(PYTHON.replace('"language": "python"', '"language": "rust"'))
        self.assertEqual(failed(result), {"shape"})
        self.assertIn('language is "rust", expected "python"', detail(result, "shape"))
        result = self.check(PYTHON.replace('"comment_kinds": ["comment"]', '"comment_kinds": "comment"'))
        self.assertEqual(failed(result), {"shape"})
        self.assertIn("comment_kinds must be a list of strings", detail(result, "shape"))
        result = self.check(PYTHON.replace('"pure_shell_boundary": {"nodes": ["function_definition"]}',
                                           '"other": {"nodes": ["function_definition"]}'))
        self.assertIn("missing kinds.pure_shell_boundary", detail(result, "shape"))

    def test_invalid_kind(self):
        result = self.check(PYTHON.replace('"augmented_assignment"]', '"augmented_assignment_statement"]'))
        self.assertEqual(failed(result), {"kinds_valid"})
        self.assertEqual(detail(result, "kinds_valid"),
                         "node kinds not in the python grammar: augmented_assignment_statement")
        self.assertIn("no grammar", detail(self.check(PYTHON, "no-such-language"), "kinds_valid"))

    def test_templates(self):
        result = self.check(PYTHON.replace('"not ({text})"', '"not (x)"'))
        self.assertEqual(failed(result), {"templates"})
        self.assertIn("does not contain {text}", detail(result, "templates"))
        result = self.check(PYTHON.replace('{"raise_statement": "{noop}"}', '{"raise_statement": "{nop}"}'))
        self.assertEqual(failed(result), {"templates"})
        self.assertIn("by_kind.raise_statement uses unknown placeholders: {nop}", detail(result, "templates"))

    def test_broken_control(self):
        name = "python-mvibe-ratelimit"
        scope = scope_of(name)
        result = pc.check_pack(BROKEN_PYTHON, "python", scope, CONTROLS[name])
        self.assertFalse(result["admitted"])
        self.assertIn("controls", failed(result))
        self.assertFalse(failed(result) & {"json", "size", "shape", "kinds_valid", "templates"})
        feedback = pc.feedback_for_model(result)
        self.assertIn('seed:failure_state@108 expected ["ok"] got ["kind_mismatch"]', feedback)
        self.assertIn("no_site 1", detail(result, "seed_mutants"))
        source = scope.source("mvibe_proxy/ratelimit.py")
        for line in source.splitlines():
            if len(line.strip()) >= 12:
                self.assertNotIn(line.strip(), feedback)
        for seed in CONTROLS[name]["seeds"]:
            self.assertNotIn(seed["anchor"], feedback)

    def test_feedback_lists_only_failures(self):
        result = self.check(PYTHON.replace('"not ({text})"', '"not (x)"').replace('"pass"', '"pass", "extra": 1'))
        feedback = pc.feedback_for_model(result)
        self.assertEqual([line.split(":")[0] for line in feedback.splitlines()], ["- shape", "- templates"])

    def test_cli(self):
        good, bad = ROOT / "cli-good.json", ROOT / "cli-bad.json"
        good.write_text(PYTHON, encoding="utf-8", newline="\n")
        bad.write_text(PYTHON + "\n" * 30, encoding="utf-8", newline="\n")
        for path, expected in ((good, 0), (bad, 1)):
            out = ROOT / f"{path.stem}-result.json"
            proc = subprocess.run([sys.executable, str(HERE / "pack_check.py"), "--pack", str(path),
                                   "--language", "python", "--out", str(out)], cwd=HERE, capture_output=True, text=True)
            self.assertEqual(proc.returncode, expected, proc.stdout + proc.stderr)
            self.assertEqual(json.loads(out.read_text(encoding="utf-8"))["admitted"], expected == 0)


class TablesOverride(unittest.TestCase):
    def tearDown(self):
        os.environ.pop("AD_TABLES_DIR", None)

    def test_load_table(self):
        override = ROOT / "override"
        override.mkdir(exist_ok=True)
        (override / "python.json").write_text(json.dumps({"language": "python", "marker": 1}), encoding="utf-8")
        default = json.loads(PYTHON)
        self.assertEqual(v.load_table("python"), default)
        self.assertEqual(v.tables_dir(), HERE / "tables")
        os.environ["AD_TABLES_DIR"] = str(override)
        self.assertEqual(v.load_table("python"), {"language": "python", "marker": 1})
        self.assertEqual(v.tables_dir(), override)
        os.environ["AD_TABLES_DIR"] = ""
        self.assertEqual(v.load_table("python"), default)
        del os.environ["AD_TABLES_DIR"]
        self.assertEqual(v.load_table("python"), default)

    def test_run_controls_scope_option(self):
        name = "rust-mmhmem-setup-update"
        out = ROOT / "controls-rust.json"
        proc = subprocess.run([sys.executable, str(HERE / "run_controls.py"), "--scopes-dir", str(SCOPES_DIR),
                               "--out", str(out), "--scope", name], cwd=HERE, capture_output=True, text=True)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        results = json.loads(out.read_text(encoding="utf-8"))["results"]
        self.assertEqual({r["scope"] for r in results}, {name})
        self.assertIn(f"{len(results)} of {len(results)} controls", proc.stdout)
        proc = subprocess.run([sys.executable, str(HERE / "run_controls.py"), "--scopes-dir", str(SCOPES_DIR),
                               "--out", str(ROOT / "x.json"), "--scope", "no-such-scope"], cwd=HERE,
                              capture_output=True, text=True)
        self.assertEqual(proc.returncode, 2)


class Author(unittest.TestCase):
    def setUp(self):
        self.dir = ROOT / self._testMethodName
        self.dir.mkdir()
        self.seen: list[dict] = []
        self.sleeps: list[float] = []

    def serve(self, *replies) -> str:
        queue, seen = list(replies), self.seen

        class Handler(BaseHTTPRequestHandler):
            def do_POST(self):
                raw = self.rfile.read(int(self.headers["Content-Length"]))
                seen.append({"path": self.path, "headers": {k.lower(): val for k, val in self.headers.items()},
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

    def run_author(self, base: str, *, style="anthropic", name="out", language="python", examples=("rust", "typescript"),
                   extra=(), opener=a.open_url) -> tuple[int, Path, str]:
        env = self.dir / f"{name}.env"
        env.write_text(f"LESSCODE_MODEL_API_STYLE={style}\nLESSCODE_MODEL_BASE_URL={base}\n"
                       f"LESSCODE_MODEL_NAME=test-model\nLESSCODE_MODEL_API_KEY={KEY}\n", encoding="utf-8")
        out = self.dir / name
        argv = ["--language", language, "--examples", *examples, "--guide", str(GUIDE), "--out-dir", str(out),
                "--endpoint-env", str(env), *extra]
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(buf):
            code = ap.main(argv, environ={}, opener=opener, sleep=self.sleeps.append)
        output = buf.getvalue()
        self.assertNotIn(KEY, output)
        for path in self.dir.rglob("*"):
            if path.is_file() and path.suffix != ".env":
                self.assertNotIn(KEY.encode(), path.read_bytes(), path)
        return code, out, output

    def author(self, out: Path) -> dict:
        return json.loads((out / "author.json").read_text(encoding="utf-8"))

    def ledger(self) -> list[dict]:
        path = self.dir / "ledger.jsonl"
        return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()] if path.exists() else []

    def test_user_message(self):
        self.assertEqual(ap.PACK_USER_TEMPLATE, FROZEN_USER_TEMPLATE)
        self.assertEqual(ap.REPAIR_TEMPLATE, FROZEN_REPAIR_TEMPLATE)
        override = self.dir / "override"
        override.mkdir()
        (override / "rust.json").write_text("{}", encoding="utf-8")
        os.environ["AD_TABLES_DIR"] = str(override)
        try:
            code, out, _ = self.run_author(f"http://127.0.0.1:{closed_port()}", language="typescript",
                                           examples=("rust", "python"), extra=["--dry-run"])
        finally:
            del os.environ["AD_TABLES_DIR"]
        self.assertEqual(code, 0)
        body = json.loads((out / "attempt1-request.json").read_text(encoding="utf-8"))["body"]
        expected = ("Example packs:\n\nrust.json:\n" + pack("rust") + "\n\npython.json:\n" + pack("python")
                    + "\n\nWrite the pack for: typescript")
        self.assertEqual(body["messages"], [{"role": "user", "content": expected}])
        self.assertEqual(body["system"], GUIDE.read_bytes().decode("utf-8"))
        record = self.author(out)
        self.assertEqual(record["example_packs_sha256"],
                         {n: a.sha((HERE / "tables" / f"{n}.json").read_bytes()) for n in ("rust", "python")})
        self.assertEqual(record["pack_user_template_sha256"], a.sha(FROZEN_USER_TEMPLATE.encode("utf-8")))
        self.assertEqual(record["repair_template_sha256"], a.sha(FROZEN_REPAIR_TEMPLATE.encode("utf-8")))

    def test_admitted_first(self):
        reply = PYTHON.rstrip("\n")
        code, out, output = self.run_author(self.serve((200, anthropic_reply(reply))))
        self.assertEqual(code, 0, output)
        self.assertEqual(len(self.seen), 1)
        request = self.seen[0]
        self.assertEqual(request["path"], "/v1/messages")
        self.assertEqual(request["headers"]["x-api-key"], KEY)
        self.assertEqual(request["body"]["max_tokens"], 4000)
        self.assertEqual(request["body"]["system"], GUIDE.read_bytes().decode("utf-8"))
        self.assertEqual(len(request["body"]["messages"]), 1)
        self.assertEqual((out / "python.json").read_bytes(), (reply + "\n").encode("utf-8"))
        self.assertEqual((out / "python.json").read_bytes(), (HERE / "tables" / "python.json").read_bytes())
        self.assertEqual(sorted(p.name for p in out.iterdir()),
                         ["attempt1-check.json", "attempt1-reply.txt", "attempt1-request.json",
                          "attempt1-response.json", "author.json", "python.json"])
        saved = json.loads((out / "attempt1-request.json").read_text(encoding="utf-8"))
        self.assertEqual((saved["headers"]["x-api-key"], saved["body"]), ("<redacted>", request["body"]))
        record = self.author(out)
        self.assertEqual((record["admitted"], record["exit_code"], record["dry_run"]), (True, 0, False))
        self.assertEqual(record["pack_sha256"], a.sha((out / "python.json").read_bytes()))
        self.assertEqual((record["requested_model"], record["returned_model"], record["base_url_host"]),
                         ("test-model", "test-model-returned", "127.0.0.1"))
        self.assertEqual(record["guide_sha256"], a.sha(GUIDE.read_bytes()))
        row = record["attempts"][0]
        self.assertEqual((row["input_tokens"], row["output_tokens"], row["stop_reason"], row["leniency"],
                          row["admitted"], row["failed_checks"]), (2100, 300, "end_turn", [], True, []))
        rows = self.ledger()
        self.assertEqual(len(rows), 1)
        self.assertEqual(set(rows[0]), {"utc", "scope", "out_dir", "model", "input_tokens", "output_tokens",
                                        "attempts", "exit_code"})
        self.assertEqual((rows[0]["scope"], rows[0]["out_dir"], rows[0]["exit_code"]), ("pack:python", "out", 0))

    def test_repair_then_admitted(self):
        name = "python-mvibe-ratelimit"
        code, out, output = self.run_author(
            self.serve((200, anthropic_reply(BROKEN_PYTHON)), (200, anthropic_reply(PYTHON.rstrip("\n")))),
            extra=["--scope", name, "--scopes-dir", str(SCOPES_DIR)])
        self.assertEqual(code, 0, output)
        self.assertEqual(len(self.seen), 2)
        check1 = json.loads((out / "attempt1-check.json").read_text(encoding="utf-8"))
        feedback = pc.feedback_for_model(check1)
        self.assertIn("seed:failure_state@108", feedback)
        first, second = self.seen[0]["body"], self.seen[1]["body"]
        self.assertEqual(second["system"], first["system"])
        self.assertEqual(second["messages"], [
            first["messages"][0], {"role": "assistant", "content": BROKEN_PYTHON},
            {"role": "user", "content": FROZEN_REPAIR_TEMPLATE.format(feedback=feedback)}])
        for seed in CONTROLS[name]["seeds"]:
            self.assertNotIn(seed["anchor"], second["messages"][2]["content"])
        record = self.author(out)
        self.assertEqual([r["admitted"] for r in record["attempts"]], [False, True])
        self.assertIn("controls", record["attempts"][0]["failed_checks"])
        self.assertEqual(record["attempts"][0]["leniency"], ["whitespace"])  # BROKEN_PYTHON ends with a newline
        self.assertEqual((out / "python.json").read_bytes(), (HERE / "tables" / "python.json").read_bytes())
        self.assertTrue((out / "attempt2-check.json").is_file())
        self.assertEqual([r["exit_code"] for r in self.ledger()], [2, 0])

    def test_repair_openai(self):
        bad = PYTHON.replace('"language": "python",', '"language": "python", "extra": 1,')
        code, out, output = self.run_author(
            self.serve((200, openai_reply(bad)), (200, openai_reply(PYTHON))), style="openai")
        self.assertEqual(code, 0, output)
        messages = self.seen[1]["body"]["messages"]
        self.assertEqual([m["role"] for m in messages], ["system", "user", "assistant", "user"])
        self.assertEqual(messages[0]["content"], GUIDE.read_bytes().decode("utf-8"))
        self.assertEqual(messages[2]["content"], bad)
        self.assertIn("- shape: unknown top-level keys: extra", messages[3]["content"])
        self.assertEqual(self.seen[1]["path"], "/chat/completions")

    def test_rejected_twice(self):
        bad = PYTHON.replace('"not ({text})"', '"not (x)"')
        code, out, output = self.run_author(self.serve((200, anthropic_reply(bad)), (200, anthropic_reply(bad))))
        self.assertEqual(code, 2, output)
        self.assertFalse((out / "python.json").exists())
        for n in (1, 2):
            for suffix in ("request.json", "response.json", "reply.txt", "check.json"):
                self.assertTrue((out / f"attempt{n}-{suffix}").is_file())
        record = self.author(out)
        self.assertEqual((record["admitted"], record["pack_sha256"], record["exit_code"]), (False, None, 2))
        self.assertEqual([r["failed_checks"] for r in record["attempts"]], [["templates"], ["templates"]])
        self.assertEqual([r["exit_code"] for r in self.ledger()], [2, 2])

    def test_cut_off_then_rejected(self):
        code, out, _ = self.run_author(self.serve((200, anthropic_reply(PYTHON[:200], "max_tokens")),
                                                  (200, anthropic_reply(PYTHON, "max_tokens"))))
        self.assertEqual(code, 2)
        record = self.author(out)
        self.assertEqual([(r["cut_off"], r["admitted"]) for r in record["attempts"]], [(True, False), (True, False)])
        self.assertEqual(record["attempts"][0]["failed_checks"][0], "json")
        self.assertFalse((out / "python.json").exists())

    def test_fenced_and_whitespace_replies(self):
        body = PYTHON.rstrip("\n")
        for n, (reply, leniency) in enumerate((("```json\n" + body + "\n```", ["code_fence"]),
                                               ("\n " + body + "\n\n", ["whitespace"]))):
            code, out, output = self.run_author(self.serve((200, anthropic_reply(reply))), name=f"r{n}")
            self.assertEqual(code, 0, output)
            self.assertEqual(self.author(out)["attempts"][0]["leniency"], leniency)
            self.assertEqual((out / "python.json").read_bytes(), (body + "\n").encode("utf-8"))
            self.assertEqual((out / "attempt1-reply.txt").read_text(encoding="utf-8", newline=""), reply)

    def test_dry_run_opens_no_connection(self):
        calls = []
        base = f"http://127.0.0.1:{closed_port()}"
        code, out, output = self.run_author(base, extra=["--dry-run"], opener=lambda *x: calls.append(x))
        self.assertEqual((code, calls), (0, []))
        self.assertIn("dry run", output)
        self.assertEqual(sorted(p.name for p in out.iterdir()), ["attempt1-request.json", "author.json"])
        record = self.author(out)
        self.assertEqual((record["dry_run"], record["attempts"], record["exit_code"]), (True, [], 0))
        self.assertEqual(json.loads((out / "attempt1-request.json").read_text(encoding="utf-8"))["url"],
                         base + "/v1/messages")
        self.assertEqual(self.ledger(), [])
        code, _, _ = self.run_author(base, name="real-opener", extra=["--dry-run"])  # a connection would fail
        self.assertEqual(code, 0)

    def test_refuses_existing_author_json(self):
        out = self.dir / "out"
        out.mkdir()
        (out / "author.json").write_text("{}", encoding="utf-8")
        base = self.serve()
        for extra in ((), ("--dry-run",)):
            code, _, output = self.run_author(base, extra=list(extra))
            self.assertEqual(code, 3)
            self.assertIn("refusing", output)
        self.assertEqual((sorted(p.name for p in out.iterdir()), self.seen), (["author.json"], []))
        self.assertEqual((out / "author.json").read_text(encoding="utf-8"), "{}")

    def test_request_failed(self):
        code, out, _ = self.run_author(self.serve((400, {"error": "bad request"})))
        self.assertEqual((code, len(self.seen), self.sleeps), (4, 1, []))
        record = self.author(out)
        self.assertEqual((record["exit_code"], len(record["attempts"])), (4, 1))
        self.assertIn("400", record["error"])
        self.assertFalse((out / "attempt2-request.json").exists())
        self.assertEqual([r["exit_code"] for r in self.ledger()], [4])
        code, _, _ = self.run_author(self.serve((500, "x"), (200, anthropic_reply(PYTHON))), name="retry")
        self.assertEqual((code, self.sleeps), (0, [10]))

    def test_usage_errors(self):
        base = f"http://127.0.0.1:{closed_port()}"
        scopes = str(SCOPES_DIR)
        cases = [
            (("no-such-pack",), [], "no example pack"),
            (("rust",), ["--scope", "python-mvibe-ratelimit"], "go together"),
            (("rust",), ["--scope", "rust-mmhmem-setup-update", "--scopes-dir", scopes], "is rust, not python"),
            (("rust",), ["--scope", "no-such-scope", "--scopes-dir", scopes], "unknown scope"),
        ]
        for n, (examples, extra, message) in enumerate(cases):
            code, out, output = self.run_author(base, name=f"u{n}", examples=examples, extra=extra)
            self.assertEqual(code, 3, output)
            self.assertIn(message, output)
            self.assertFalse((out / "author.json").exists())
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(buf):
            self.assertEqual(ap.main(["--language", "python"], environ={}), 3)
            self.assertEqual(ap.main(["--language", "python", "--examples", "rust", "--guide", str(GUIDE),
                                      "--out-dir", str(self.dir / "no-env")],
                                     environ={"LESSCODE_MODEL_ENV": str(self.dir / "absent.env")}), 3)


class AnnotateTests(unittest.TestCase):
    def test_annotate_suite_passes(self):
        proc = subprocess.run([sys.executable, str(HERE / "test_annotate.py")], cwd=HERE, capture_output=True,
                              text=True, encoding="utf-8", errors="replace")
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn("OK", proc.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
