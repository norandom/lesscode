"""Edit-scope check (deconvolution-loop req 6.4, 6.5), task 5.1.
`editscope.py ORIGINAL CANDIDATE --scope F [F ...] --out O [--dirs src tests packages] [--mentions FILE ...]`
Hashes (sha256) every file directly in the repository root and every file under the given top-level directories
of both trees, skipping __pycache__, .pytest_cache, .git and *.pyc, and reports files only in one tree, files whose
content differs, and the differing set against the scope. Both trees are read only.
  edit_scope  "pass" iff nothing was added or removed and nothing outside the scope differs (req 6.4, 6.5)
  verdict     "pass" iff edit_scope passes and the differing set is exactly the scope (every scope file changed)
--mentions scans text files (the regenerator's change log, response) for *.py tokens outside the scope."""
import hashlib, json, re, sys
from pathlib import Path

SKIP_DIRS, SKIP_SUFFIX, DIRS = {"__pycache__", ".pytest_cache", ".git"}, {".pyc"}, ("src", "tests", "packages")
PY_TOKEN = re.compile(r"[\w./\\-]*\w\.py\b")


def digest(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def walk(root: Path, dirs=DIRS) -> dict[str, str]:
    out = {p.name: digest(p) for p in root.iterdir() if p.is_file() and p.suffix not in SKIP_SUFFIX}
    for d in dirs:
        for p in sorted((root / d).rglob("*")) if (root / d).is_dir() else []:
            rel = p.relative_to(root)
            if p.is_file() and p.suffix not in SKIP_SUFFIX and not SKIP_DIRS & set(rel.parts):
                out[rel.as_posix()] = digest(p)
    return out


def check(original: Path, candidate: Path, scope, dirs=DIRS) -> dict:
    a, b, scope = walk(original, dirs), walk(candidate, dirs), sorted(scope)
    common = a.keys() & b.keys()
    differing = sorted(k for k in common if a[k] != b[k])
    only_o, only_c = sorted(a.keys() - b.keys()), sorted(b.keys() - a.keys())
    outside = sorted(set(differing) - set(scope))
    edit_scope = "pass" if not only_o and not only_c and not outside else "fail"
    return {"original": str(original), "candidate": str(candidate), "dirs": list(dirs), "skip_dirs": sorted(SKIP_DIRS),
            "skip_suffix": sorted(SKIP_SUFFIX), "files_original": len(a), "files_candidate": len(b),
            "files_compared": len(common), "only_in_original": only_o, "only_in_candidate": only_c,
            "differing": differing, "scope": scope, "differing_outside_scope": outside,
            "scope_unchanged": sorted(set(scope) - set(differing)), "scope_missing": sorted(set(scope) - common),
            "original_hashes": {k: a.get(k) for k in scope}, "candidate_hashes": {k: b.get(k) for k in scope},
            "edit_scope": edit_scope, "verdict": "pass" if edit_scope == "pass" and differing == scope else "fail"}


def mentions(files, scope) -> list[str]:
    names = {Path(s).name for s in scope} | set(scope)
    found = {t for f in files for t in PY_TOKEN.findall(Path(f).read_text(encoding="utf-8", errors="replace"))}
    return sorted(t for t in found if t not in names and Path(t).name not in names)


if __name__ == "__main__":
    a = sys.argv[1:]
    if len(a) < 2 or "--scope" not in a:
        sys.exit(print(__doc__) or 2)
    opts, key = {}, None  # "--k v1 v2" -> {"--k": [v1, v2]}
    for x in a[2:]:
        key, _ = (x, opts.setdefault(x, [])) if x.startswith("--") else (key, opts[key].append(x))
    res = check(Path(a[0]).resolve(), Path(a[1]).resolve(), opts["--scope"], tuple(opts.get("--dirs") or DIRS))
    if "--mentions" in opts:
        res["mentions"] = {"files": opts["--mentions"], "outside_scope": mentions(opts["--mentions"], opts["--scope"])}
    Path(opts["--out"][0]).write_text(json.dumps(res, indent=1), encoding="utf-8")
    print(json.dumps({k: res[k] for k in ("files_compared", "only_in_original", "only_in_candidate", "differing",
                                           "differing_outside_scope", "scope_unchanged", "edit_scope", "verdict")}))
    sys.exit(0 if res["verdict"] == "pass" else 1)
