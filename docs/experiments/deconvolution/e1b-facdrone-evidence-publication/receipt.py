"""E2's receipt.py re-pointed at results/receipt.json here: `run LABEL [--cwd D] [--env K=V] -- CMD`;
`facts VENV_PYTHON` also records the E1b data files (.md, .toml, patches) and the scope record."""
import importlib.util, json, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("e2_receipt", HERE.parent / "e2-facdrone-edge-census/receipt.py")
e2 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(e2)
e2.HERE, e2.RECEIPT = HERE, HERE / "results" / "receipt.json"

if __name__ == "__main__":
    e2.main(sys.argv[1:])
    if sys.argv[1] == "facts":
        import scope
        data = json.loads(e2.RECEIPT.read_text())
        extra = [p for p in HERE.glob("*") if p.suffix in (".md", ".toml")] + sorted(HERE.glob("mutants/*.patch"))
        data["facts"][-1].update(scope=scope.record(), e2_receipt_sha256=e2.sha(HERE.parent / "e2-facdrone-edge-census/receipt.py"),
                                 data_sha256={str(p.relative_to(HERE)): e2.sha(p) for p in extra})
        e2.RECEIPT.write_text(json.dumps(data, indent=1))
