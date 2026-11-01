"""Plot category footprints, without treating test code as a savings target."""

from importlib.metadata import distributions
import json
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

root = Path(__file__).resolve().parent
projects = ["mmhmem", "mvibe"]
data = [json.loads((root / f"{name}-baseline.json").read_text()) for name in projects]
categories = [("production", "#285D8C"), ("tests", "#D18B32"),
              ("support", "#8574A8"), ("fixtures", "#899599")]
fig, ax = plt.subplots(figsize=(9, 3.6))
fig.subplots_adjust(left=0.12, right=0.98, bottom=0.25, top=0.74)
left = [0, 0]
for name, color in categories:
    values = [d["categories"].get(name, {}).get("physical_lines", 0) for d in data]
    ax.barh(projects, values, left=left, label=name.capitalize(), color=color)
    for row, value in enumerate(values):
        if value >= 1500:
            ax.text(left[row] + value / 2, row, f"{value:,}", ha="center", va="center", color="white")
    left = [start + value for start, value in zip(left, values)]
ax.invert_yaxis()
ax.set_xlim(0, 23000)
ax.set_xlabel("Physical lines in declared tracked source scope")
fig.text(0.01, 0.95, "Production and tests have separate footprints", fontsize=14)
ax.legend(ncols=4, loc="upper center", bbox_to_anchor=(0.5, 1.24), frameon=False)
ax.spines[["top", "right", "left"]].set_visible(False)
ax.grid(axis="x", alpha=0.15)
ax.set_axisbelow(True)
fig.text(0.01, 0.01, "Baseline only: no code savings or maintenance benefit demonstrated.", fontsize=9)
matplotlib.rcParams["svg.hashsalt"] = "lesscode-local-adoption-v1"
fig.savefig(root / "footprint.svg", bbox_inches="tight", metadata={"Date": None})
fig.savefig(root / "footprint.png", dpi=160, bbox_inches="tight")
packages = {d.metadata["Name"]: d.version for d in distributions()}
(root / "plot-environment.json").write_text(json.dumps(packages, indent=2, sort_keys=True) + "\n")
