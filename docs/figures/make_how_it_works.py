"""Write docs/figures/how-it-works.svg: three things a developer does, shown as what is typed and seen."""
import sys
from html import escape
from pathlib import Path

MONO = 'font-family="ui-monospace, SFMono-Regular, Consolas, Menlo, monospace"'
INK, MUTED, LINE, PANEL = "#1f2328", "#59636e", "#d1d9e0", "#f6f8fa"
out = []


def text(x, y, s, fill=INK, extra=""):
    out.append(f'<text x="{x}" y="{y}" fill="{fill}" xml:space="preserve" {extra}>{s}</text>')


def heading(number, y, title, sub):
    out.append(f'<circle cx="32" cy="{y}" r="13" fill="{INK}"/>')
    text(32, y + 5, number, "#ffffff", 'font-weight="600" font-size="13" text-anchor="middle"')
    text(54, y - 2, title, INK, 'font-weight="600" font-size="15"')
    text(54, y + 15, sub, MUTED)


def box(x, y, w, h, fill=PANEL):
    out.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="6" fill="{fill}" stroke="{LINE}"/>')


def code(x, y, lines, size="11"):
    for i, line in enumerate(lines):
        text(x, y + 16 * i, escape(line), INK, f'{MONO} font-size="{size}"')


def tint(x, y_first_line, first, last, colour):
    top = y_first_line - 13 + 16 * (first - 1)
    out.append(f'<rect x="{x}" y="{top}" width="408" height="{16 * (last - first + 1)}" fill="{colour}"/>')


# 1 ------------------------------------------------------------------
heading("1", 30, "See where your tests guard the code",
        "Small faults are planted one at a time, in a copy. Do your tests notice?")
box(20, 58, 840, 132)
m = f'{MONO} font-size="11.5"'
text(34, 80, f'<tspan fill="{MUTED}">$</tspan> lesscode-core step plan --config run.json --work ../work --out plan.json', INK, m)
text(34, 102, "function                              noticed  unnoticed  tokens", MUTED, m)
text(34, 120, "recording.rs: replay_to_l2                  0         20    1010   no fault noticed", INK, m)
text(34, 138, "recording.rs: assemble_meta                 0         17     824   no fault noticed", INK, m)
text(34, 156, "recording.rs: verdict                      10          1     151", INK, m)
text(34, 174, "... 28 more functions", MUTED, m)
out.append('<rect x="28" y="144" width="560" height="17" rx="3" fill="none" stroke="#0969da" stroke-width="1.5"/>')
out.append(f'<path d="M650,152 H596" stroke="{MUTED}" stroke-width="1.2" fill="none" marker-end="url(#tip)"/>')
text(656, 120, "not today: nothing guards these", MUTED)
text(656, 149, "start here:", INK, 'font-weight="600"')
text(656, 164, "one test away from fully guarded", MUTED)

# 2 ------------------------------------------------------------------
heading("2", 228, "Pin first, then make it simpler",
        "You or a model write the test and the change. Whether the test counts is checked, not assumed.")
box(20, 256, 840, 50)
text(34, 277, f'<tspan fill="{MUTED}">$</tspan> lesscode-core step pin --declaration step.json --fault 2 --work ../work', INK, m)
text(34, 295, '#2 line 864 swap_next: <tspan fill="#1a7f37" font-weight="600">admitted</tspan> '
     f'<tspan fill="{MUTED}">(tests pass on the code as it is, fail on the fault)</tspan>', INK, m)

before = [
    "let mut reasons = if arm_exit == 0 {",
    "    Vec::new()",
    "} else {",
    '    vec![format!("arm exit {arm_exit}")]',
    "};",
    "if !fresh {",
    '    reasons.push("replay not fresh".into());',
    "}",
    "if level2_ops > 0 && !has_evidence {",
    '    reasons.push(format!("{level2_ops} L2 diffs"));',
    "}",
    "if reasons.is_empty() {",
    "    return (0, reasons);",
    "}",
    "(if arm_exit == 2 { 2 } else { 1 }, reasons)",
]
after = [
    "let mut reasons = Vec::new();",
    "if arm_exit != 0 {",
    '    reasons.push(format!("arm exit {arm_exit}"));',
    "}",
    "if !fresh {",
    '    reasons.push("replay not fresh".into());',
    "}",
    "if level2_ops > 0 && !has_evidence {",
    '    reasons.push(format!("{level2_ops} L2 diffs"));',
    "}",
    "let code = if reasons.is_empty() {",
    "    0",
    "} else if arm_exit == 2 {",
    "    2",
    "} else {",
    "    1",
    "};",
    "(code, reasons)",
]
top, first = 338, 362
height = 16 * len(after) + 22
text(20, 330, "before", INK, 'font-weight="600"')
text(430, 330, "the list is set up inside a branch; two ways out", MUTED, 'text-anchor="end"')
box(20, top, 410, height, "#ffffff")
tint(21, first, 1, 5, "#ffebe9")
tint(21, first, 12, 15, "#ffebe9")
code(32, first, before)
text(450, 330, "after", INK, 'font-weight="600"')
text(860, 330, "set up once, checks in a row, one way out", MUTED, 'text-anchor="end"')
box(450, top, 410, height, "#ffffff")
tint(451, first, 1, 4, "#dafbe1")
tint(451, first, 11, 18, "#dafbe1")
code(462, first, after)
note = top + height + 18
text(20, note, "verdict() in native/src/recording.rs, strings shortened. Same result on 480 input combinations. "
     "More lines, one token fewer: clearer, not shorter.", MUTED)

# 3 ------------------------------------------------------------------
y3 = note + 40
heading("3", y3, "One answer on the branch, enforced when you push",
        "Your tests must pass before and after, and must not notice fewer faults in what you touched.")
main_y, branch_y = y3 + 60, y3 + 104
out.append(f'<g fill="none" stroke-width="2"><path d="M30,{main_y} H400" stroke="{MUTED}"/>'
           f'<path d="M90,{main_y} C112,{main_y} 112,{branch_y} 134,{branch_y} H300 C322,{branch_y} 322,{main_y} 344,{main_y}" stroke="#0969da"/></g>')
for cx in (50, 90):
    out.append(f'<circle cx="{cx}" cy="{main_y}" r="5" fill="{MUTED}"/>')
for cx in (170, 250):
    out.append(f'<circle cx="{cx}" cy="{branch_y}" r="5" fill="#0969da"/>')
out.append(f'<circle cx="344" cy="{main_y}" r="5.5" fill="#ffffff" stroke="{MUTED}" stroke-width="2" stroke-dasharray="2 2"/>')
text(30, main_y - 14, "main", MUTED, f'{MONO} font-size="11"')
text(344, main_y - 14, "merge", MUTED, 'font-size="11" text-anchor="middle"')
text(140, branch_y - 10, "step/one-exit", MUTED, f'{MONO} font-size="10.5"')
text(170, branch_y + 20, "pins", INK, 'font-size="11" text-anchor="middle"')
text(250, branch_y + 20, "change", INK, 'font-size="11" text-anchor="middle"')
text(210, branch_y + 38, "two commits, in this order", MUTED, 'font-size="11" text-anchor="middle"')

ty = y3 + 34
box(424, ty, 436, 126)
text(438, ty + 22, f'<tspan fill="{MUTED}">$</tspan> lesscode-core step check --declaration step.json ...', INK, m)
text(438, ty + 40, 'step one-exit: <tspan fill="#1a7f37" font-weight="600">accepted</tspan>', INK, m)
text(438, ty + 64, f'<tspan fill="{MUTED}">$</tspan> git push', INK, m)
text(438, ty + 82, 'step/one-exit: <tspan fill="#1a7f37" font-weight="600">accepted</tspan>', INK, m)
text(438, ty + 102, "or, when the code was changed after the check:", MUTED, m)
text(438, ty + 118, 'step/one-exit: <tspan fill="#cf222e" font-weight="600">refused</tspan>', INK, m)
foot = ty + 126 + 24
text(20, foot, "Output shortened. The step's spec is kept consistent by openidea-kiro; lesscode checks the code and the tests.", MUTED)

total = foot + 16
head = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 880 {total}" width="880" height="{total}" '
        'font-family="-apple-system, Segoe UI, Helvetica, Arial, sans-serif" font-size="12" role="img">\n'
        "<title>Three things you do: see where your tests guard the code, pin and then simplify one function, "
        "and let one check decide on the branch before you push</title>\n"
        '<defs><marker id="tip" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto">'
        f'<path d="M0,1 L10,5 L0,9 z" fill="{MUTED}"/></marker></defs>\n'
        f'<rect width="880" height="{total}" fill="#ffffff"/>\n')
Path(sys.argv[1]).write_text(head + "\n".join(out) + "\n</svg>\n", encoding="utf-8", newline="\n")
print(total)
