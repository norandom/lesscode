"""E1b frozen scope: the R15 case A files, the entry callables, their hashes (`scope.py [ROOT]`
prints the record as JSON; exit 1 if a production file differs from the R15 manifest)."""
import hashlib, json, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
LESSCODE = HERE.parents[3]
FROZEN = LESSCODE / ".runs/e2-facdrone/source"
RUN02 = LESSCODE / "docs/experiments/constraint-modeler/facdrone-discovery/run-02"
CASE = RUN02 / "case-evidence-publication.json"
MANIFEST = RUN02 / "source-manifest.json"
R16_CHECK = LESSCODE / "docs/experiments/constraint-modeler/evidence-publication/check.py"

FILES = (  # order as in the case bundle; asserted against it below
    "src/facdrone/shells/decision_evidence_publication.py",
    "src/facdrone/shells/decision_evidence_store.py",
    "src/facdrone/shells/decision_evidence_reader.py",
    "src/facdrone/shells/decision_input_slice_store.py",
    "src/facdrone/shells/decision_slice_publisher.py",
    "src/facdrone/shells/cppi/state_repository.py",
    "src/facdrone/shells/wiring/run_evidence_store.py",
    "packages/facdrone-kernel/facdrone_kernel/domain/decision_evidence.py",
)
MUTATED = FILES[0], FILES[5]  # the cosmic-ray sample is drawn from these two only

# Entry callables named in the plan, as module:qualname (the oracle drives these).
_E = {
    "decision_evidence_publication": "seal_evidence seal_evidence_with_run_slices bind_evidence "
    "bind_evidence_with_run_slices compact_capture payload_batches stored_evidence_payload stored_bound_payload",
    "cppi.state_repository": "SqlLiveStateRepository.bootstrap SqlLiveStateRepository.commit _publish_evidence",
    "wiring.run_evidence_store": "persist_run delete_run_evidence",
    "decision_slice_publisher": "publish_slices require_referenced_slices",
    "decision_input_slice_store": "append_slice SliceResolver.resolve",
    "decision_evidence_reader": "read_resolved_evidence",
}
ENTRIES = tuple(f"facdrone.shells.{m}:{n}" for m, names in _E.items() for n in names.split())

sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()


def record(root: Path = FROZEN) -> dict:
    case = json.loads(CASE.read_text(encoding="utf-8"))
    assert tuple(f["path"] for f in case["files"]) == FILES, "scope differs from the case bundle"
    manifest = {f["path"]: f["sha256"] for f in json.loads(MANIFEST.read_text(encoding="utf-8"))["files"]}
    files = {p: {"sha256": sha(root / p), "manifest_sha256": manifest[p],
                 "lines": len((root / p).read_text(encoding="utf-8").splitlines())} for p in FILES}
    changed = [p for p, v in files.items() if v["sha256"] != v["manifest_sha256"]]
    return {"root": str(root), "case_sha256": sha(CASE), "manifest_sha256": sha(MANIFEST),
            "r16_check_sha256": sha(R16_CHECK), "files": files, "entries": ENTRIES,
            "changed": changed, "physical_lines": sum(v["lines"] for v in files.values())}



def changed_lines(original: str, mutated: str) -> list[int]:
    """Amendment 7: original lines a mutant changes, widened to the enclosing statement header
    (coverage records a multi-line statement under its own line numbers, not every physical line)."""
    import ast, difflib
    a, b = original.splitlines(), mutated.splitlines()
    hit = set()
    for tag, i1, i2, _, _ in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes():
        if tag != "equal":  # a pure insertion touches its neighbours
            hit |= set(range(i1 + 1, i2 + 1)) if i2 > i1 else {max(i1, 1), i1 + 1}
    spans = [(n.lineno, (n.body[0].lineno - 1 if getattr(n, "body", None) and isinstance(n.body, list)
                         and n.body[0].lineno > n.lineno else n.end_lineno))
             for n in ast.walk(ast.parse(original)) if isinstance(n, ast.stmt)]
    out = set(hit)
    for line in hit:
        inner = [s for s in spans if s[0] <= line <= s[1]]
        if inner:
            lo, hi = min(inner, key=lambda s: s[1] - s[0])
            out |= set(range(lo, hi + 1))
    return sorted(out)


if __name__ == "__main__":
    rec = record(Path(sys.argv[1]) if len(sys.argv) > 1 else FROZEN)
    print(json.dumps(rec, indent=1))
    sys.exit(1 if rec["changed"] else 0)
