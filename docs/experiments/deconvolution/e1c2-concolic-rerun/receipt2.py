"""E1c-2 receipt: E1c's receipt.py (imported, unchanged) rebound to this directory's results/."""
import sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "e1c-concolic-rung"))
import receipt  # noqa: E402
receipt.HERE, receipt.RECEIPT, receipt.LOGS = HERE, HERE / "results" / "receipt.json", HERE / "results" / "logs"
if __name__ == "__main__":
    receipt.main(sys.argv[1:])
