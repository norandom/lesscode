"""E4 pure share by statements (plan 6.2 step 2b): `pureshare4.py ROOT --structure S.json --out O`.
t51-integration/pureshare.py reads its file list from E3's module-level `surface.SET` (five files); this shim rebinds that one
name to surface4.SET (the seven lifecycle files) and calls `share` as it is, the way measures4.py rebinds the E3 measurers."""
import json, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE), str(HERE.parent / "t51-integration")]
import pureshare  # noqa: E402  (frozen 5.1 file; only its module-level SET is rebound below)
from surface4 import SET  # noqa: E402
pureshare.SET = SET
if __name__ == "__main__":
    a = sys.argv[1:]
    res = pureshare.share(Path(a[0]).resolve(), json.loads(Path(a[a.index("--structure") + 1]).read_text(encoding="utf-8")))
    Path(a[a.index("--out") + 1]).write_text(json.dumps(res | {"set": SET}, indent=1), encoding="utf-8")
    print(json.dumps({k: v for k, v in res.items() if k != "per_function"}))
