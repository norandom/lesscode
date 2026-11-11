"""Finish annotation runs whose reply was received and saved but could not be read at the time.

On 2026-11-11 `annotate.py` could not read a reply that arrives as a list of parts (reasoning and text), as
`mistral-large-4` sends it. The calls were made and their responses saved; nothing after that was written,
including the ledger line. This finishes those runs from the saved `response.json` with the corrected
reader. It makes no model call.

    uv run python slice-b/salvage_annotations.py results/slice-b/annot-mistral
"""

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

import annotate as a  # noqa: E402


def main(arm: Path) -> int:
    done = 0
    for directory in sorted(p for p in arm.iterdir() if p.is_dir()):
        call_path, response_path = directory / "call.json", directory / "response.json"
        call = json.loads(call_path.read_text(encoding="utf-8"))
        if call.get("exit_code") is not None or not response_path.exists():
            continue
        attempts = json.loads(response_path.read_text(encoding="utf-8"))["attempts"]
        last = attempts[-1]
        code = 4
        call["error"] = last["error"]
        if last["error"] is None:
            result = a.extract(call["api_style"], last["body"])
            text = result.pop("text")
            (directory / "reply.txt").write_text(text, encoding="utf-8", newline="\n")
            annotations, leniency = a.parse_reply(text)
            if annotations is not None:
                (directory / "annotations.json").write_text(json.dumps(annotations, indent=1, ensure_ascii=False) + "\n",
                                                            encoding="utf-8", newline="\n")
            cut_off = result["stop_reason"] == a.CUT_OFF[call["api_style"]]
            code = 0 if annotations is not None and not cut_off else 2
            call.update(result, parse="ok" if annotations is not None else "parse_failed", leniency=leniency,
                        annotations=None if annotations is None else len(annotations))
        call.update(attempts=len(attempts), exit_code=code, salvaged="2026-11-11: read from the saved response")
        call_path.write_text(json.dumps(call, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
        ledger = {"utc": call.get("started_utc"), "scope": call["scope"], "out_dir": directory.name,
                  "model": call["requested_model"], "input_tokens": call.get("input_tokens"),
                  "output_tokens": call.get("output_tokens"), "attempts": len(attempts), "exit_code": code, "salvaged": True}
        with (arm / "ledger.jsonl").open("a", encoding="utf-8", newline="\n") as fh:
            fh.write(json.dumps(ledger) + "\n")
        print(f"{directory.name}: exit {code}, parse {call.get('parse')}, {call.get('annotations')} annotations, "
              f"{call.get('input_tokens')} in / {call.get('output_tokens')} out")
        done += 1
    print(f"{done} runs finished from saved responses")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(Path(sys.argv[1])))
