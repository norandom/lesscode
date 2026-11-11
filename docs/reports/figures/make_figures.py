"""Figures for docs/reports/agnostic-driver-feasibility.md.

Research tooling, standard library only. Reads the committed receipts and ledgers and writes static SVG
files beside this script:

    uv run --no-project python docs/reports/figures/make_figures.py

Colours are the first four categorical slots of a palette checked for colour-vision separation; every mark
carries a visible number and each figure has a table in the report, so nothing is told by colour alone.
"""

import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
RESULTS = HERE.parents[1] / "experiments" / "annotation-driver" / "results"
SURFACE, INK, SECOND, MUTED, GRID, AXIS = "#fcfcfb", "#0b0b0b", "#52514e", "#898781", "#e1e0d9", "#c3c2b7"
BLUE, ORANGE, AQUA, YELLOW = "#2a78d6", "#eb6834", "#1baf7a", "#eda100"
FONT = 'font-family="system-ui, -apple-system, Segoe UI, Helvetica, Arial, sans-serif"'
WIDTH, LEFT, RIGHT, ROW = 820, 300, 70, 30
PRICE = {"claude-opus-5-5": (4.0, 20.0), "mistral-large-4": (0.7, 2.1)}


def text(x, y, body, size=12, fill=INK, anchor="start", weight="normal"):
    body = str(body).replace("&", "&amp;").replace("<", "&lt;")
    return f'<text x="{x:.1f}" y="{y:.1f}" font-size="{size}" fill="{fill}" text-anchor="{anchor}" font-weight="{weight}">{body}</text>'


def bar(x, y, w, h, fill, round_end=True):
    """A bar from x to x + w: flat at the baseline, 4px rounded at the data end."""
    if w <= 0:
        return ""
    r = min(4.0, w / 2) if round_end else 0
    return (f'<path d="M{x:.1f},{y:.1f} h{w - r:.1f} a{r},{r} 0 0 1 {r},{r} v{h - 2 * r:.1f} '
            f'a{r},{r} 0 0 1 -{r},{r} h-{w - r:.1f} z" fill="{fill}"/>')


def legend(items, y):
    out, x = [], LEFT
    for label, colour in items:
        out += [f'<rect x="{x}" y="{y - 10}" width="12" height="12" rx="2" fill="{colour}"/>', text(x + 18, y, label, 12, SECOND)]
        x += 30 + 7 * len(label)
    return out


def page(name, title, note, rows, body):
    height = 86 + rows * ROW + 34
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {WIDTH} {height}" width="{WIDTH}" height="{height}" {FONT} role="img">',
             f"<title>{title}</title>", f'<rect width="{WIDTH}" height="{height}" fill="{SURFACE}"/>',
             text(20, 28, title, 15, INK, weight="600"), *body, text(20, height - 14, note, 11, MUTED), "</svg>"]
    (HERE / name).write_text("\n".join(p for p in parts if p) + "\n", encoding="utf-8", newline="\n")


def receipt(path):
    return json.loads((RESULTS / path).read_text(encoding="utf-8"))["counts_by_class"]


def classes():
    rows = [("Python ratelimit.py (R73)", "native/python-receipt.json"),
            ("TypeScript state.ts (R73)", "native/ts-state-receipt.json"),
            ("TypeScript verification.ts (R73)", "native/ts-verification-receipt.json"),
            ("Rust update.rs (R73)", "native/rust-receipt.json"),
            ("Java Lexer.java (R73)", "native/java-scaled-receipt.json"),
            ("TypeScript degradation.ts, candidate (R75)", "specimens/ts-candidate-receipt.json"),
            ("TypeScript degradation.ts, original (R75)", "specimens/ts-baseline-receipt.json"),
            ("Java CSVPrinter.java, candidate (R75)", "specimens/java-candidate-native-receipt.json"),
            ("Java CSVPrinter.java, original (R75)", "specimens/java-baseline-native-receipt.json"),
            ("Rust recording.rs, original (R75)", "specimens/rust-baseline-receipt.json")]
    order = [("killed", BLUE), ("survived", ORANGE), ("invalid", AQUA), ("timeout", YELLOW)]
    span = WIDTH - LEFT - RIGHT
    body = legend(order, 54)
    for i, (label, path) in enumerate(rows):
        counts = receipt(path)
        total = sum(counts.get(k, 0) for k, _ in order)
        y, x = 74 + i * ROW, float(LEFT)
        body.append(text(LEFT - 10, y + 15, label, 12, INK, "end"))
        for key, colour in order:
            n = counts.get(key, 0)
            w = span * n / total
            if n:
                body.append(f'<rect x="{x:.1f}" y="{y}" width="{max(w - 2, 1):.1f}" height="20" rx="2" fill="{colour}"/>')
                if w > 24:
                    body.append(text(x + (w - 2) / 2, y + 14, n, 11, INK if colour == YELLOW else "#ffffff", "middle"))
            x += w
        body.append(text(WIDTH - RIGHT + 8, y + 15, f"n = {total}", 11, SECOND))
    page("mutant-classes.svg", "Mutant classes per file, with each project's own tests",
         "Share of each file's mutants by class; numbers in the bars are counts. Source: committed run receipts.", len(rows), body)


def single_series(name, title, note, rows, unit, colours=None, groups=None):
    """Horizontal bars with the value at the bar's end; rows are (label, value, detail)."""
    span = WIDTH - LEFT - RIGHT - 170  # room for the value and its detail after the longest bar
    top = max(v for _, v, _ in rows)
    body = legend(groups, 54) if groups else []
    for i, (label, value, detail) in enumerate(rows):
        y = 74 + i * ROW
        body.append(text(LEFT - 10, y + 15, label, 12, INK, "end"))
        w = span * value / top
        body.append(bar(LEFT, y, max(w, 1.5), 20, colours[i] if colours else BLUE))
        body.append(text(LEFT + max(w, 1.5) + 6, y + 15, f"{value:g} {unit}{detail}", 11, SECOND))
    body.append(f'<line x1="{LEFT}" y1="70" x2="{LEFT}" y2="{74 + len(rows) * ROW}" stroke="{AXIS}" stroke-width="1"/>')
    page(name, title, note, len(rows), body)


def efficiency():
    # label, wall seconds, mutants, copies; wall times from the entries R73 and R75.
    runs = [("Python ratelimit.py", 118, 197, "6 copies"), ("TypeScript state.ts", 107, 71, "6 copies"),
            ("TypeScript verification.ts", 88, 55, "6 copies"), ("Java Lexer.java, containers", 814, 170, "4 copies"),
            ("Java CSVPrinter.java original, native", 946, 94, "4 copies"),
            ("Java CSVPrinter.java candidate, native", 1462, 93, "4 copies"),
            ("Rust recording.rs", 738, 133, "3 copies"), ("Rust update.rs", 1380, 64, "1 copy")]
    rows = [(label, round(wall / n, 1), f"  ({n} mutants, {copies})") for label, wall, n, copies in runs]
    single_series("seconds-per-mutant.svg", "Wall time per mutant, whole batch including setup and unmutated runs",
                  "Seconds of wall time divided by mutants run. One shared machine; other batches ran beside some of these.",
                  rows, "s")


def admission():
    rows = [("Opus, guide v3 (R69)", 15, BLUE), ("Opus, guide v3, rerun (R78)", 15, BLUE), ("Opus, plus node kinds (R78)", 15, BLUE),
            ("Opus, guide from GEPA (R71)", 20, BLUE), ("Opus, guide from MIPROv2 (R79)", 15, BLUE),
            ("Mistral Large 4, guide v3 (R72)", 5, ORANGE), ("Mistral Large 4, guide v3, rerun (R78)", 5, ORANGE),
            ("Mistral Large 4, plus node kinds (R78)", 6, ORANGE)]
    single_series("first-replies-admitted.svg", "Pack authoring: first replies admitted, of 20 (five per language)",
                  "Four languages, five fresh runs each. On Mistral the optimiser found no guide to judge (R74).",
                  [(label, value, " of 20") for label, value, _ in rows], "", [c for _, _, c in rows],
                  [("claude-opus-5-5", BLUE), ("mistral-large-4", ORANGE)])


def spend():
    runs = [("Baseline, Opus (R68)", "opus"), ("Baseline rerun, Opus (R69)", "opus-rerun1"),
            ("GEPA search and judgement, Opus (R71)", ["optimise1-opus", "optimise1-opus-final-topup"]),
            ("Baseline, Mistral (R72)", ["mistral", "mistral-attempt1-crashed"]), ("GEPA search, Mistral (R74)", "optimise1-mistral"),
            ("Node kinds, Mistral, two arms (R78)", ["kinds-mistral-control", "kinds-mistral-treatment"]),
            ("Node kinds, Opus, two arms (R78)", ["kinds-opus-control", "kinds-opus-treatment"]),
            ("MIPROv2 search and judgement, Opus (R79)", "optimise2-opus-mipro"),
            ("Annotation baseline, Opus (R83)", "annot-opus"), ("Annotation baseline, Mistral (R83)", "annot-mistral"),
            ("Annotation rules over three runs, Opus (R85)", "annot-opus-ensemble")]
    rows, colours = [], []
    for label, names in runs:
        usd, calls, model = 0.0, 0, None
        for name in [names] if isinstance(names, str) else names:
            for line in (RESULTS / "slice-b" / name / "ledger.jsonl").read_text(encoding="utf-8").splitlines():
                if line.strip():
                    row = json.loads(line)
                    model = row.get("model") or model
                    rate = PRICE[row["model"]]
                    usd += ((row.get("input_tokens") or 0) * rate[0] + (row.get("output_tokens") or 0) * rate[1]) / 1e6
                    calls += 1
        rows.append((label, round(usd, 2), f"  ({calls} calls)"))
        colours.append(BLUE if model == "claude-opus-5-5" else ORANGE)
    total = sum(v for _, v, _ in rows)
    single_series("slice-b-spend.svg", f"Slice B model spend per run at list prices: {total:.2f} USD in all",
                  "From the runs' ledgers. Gate attempts before Slice B (about 3.2 USD) are not shown.",
                  rows, "USD", colours, [("claude-opus-5-5", BLUE), ("mistral-large-4", ORANGE)])


def call_arguments():
    rows = [("Python ratelimit.py", "python"), ("TypeScript state.ts", "ts-state"),
            ("TypeScript verification.ts", "ts-verification"), ("Java Lexer.java", "java"), ("Rust update.rs", "rust")]
    order = [("killed", BLUE), ("survived", ORANGE), ("invalid", AQUA)]
    counts = {}
    for label, name in rows:
        mutants = json.loads((RESULTS / "native-args" / f"{name}-receipt.json").read_text(encoding="utf-8"))["mutants"]
        new = [m["class"] for m in mutants if m["site"]["kind"] == "call_argument"]
        counts[label] = ({key: new.count(key) for key, _ in order}, len(mutants) - len(new))
    top = max(sum(c.values()) for c, _ in counts.values())
    span = WIDTH - LEFT - RIGHT - 170
    body = legend(order, 54)
    for i, (label, _) in enumerate(rows):
        found, earlier = counts[label]
        y, x = 74 + i * ROW, float(LEFT)
        body.append(text(LEFT - 10, y + 15, label, 12, INK, "end"))
        for key, colour in order:
            w = span * found[key] / top
            if found[key]:
                body.append(f'<rect x="{x:.1f}" y="{y}" width="{max(w - 2, 1):.1f}" height="20" rx="2" fill="{colour}"/>')
                if w > 18:
                    body.append(text(x + (w - 2) / 2, y + 14, found[key], 11, "#ffffff", "middle"))
            x += w
        body.append(text(x + 6, y + 15, f"{sum(found.values())} new beside {earlier}", 11, SECOND))
    page("call-argument-mutants.svg", "Mutants added by the call-argument operators, by class",
         "Counts of new mutants per file. Earlier kinds are unchanged. Source: receipts under results/native-args.",
         len(rows), body)


def annotation_agreement():
    summary = json.loads((RESULTS / "slice-b" / "annot-opus" / "summary.json").read_text(encoding="utf-8"))
    names = [("rust-mmhmem-setup-update", "Rust update.rs"), ("python-mvibe-ratelimit", "Python ratelimit.py"),
             ("typescript-openultracode-state", "TypeScript state.ts"),
             ("typescript-openultracode-verification", "TypeScript verification.ts"),
             ("java-commons-csv-lexer", "Java Lexer.java")]
    rows, colours = [], []
    for key, label in names:
        rows.append((f"{label}: Opus run and Opus run", summary["arm"][key]["median_agreement"], ""))
        rows.append((f"{label}: Opus run and Mistral run", summary["compared_with"]["cross_arm"][key]["median_agreement"], ""))
        colours += [BLUE, ORANGE]
    single_series("annotation-agreement.svg", "Annotation: share of named sites two runs have in common (median)",
                  "Sites both runs name, divided by sites either names; accepted annotations only. Five runs per scope and model.",
                  rows, "", colours, [("two Opus runs", BLUE), ("an Opus run and a Mistral run", ORANGE)])


if __name__ == "__main__":
    call_arguments()
    annotation_agreement()
    classes()
    efficiency()
    admission()
    spend()
    print("\n".join(sorted(p.name for p in HERE.glob("*.svg"))))
