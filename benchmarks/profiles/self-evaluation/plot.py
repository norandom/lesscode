"""Render the published self-evaluation receipt; no analysis or model calls."""

import json
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt


ROOT = Path(__file__).resolve().parents[3]
receipt = ROOT / "docs/evidence/self-evaluation-2026-10-31.json"
evidence = json.loads(receipt.read_text(encoding="utf-8"))
package_metrics = evidence["whole_package"]["metrics"]
auxiliary_sides = evidence["auxiliary"]["sides"]
output = ROOT / "docs/images"
output.mkdir(exist_ok=True)
plt.rcParams.update({"svg.fonttype": "none", "svg.hashsalt": "lesscode-self-example-v1",
                     "font.size": 11, "axes.spines.top": False, "axes.spines.right": False})
metadata = {"Date": None, "Description": "Measured data: docs/evidence/self-evaluation-2026-10-31.json"}

figure, axes = plt.subplots(1, 2, figsize=(10, 3.7), layout="constrained")
panels = [
    ("Cyclomatic complexity (Lizard)", "CCN", [package_metrics[side]["target_function"]["ccn"]
                                            for side in ("baseline", "candidate")]),
    ("Logical lines (Radon)", "LLOC", [auxiliary_sides[side]["target_function"]["lloc"]
                                      for side in ("baseline", "candidate")]),
]
for axis, (title, unit, values) in zip(axes, panels):
    bars = axis.bar(["Before", "After"], values, color=["#64748b", "#0f766e"], width=0.5)
    axis.set(title=title, ylabel=unit, ylim=(0, max(values) * 1.25))
    axis.bar_label(bars, padding=4)
    axis.set_axisbelow(True)
    axis.grid(axis="y", alpha=0.2)
figure.suptitle("One function: decision.evaluate_threshold", fontsize=15)
figure.savefig(output / "self-evaluation-local.svg", metadata=metadata)
plt.close(figure)

hotspots = package_metrics["candidate"]["hotspots"][:6]
labels = [f"{Path(function['path']).stem}.{function['name']}" for function in hotspots]
values = [function["ccn"] for function in hotspots]
figure, axis = plt.subplots(figsize=(9, 4), layout="constrained")
bars = axis.barh(labels, values, color="#b45309", height=0.6)
axis.invert_yaxis()
axis.set(title="Remaining package hotspots after the change", xlabel="Lizard CCN",
         xlim=(0, max(values) * 1.15))
axis.bar_label(bars, padding=4)
axis.set_axisbelow(True)
axis.grid(axis="x", alpha=0.2)
figure.savefig(output / "self-evaluation-hotspots.svg", metadata=metadata)
plt.close(figure)
for chart_name in ("self-evaluation-local.svg", "self-evaluation-hotspots.svg"):
    chart = output / chart_name
    chart.write_text("\n".join(line.rstrip() for line in chart.read_text().splitlines()) + "\n",
                     encoding="utf-8")
print("Rendered measured local change and remaining hotspots from", receipt)
