r"""Slice B drift baseline for pack authoring (preregistered in R66). Research tooling; makes real model calls.

    uv run --no-sync python slice-b\author_baseline.py --out-dir DIR --bin LESSCODE_CORE --mutants-bin PACK_MUTANTS
        [--languages L ...] [--runs 5] [--cap-usd 20] [--workers 4] [--dry-run]

For each language and run: one fresh conversation asks for the pack (system message: the frozen guide; user
message: two shipped packs of other languages and the language name). The reply is checked by the native
`deconvolute admit-pack` with control replay on every gate scope of that language. A rejected pack gets at most
two repair turns that carry only the admission feedback. An admitted pack's mutants are compared with the
shipped pack's on the gate scopes. A refusal or an empty reply ends the run; the request is never reworded.

Spend is tracked across every ledger.jsonl under results/ at an assumed price; no call is made that could pass
the cap. The API key is never printed or written.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
PROTOTYPE = HERE.parent
REPO = PROTOTYPE.parent.parent.parent
sys.path.insert(0, str(PROTOTYPE))

import annotate as a  # noqa: E402
import author_pack as ap  # noqa: E402

PACKS = REPO / "native" / "packs"
FIXTURES = REPO / "native" / "tests" / "fixtures" / "driver"
EXAMPLES = {"rust": ["python", "java"], "python": ["rust", "java"], "typescript": ["rust", "python"],
            "java": ["rust", "python"]}
LIST_USD_PER_MTOK = {"claude-opus-5-5": (4.0, 20.0), "mistral-large-4": (0.7, 2.1)}  # vendor model table, cached 2026-10-06 (R66 Amendment 3)
USD_PER_MTOK = (15.0, 75.0)  # for a model not listed above: the conservative assumption of R66
RESERVE_USD = 0.25  # held back before each call; the gate's authoring calls cost about 0.18
MAX_REPAIRS = 2
MAX_OUTPUT_TOKENS = 4000
TIMEOUT_S = 600
KINDS_TEMPLATE = ("\n\nThese are the named node kinds of the {language} grammar that the pack will be checked "
                  "against, one per line. Every node kind you name in the pack must be one of them.\n\n{kinds}")


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def cost(tokens_in, tokens_out, model=None) -> float:
    rate_in, rate_out = LIST_USD_PER_MTOK.get(model, USD_PER_MTOK)
    return ((tokens_in or 0) * rate_in + (tokens_out or 0) * rate_out) / 1e6


def spent_before(results: Path) -> float:
    total = 0.0
    for ledger in results.rglob("ledger.jsonl"):
        for line in ledger.read_text(encoding="utf-8").splitlines():
            if line.strip():
                row = json.loads(line)
                total += cost(row.get("input_tokens"), row.get("output_tokens"), row.get("model"))
    return total


class Budget:
    def __init__(self, cap: float, spent: float, ledger: Path):
        self.cap, self.spent, self.reserved, self.ledger = cap, spent, 0.0, ledger
        self.lock = threading.Lock()

    def reserve(self) -> bool:
        with self.lock:
            if self.spent + self.reserved + RESERVE_USD > self.cap:
                return False
            self.reserved += RESERVE_USD
            return True

    def settle(self, row: dict) -> None:
        with self.lock:
            self.reserved -= RESERVE_USD
            self.spent += cost(row.get("input_tokens"), row.get("output_tokens"), row.get("model"))
            with self.ledger.open("a", encoding="utf-8", newline="\n") as fh:
                fh.write(json.dumps(row) + "\n")


def scopes_of(language: str) -> list[dict]:
    """The gate scopes of a language with a control file written from the retained reference."""
    found = []
    for path in sorted((FIXTURES / "reference").glob("*.json")):
        ref = json.loads(path.read_text(encoding="utf-8"))
        if ref["language"] == language:
            found.append(ref)
    return found


def write_controls(ref: dict, directory: Path) -> Path:
    cases = [{"label": c["label"], "annotations": c["annotations"], "expected": c["expected"]} for c in ref["controls"]]
    path = directory / f"{ref['scope']}.controls.json"
    path.write_text(json.dumps({"files": ref["files"], "test_paths": ref["test_paths"], "cases": cases}),
                    encoding="utf-8", newline="\n")
    return path


def admit(args, language: str, pack: Path, refs: list[dict], controls: dict, out: Path, tag: str) -> tuple[bool, list[str]]:
    """Admission on every scope of the language: (admitted on all, feedback lines without duplicates)."""
    admitted, feedback = True, []
    sets = [(ref["scope"], controls[ref["scope"]], FIXTURES / "scopes" / ref["repository"]) for ref in refs]
    for extra in sorted((FIXTURES / "controls").glob("*/controls.json")):  # hand-written sets (R66 Amendment 3)
        if json.loads(extra.read_text(encoding="utf-8")).get("language") == language:
            sets.append((f"controls-{extra.parent.name}", extra, extra.parent))
    for name, control_file, root in sets:
        receipt = out / f"{tag}-admission-{name}.json"
        proc = subprocess.run([str(args.bin), "deconvolute", "admit-pack", "--pack", str(pack), "--lang", language,
                               "--controls", str(control_file), "--source-root", str(root), "--out", str(receipt)],
                              capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=300)
        if proc.returncode != 0 or not receipt.is_file():
            admitted = False
        if receipt.is_file():
            for check in json.loads(receipt.read_text(encoding="utf-8"))["checks"]:
                line = f"- {check['name']}: {check['detail']}"
                if not check["pass"] and line not in feedback:
                    feedback.append(line)
        elif proc.returncode != 0:
            feedback.append(f"- admission stopped with exit code {proc.returncode}")
    return admitted, feedback


def mutants(args, pack: Path, ref: dict) -> list | None:
    argv = [str(args.mutants_bin), "--pack", str(pack), "--source-root", str(FIXTURES / "scopes" / ref["repository"])]
    for f in ref["files"]:
        argv += ["--file", f]
    for t in ref["test_paths"]:
        argv += ["--test-path", t]
    proc = subprocess.run(argv, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=300)
    return json.loads(proc.stdout)["mutants"] if proc.returncode == 0 else None


def compare(args, pack: Path, language: str, refs: list[dict]) -> dict:
    per_scope, same = {}, True
    for ref in refs:
        shipped, written = mutants(args, PACKS / f"{language}.json", ref), mutants(args, pack, ref)
        key = lambda m: json.dumps(m, sort_keys=True)  # noqa: E731
        if shipped is None or written is None:
            per_scope[ref["scope"]] = {"identical": False, "error": "mutants could not be listed"}
            same = False
            continue
        a_keys, b_keys = [key(m) for m in shipped], [key(m) for m in written]
        identical = a_keys == b_keys
        same = same and identical
        per_scope[ref["scope"]] = {"identical": identical, "shipped": len(shipped), "written": len(written),
                                   "only_shipped": len(set(a_keys) - set(b_keys)),
                                   "only_written": len(set(b_keys) - set(a_keys))}
    return {"identical": same, "scopes": per_scope}


def one_run(args, cfg: dict, guide: str, language: str, run: int, budget: Budget, base: Path) -> dict:
    out = base / language / f"run{run}"
    out.mkdir(parents=True, exist_ok=True)
    secret = cfg[a.KEY]
    refs = scopes_of(language)
    controls = {ref["scope"]: write_controls(ref, out) for ref in refs}
    names = EXAMPLES[language]
    packs = {name: (PACKS / f"{name}.json").read_bytes() for name in names}
    user = ap.user_message(language, names, packs)
    if args.kinds_dir:  # R76: the grammar's node kinds as additional input
        user += KINDS_TEMPLATE.format(language=language,
                                      kinds=(args.kinds_dir / f"{language}.txt").read_text(encoding="utf-8"))
    record = {"language": language, "run": run, "examples": names, "started_utc": utc_now(),
              "requested_model": cfg[a.NAME], "attempts": [], "outcome": None, "first_reply_admitted": False,
              "admitted": False, "repairs": 0, "pack_sha256": None, "mutants": None}

    def save(name: str, content) -> None:
        text = content if isinstance(content, str) else json.dumps(content, indent=1, ensure_ascii=False) + "\n"
        (out / name).write_text(text.replace(secret, a.REDACTED), encoding="utf-8", newline="\n")

    def finish(outcome: str) -> dict:
        record.update(outcome=outcome, finished_utc=utc_now())
        save("run.json", record)
        return record

    history, message = [], user
    for n in range(1, MAX_REPAIRS + 2):
        if not budget.reserve():
            return finish("not_run_budget" if n == 1 else "stopped_budget")
        url, headers, body = a.build_request(cfg, guide, message, args.max_output_tokens, history)
        save(f"attempt{n}-request.json", {"url": url, "body": body})
        if args.dry_run:
            budget.settle({"utc": utc_now(), "scope": f"pack:{language}", "out_dir": f"{language}/run{run}",
                           "model": cfg[a.NAME], "input_tokens": 0, "output_tokens": 0, "dry_run": True})
            return finish("dry_run")
        call_utc = utc_now()
        sent = a.send(url, headers, body, TIMEOUT_S)
        save(f"attempt{n}-response.json", {"attempts": sent})
        row = {"attempt": n, "started_utc": call_utc, "input_tokens": None, "output_tokens": None,
               "stop_reason": None, "returned_model": None, "admitted": False, "feedback": None,
               "error": sent[-1]["error"]}
        reply = None
        if sent[-1]["error"] is None:
            try:
                result = a.extract(cfg[a.STYLE], sent[-1]["body"])
                reply = result.pop("text")
                if isinstance(reply, list):  # content parts (reasoning and text): keep the text parts
                    reply = "".join(part.get("text", "") for part in reply if isinstance(part, dict) and part.get("type") == "text")
                row.update(result)
            except (ValueError, KeyError, IndexError, TypeError, AttributeError) as exc:
                row["error"] = f"unreadable response envelope: {type(exc).__name__}"
        budget.settle({"utc": call_utc, "scope": f"pack:{language}", "out_dir": f"{language}/run{run}",
                       "model": cfg[a.NAME], "input_tokens": row["input_tokens"],
                       "output_tokens": row["output_tokens"], "attempt": n})
        record["attempts"].append(row)
        if row["error"] is not None:
            return finish("request_failed")
        save(f"attempt{n}-reply.txt", reply)
        if row["stop_reason"] == "refusal" or not reply.strip():
            return finish("refusal" if row["stop_reason"] == "refusal" else "empty_reply")
        text, _ = ap.pack_text(reply)
        pack = out / f"attempt{n}-pack.json"
        pack.write_bytes((text if text.endswith("\n") else text + "\n").replace(secret, a.REDACTED).encode("utf-8"))
        admitted, feedback = admit(args, language, pack, refs, controls, out, f"attempt{n}")
        if row["stop_reason"] == a.CUT_OFF[cfg[a.STYLE]]:
            admitted, feedback = False, ["- the reply was cut off before the pack was complete", *feedback]
        row.update(admitted=admitted, feedback=feedback)
        if admitted:
            record.update(admitted=True, first_reply_admitted=n == 1, repairs=n - 1,
                          pack_sha256=a.sha(pack.read_bytes()), mutants=compare(args, pack, language, refs))
            return finish("admitted")
        history += [{"role": "user", "content": message}, {"role": "assistant", "content": reply}]
        message = ap.REPAIR_TEMPLATE.format(feedback="\n".join(feedback))
    record["repairs"] = MAX_REPAIRS
    return finish("rejected_after_repairs")


def summarise(records: list[dict]) -> dict:
    summary = {}
    for language in sorted({r["language"] for r in records}):
        rows = [r for r in records if r["language"] == language]
        admitted = [r for r in rows if r["admitted"]]
        tokens_in = sum((t["input_tokens"] or 0) for r in rows for t in r["attempts"])
        tokens_out = sum((t["output_tokens"] or 0) for r in rows for t in r["attempts"])
        summary[language] = {
            "runs": len(rows), "outcomes": {o: sum(r["outcome"] == o for r in rows) for o in sorted({r["outcome"] for r in rows})},
            "first_reply_admitted": sum(r["first_reply_admitted"] for r in rows),
            "admitted": len(admitted), "repair_replies": sum(max(0, len(r["attempts"]) - 1) for r in rows),
            "same_mutants": sum(bool(r["mutants"] and r["mutants"]["identical"]) for r in admitted),
            "distinct_admitted_packs": len({r["pack_sha256"] for r in admitted}),
            "failed_checks_first_reply": sorted({line.split(":")[0].lstrip("- ") for r in rows if r["attempts"]
                                                 for line in (r["attempts"][0]["feedback"] or [])}),
            "input_tokens": tokens_in, "output_tokens": tokens_out,
            "usd": round(cost(tokens_in, tokens_out, rows[0]["requested_model"]), 2),
        }
    return summary


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--bin", type=Path, required=True)
    parser.add_argument("--mutants-bin", type=Path, required=True)
    parser.add_argument("--guide", type=Path, default=PROTOTYPE / "pack-guide-v3.md")
    parser.add_argument("--languages", nargs="+", default=list(EXAMPLES))
    parser.add_argument("--runs", type=int, default=5)
    parser.add_argument("--cap-usd", type=float, default=20.0)
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--max-output-tokens", type=int, default=MAX_OUTPUT_TOKENS)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--endpoint-env", type=Path)
    parser.add_argument("--kinds-dir", type=Path, help="directory with LANGUAGE.txt node kind lists (R76)")
    args = parser.parse_args()
    if (args.out_dir / "summary.json").exists():
        print("refusing to overwrite an existing run", file=sys.stderr)
        return 3
    import os
    cfg, error = a.load_config(a.env_path(args.endpoint_env, os.environ))
    if error:
        print(error, file=sys.stderr)
        return 3
    guide_bytes = args.guide.read_bytes()
    args.out_dir.mkdir(parents=True, exist_ok=True)
    spent = spent_before(PROTOTYPE / "results")
    budget = Budget(args.cap_usd, spent, args.out_dir / "ledger.jsonl")
    started = time.monotonic()
    jobs = [(language, run) for run in range(1, args.runs + 1) for language in args.languages]
    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        records = list(pool.map(lambda job: one_run(args, cfg, guide_bytes.decode("utf-8"), job[0], job[1], budget,
                                                    args.out_dir), jobs))
    summary = {
        "receipt": "slice-b-pack-authoring-baseline-v1", "preregistration": "R66", "dry_run": args.dry_run,
        "requested_model": cfg[a.NAME], "guide_sha256": a.sha(guide_bytes),
        "shipped_packs_sha256": {p.stem: a.sha(p.read_bytes()) for p in sorted(PACKS.glob("*.json"))},
        "node_kinds_sha256": ({p.stem: a.sha(p.read_bytes()) for p in sorted(args.kinds_dir.glob("*.txt"))}
                              if args.kinds_dir else None),
        "runs_per_language": args.runs, "max_output_tokens": args.max_output_tokens, "max_repairs": MAX_REPAIRS, "usd_per_mtok_list": LIST_USD_PER_MTOK, "usd_per_mtok_otherwise": USD_PER_MTOK,
        "spent_before_usd": round(spent, 2), "spent_after_usd": round(budget.spent, 2),
        "cap_usd": args.cap_usd, "wall_seconds": round(time.monotonic() - started, 1),
        "languages": summarise(records),
    }
    (args.out_dir / "summary.json").write_text(json.dumps(summary, indent=1) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(summary["languages"], indent=1))
    print(f"spent before {summary['spent_before_usd']} USD, after {summary['spent_after_usd']} USD "
          f"(assumed price), cap {args.cap_usd}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
