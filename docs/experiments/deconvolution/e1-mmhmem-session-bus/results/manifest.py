"""Per-file SHA-256 of src/ and tests/ (plus Cargo.toml, Cargo.lock) in a copy.

Usage: py -I manifest.py <copy_root> <out.json> [git_repo commit]
With git_repo/commit, also records the git blob id for each path and whether
the copy's bytes hash to that blob (git hash-object semantics).
"""
import hashlib
import json
import os
import subprocess
import sys

root, out = sys.argv[1], sys.argv[2]
blobs = {}
if len(sys.argv) > 4:
    repo, commit = sys.argv[3], sys.argv[4]
    ls = subprocess.run(["git", "ls-tree", "-r", commit, "--", "src", "tests", "Cargo.toml", "Cargo.lock"],
                        cwd=repo, capture_output=True, text=True, check=True).stdout
    for line in ls.splitlines():
        meta, path = line.split("\t", 1)
        blobs[path] = meta.split()[2]

files = {}
paths = []
for top in ("src", "tests"):
    for d, _, fs in os.walk(os.path.join(root, top)):
        for f in fs:
            paths.append(os.path.relpath(os.path.join(d, f), root).replace("\\", "/"))
paths += ["Cargo.toml", "Cargo.lock"]
for p in sorted(paths):
    data = open(os.path.join(root, p), "rb").read()
    rec = {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
    if blobs:
        git_blob = hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()
        rec["git_blob"] = blobs.get(p)
        rec["matches_git_blob"] = git_blob == blobs.get(p)
        lf = data.replace(b"\r\n", b"\n")
        rec["matches_git_blob_crlf_to_lf"] = hashlib.sha1(b"blob %d\0" % len(lf) + lf).hexdigest() == blobs.get(p)
    files[p] = rec
summary = {"root": root, "file_count": len(files),
           "all_match_git": all(v.get("matches_git_blob", True) for v in files.values()),
           "all_match_git_after_crlf_to_lf": all(v.get("matches_git_blob", True) or v.get("matches_git_blob_crlf_to_lf", True) for v in files.values()),
           "note": "git archive applies core.autocrlf=true (specimen config), so text files are CRLF in the copy; LF-normalised bytes must equal the git blob",
           "files_not_in_git": [p for p in files if blobs and p not in blobs],
           "git_files_missing": [p for p in blobs if p not in files],
           "files": files}
json.dump(summary, open(out, "w", encoding="utf-8"), indent=1)
print(out, summary["file_count"], "all_match_git=", summary["all_match_git"], "after_crlf_norm=", summary["all_match_git_after_crlf_to_lf"],
      "extra=", summary["files_not_in_git"], "missing=", summary["git_files_missing"])
