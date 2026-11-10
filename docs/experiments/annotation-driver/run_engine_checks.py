r"""Gate checklist step 3: model-free checks of the engine, the packs and the harness.

    uv run --no-sync python run_engine_checks.py --scopes-dir D:\BuildArtifacts\lesscode\annotation-driver\scopes --out-dir D:\BuildArtifacts\lesscode\annotation-driver\engine-checks

a  every node kind a pack names is valid for its grammar; packs at most 40 lines
b  ast-grep positions: character offsets and columns, a CRLF pair is one line
   break, sites do not depend on line endings or on non-ASCII text before them
c  each control seed with an operator yields a changed mutant that differs only
   inside its statement; the same edit check for every eligible site
d  eligible-site counts per scope and kind, by the R59 method and normalised
e  select is deterministic and kind-balanced
f  harness smoke test per scope: identical, unparseable and real-change mutants

Writes OUT-DIR/engine-checks-receipt.json and the smoke logs. Exit code 0 only
if every check holds.
"""

from __future__ import annotations

import argparse
import bisect
import hashlib
import json
import time
from collections import Counter
from dataclasses import asdict
from pathlib import Path

from ast_grep_py import SgRoot

import engine as e
import harness as h
import validator as v

HERE = Path(__file__).parent
R59_COUNTS = {
    "rust-mmhmem-setup-update": {"failure_state": 19, "branch_condition": 16, "error_order": 2, "state_write": 1},
    "python-mvibe-ratelimit": {"state_write": 111, "failure_state": 44, "branch_condition": 40, "error_order": 5},
    "typescript-openultracode-state": {"failure_state": 29, "branch_condition": 17, "state_write": 5},
    "java-commons-csv-lexer": {"state_write": 52, "branch_condition": 51, "failure_state": 43, "error_order": 11},
}
# Normalised sites with the pack's exclude_kinds: the Rust pack drops 4 let_condition
# and 1 let_chain conditions; parenthesised conditions map one to one onto their inner
# expression. The TypeScript pack excludes the empty condition of one `for (;;)` loop.
ENGINE_COUNTS = {**R59_COUNTS,
                 "rust-mmhmem-setup-update":
                 {"failure_state": 19, "branch_condition": 11, "error_order": 2, "state_write": 1},
                 "typescript-openultracode-state":
                 {"failure_state": 29, "branch_condition": 16, "state_write": 5}}
SAMPLE = " \u00e9\u20ac\U0001F600e\u0301"  # Latin-1, BMP, astral and combining characters


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def kind_counts(sites: list[e.Site]) -> dict[str, int]:
    return dict(sorted(Counter(s.kind for s in sites).items()))


def pack_kinds(pack: dict) -> set[str]:
    kinds = {*pack["comment_kinds"], *pack["string_kinds"], *pack["unwrap_kinds"], *pack["statement_wrappers"]}
    for rule in pack["kinds"].values():
        kinds |= {*rule.get("nodes", ()), *rule.get("nodes_with_prefix", {}), *(k for k, _ in rule.get("field_of", ()))}
    ops = pack["operators"]
    return kinds | {*ops["branch_condition"].get("exclude_kinds", ()), *ops["failure_state"]["by_kind"]}


def kind_is_valid(root, kind: str) -> bool:
    try:
        root.find(kind=kind)
        return True
    except RuntimeError:
        return False


def check_packs() -> list[dict]:
    rows = []
    for path in sorted((HERE / "tables").glob("*.json")):
        pack = json.loads(path.read_text(encoding="utf-8"))
        root = SgRoot("", pack["language"]).root()
        kinds = sorted(pack_kinds(pack))
        invalid = [k for k in kinds if not kind_is_valid(root, k)]
        control = not kind_is_valid(root, "no_such_kind_control")
        lines = len(path.read_bytes().splitlines())
        rows.append({"pack": path.name, "kinds_checked": len(kinds), "invalid_kinds": invalid,
                     "invalid_kind_control_raises": control, "lines": lines,
                     "pass": not invalid and control and lines <= 40})
    return rows


def position_errors(text: str, language: str) -> tuple[int, int]:
    """(nodes, mismatches) between ast-grep positions and positions recomputed from the text."""
    newlines = [i for i, c in enumerate(text) if c == "\n"]
    nodes = bad = 0
    for node in v._walk(SgRoot(text, language).root()):
        nodes += 1
        r = node.range()
        bad += text[r.start.index:r.end.index] != node.text()
        for pos in (r.start, r.end):
            line = bisect.bisect_left(newlines, pos.index)
            line_start = newlines[line - 1] + 1 if line else 0
            bad += (line, pos.index - line_start) != (pos.line, pos.column)
    return nodes, bad


def site_keys(sites: list[e.Site], shift: int = 0, crlf: bool = False) -> list[tuple]:
    return [(s.kind, s.node_kind, s.start_line + shift, s.start_column, s.end_line + shift, s.end_column,
             s.text.replace("\n", "\r\n") if crlf else s.text) for s in sites]


def check_positions(scopes: dict, controls: dict) -> list[dict]:
    rows = []
    for name, spec in controls.items():
        scope = scopes[name]
        table = v.load_table(scope.language)
        for file in scope.files:
            raw = e.read_text(scope, file)
            lf = raw.replace("\r\n", "\n")
            marked = spec["comment_text"] + SAMPLE + "\r\n" + raw
            variants = {label: position_errors(text, scope.language)
                        for label, text in (("disk", raw), ("lf", lf), ("non_ascii_prefix", marked))}
            disk = e.file_sites(table, scope.language, file, raw)
            same_lf = site_keys(disk) == site_keys(e.file_sites(table, scope.language, file, lf), crlf=True)
            same_marked = site_keys(disk, shift=1) == site_keys(e.file_sites(table, scope.language, file, marked))
            rows.append({"scope": name, "file": file, "crlf_pairs": raw.count("\r\n"),
                         "lone_cr": raw.count("\r") - raw.count("\r\n"),
                         "non_ascii_chars": sum(ord(c) > 127 for c in raw),
                         "nodes_and_mismatches": variants, "sites_same_with_lf": same_lf,
                         "sites_same_after_non_ascii_line": same_marked,
                         "pass": all(b == 0 for _, b in variants.values()) and same_lf and same_marked})
    return rows


def offset(text: str, line: int, column: int) -> int:
    start = 0
    for _ in range(line - 1):
        start = text.index("\n", start) + 1
    return start + column


def edit_check(scope: v.Scope, site: e.Site, mutant: e.Mutant) -> dict:
    """Is the mutant changed, and does it differ from the original only inside its statement(s)?"""
    original = e.read_text(scope, site.file)
    mutated = mutant.text
    lo, hi = offset(original, site.start_line, site.start_column), offset(original, site.end_line, site.end_column)
    p = 0
    while p < min(len(original), len(mutated)) and original[p] == mutated[p]:
        p += 1
    s = 0
    while s < min(len(original), len(mutated)) - p and original[-1 - s] == mutated[-1 - s]:
        s += 1
    (a0, a1), (b0, b1) = mutant.spans[0], mutant.spans[-1]
    inside = a0 <= lo and hi <= a1 and a0 <= p and len(original) - s <= b1
    if len(mutant.spans) == 2:  # a swap: the text between the two statements is unchanged
        swapped = original[:a0] + original[b0:b1] + original[a1:b0] + original[a0:a1] + original[b1:]
        inside = inside and mutated == swapped
    return {"verdict": v.classify_mutant(scope.language, original, mutated),
            "site_text_at_offsets": original[lo:hi] == site.text, "inside_statement": inside,
            "before": original[a0:b1], "after": mutated[a0:b1 + len(mutated) - len(original)]}


def check_seeds(scopes: dict, controls: dict) -> list[dict]:
    rows = []
    for name, spec in controls.items():
        scope = scopes[name]
        eligible = set(e.eligible_sites(scope))
        for seed in spec["seeds"]:
            accepted = v.validate(scope, [seed])[0]["accepted"]
            site = e.site_from_annotation(scope, seed)
            mutant = e.mutate(scope, site) if site is not None else None
            row = {"scope": name, "seed": f"{seed['kind']}@{seed['start_line']}", "accepted": accepted,
                   "site": asdict(site) if site else None, "site_is_eligible": site in eligible,
                   "operator": mutant.operator if mutant else None}
            if mutant is None:
                row.update(note="no_operator: not mutated")
                row["pass"] = accepted and site in eligible
            else:
                row.update(edit_check(scope, site, mutant))
                row["pass"] = accepted and site in eligible and row["verdict"] == "changed" \
                    and row["site_text_at_offsets"] and row["inside_statement"]
            rows.append(row)
    return rows


def check_all_sites(scopes: dict) -> list[dict]:
    """The edit check for every eligible site; verdicts and operator coverage are reported."""
    rows = []
    for name, scope in scopes.items():
        coverage: dict[str, Counter] = {}
        verdicts, failures = Counter(), []
        for site in e.eligible_sites(scope):
            mutant = e.mutate(scope, site)
            cell = coverage.setdefault(site.kind, Counter())
            if mutant is None:
                cell[f"no_operator:{site.node_kind}"] += 1
                continue
            cell[f"operator:{mutant.operator}"] += 1
            result = edit_check(scope, site, mutant)
            verdicts[result["verdict"]] += 1
            if not (result["site_text_at_offsets"] and result["inside_statement"]):
                failures.append({**asdict(site), **result})
            if result["verdict"] != "changed":
                cell[f"verdict_{result['verdict']}"] += 1
        rows.append({"scope": name, "coverage": {k: dict(c) for k, c in sorted(coverage.items())},
                     "verdicts": dict(verdicts), "edit_failures": failures, "pass": not failures})
    return rows


def r59_table(table: dict) -> dict:
    """The pack as R59 counted with it: no parenthesis wrappers, no excluded conditions."""
    ops = {**table["operators"], "branch_condition": {**table["operators"]["branch_condition"], "exclude_kinds": []}}
    return {**table, "unwrap_kinds": [], "operators": ops}


def check_counts(scopes: dict) -> list[dict]:
    rows = []
    for name, scope in scopes.items():
        table = r59_table(v.load_table(scope.language))
        old = [s for f in scope.files for s in e.file_sites(table, scope.language, f, e.read_text(scope, f))]
        new = e.eligible_sites(scope)
        old_keys, new_keys = set(site_keys(old)), set(site_keys(new))
        only_old = Counter(f"{s.kind}:{s.node_kind}" for s, k in zip(old, site_keys(old)) if k not in new_keys)
        only_new = Counter(f"{s.kind}:{s.node_kind}" for s, k in zip(new, site_keys(new)) if k not in old_keys)
        got_old, got_new = kind_counts(old), kind_counts(new)
        rows.append({"scope": name, "r59": R59_COUNTS[name], "r59_method": got_old, "r59_total": len(old),
                     "expected_engine": ENGINE_COUNTS[name], "engine": got_new, "engine_total": len(new),
                     "only_r59_method": dict(only_old), "only_engine": dict(only_new),
                     "pass": got_old == R59_COUNTS[name] and got_new == ENGINE_COUNTS[name]})
    return rows


def balanced(chosen: list[e.Site], available: Counter) -> bool:
    n = Counter(s.kind for s in chosen)
    return all(n[j] == available[j] or n[k] <= n[j] + 1 for j in available for k in available)


def check_select(scopes: dict) -> list[dict]:
    rows = []
    for name, scope in scopes.items():
        sites = e.eligible_sites(scope)
        chosen = e.select(sites)
        available = Counter(s.kind for s in sites)
        deterministic = chosen == e.select(list(reversed(sites))) == e.select(sites)
        prefixes = all(balanced(chosen[:i], available) for i in range(1, len(chosen) + 1))
        size = len(chosen) == min(40, len(sites)) and len(set(chosen)) == len(chosen) and set(chosen) <= set(sites)
        with_operator = sum(e.mutate(scope, s) is not None for s in chosen)
        rows.append({"scope": name, "selected": len(chosen), "by_kind": kind_counts(chosen),
                     "selected_with_operator": with_operator, "deterministic": deterministic,
                     "every_prefix_balanced": prefixes, "size_and_membership": size,
                     "other_seed_differs": e.select(sites, seed=1) != chosen,
                     "pass": deterministic and prefixes and size})
    return rows


def summary(check: dict | None) -> dict | None:
    if check is None:
        return None
    return {"class": check["class"], "reason": check["reason"],
            "runs": [{k: r[k] for k in ("step", "exit_code", "timed_out", "seconds", "log")} for r in check["runs"]]}


def smoke(scopes: dict, controls: dict, scopes_dir: Path, out_dir: Path) -> list[dict]:
    rows = []
    for name, spec in controls.items():
        scope = scopes[name]
        seed = spec["seeds"][0]
        path = scope.root / seed["file"]
        raw, (old, new) = e.read_text(scope, seed["file"]), spec["real_change"]
        site = e.site_from_annotation(scope, seed)
        mutants = [e.Mutant(site, "control:identical", raw, ()),
                   e.Mutant(site, "control:unparseable", raw.replace(old, old + " ((((", 1), ()),
                   e.Mutant(site, "control:real_change", raw.replace(old, new, 1), ())]
        before = sha256_file(path)
        started = time.monotonic()
        batch = h.run_batch(scope, h.scope_config(name), scopes_dir, mutants, out_dir / "smoke" / name)
        seconds = round(time.monotonic() - started, 1)
        got = [(m["class"], m["reason"]) for m in batch["mutants"]]
        restored = before == sha256_file(path) == batch["scope_files_sha256"][seed["file"]]
        ok = batch["void"] is None and restored and len(got) == 3 and got[0] == ("invalid", "no_op") \
            and got[1] == ("invalid", "parse_error") and got[2][0] in ("killed", "survived")
        rows.append({"scope": name, "seconds": seconds, "void": batch["void"], "file_restored": restored,
                     "classes": got, "real_change_class": got[2][0] if len(got) == 3 else None,
                     "real_change_seconds": batch["mutants"][2]["seconds"] if len(got) == 3 else None,
                     "baseline_before": summary(batch["baseline_before"]),
                     "baseline_after": summary(batch["baseline_after"]), "pass": ok})
        print(f"f {name}: {got} void={batch['void']} restored={restored} {seconds}s", flush=True)
    return rows


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--scopes-dir", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    args = parser.parse_args()
    scopes = v.load_scopes(args.scopes_dir)
    controls = json.loads((HERE / "controls.json").read_text(encoding="utf-8"))["scopes"]
    checks = {
        "a_packs": check_packs(),
        "b_positions": check_positions(scopes, controls),
        "c_seeds": check_seeds(scopes, controls),
        "c_all_sites": check_all_sites(scopes),
        "d_counts": check_counts(scopes),
        "e_select": check_select(scopes),
    }
    for label, rows in checks.items():
        print(f"{label}: {sum(r['pass'] for r in rows)} of {len(rows)} pass", flush=True)
    checks["f_smoke"] = smoke(scopes, controls, args.scopes_dir, args.out_dir)
    passed = {label: all(r["pass"] for r in rows) for label, rows in checks.items()}
    inputs = ["engine.py", "harness.py", "validator.py", "run_arm.py", "run_engine_checks.py", "run_controls.py",
              "controls.json", "scopes.json", "compose.yaml", *sorted(f"tables/{p.name}" for p in (HERE / "tables").glob("*.json"))]
    receipt = {"receipt": "annotation-driver-engine-checks-v0", "passed": all(passed.values()),
               "checks_passed": passed, "inputs_sha256": {n: sha256_file(HERE / n) for n in inputs}, **checks}
    args.out_dir.mkdir(parents=True, exist_ok=True)
    out = args.out_dir / "engine-checks-receipt.json"
    out.write_text(json.dumps(receipt, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"checks passed: {passed}\nall: {receipt['passed']} -> {out}")
    return 0 if receipt["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
