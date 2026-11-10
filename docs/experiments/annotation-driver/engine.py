"""Language-agnostic mutation engine for the annotation-driver prototype gate.

One engine for every language (gate Amendment 3b): all language knowledge
comes from the data pack ``tables/<language>.json``. Site eligibility is the
validator's own (``validator.site_check``). No model, and no I/O beyond
reading scope files and packs.

Positions: ast-grep-py reports 0-based lines, columns counted in characters
(code points) and ``Pos.index`` as a character offset into the parsed string.
A CRLF pair is one line break; the CR counts as a character. The engine parses
the exact file text (UTF-8 bytes decoded, line endings kept), so offsets
address the file as it is on disk. ``run_engine_checks.py`` checks this.
"""

from __future__ import annotations

import random
import re
from dataclasses import dataclass

from ast_grep_py import SgNode, SgRoot

import validator as v

OPERATOR_KINDS = ("branch_condition", "failure_state", "state_write", "error_order")


@dataclass(frozen=True)
class Site:
    """One mutable location. Lines are 1-based, columns 0-based characters."""

    file: str
    kind: str
    node_kind: str
    start_line: int
    start_column: int
    end_line: int
    end_column: int
    text: str


@dataclass(frozen=True)
class Mutant:
    """A mutated file: its full text and the replaced character ranges of the original."""

    site: Site
    operator: str
    text: str
    spans: tuple[tuple[int, int], ...]


def read_text(scope: v.Scope, file: str) -> str:
    """The exact file text: UTF-8 bytes decoded, line endings kept."""
    return (scope.root / file).read_bytes().decode("utf-8")


def site_order(site: Site) -> tuple:
    return (site.file, site.start_line, site.start_column, site.kind, site.end_line, site.end_column)


def _span(site: Site) -> tuple[int, int, int, int]:
    return (site.start_line, site.start_column, site.end_line, site.end_column)


def _named_children(node: SgNode, table: dict) -> list[SgNode]:
    return [c for c in node.children() if c.is_named() and c.kind() not in table["comment_kinds"]]


def _normalise(table: dict, kind: str, node: SgNode) -> SgNode | None:
    """The node an operator acts on; None for a condition the pack excludes."""
    if kind != "branch_condition":
        return node
    if node.kind() in table.get("unwrap_kinds", ()):
        inner = _named_children(node, table)
        if len(inner) == 1:
            node = inner[0]
    excluded = set(table.get("operators", {}).get(kind, {}).get("exclude_kinds", ()))
    if any(n.kind() in excluded for n in v._chain(node)):
        return None
    return node


def _site_at(table: dict, file: str, kind: str, node: SgNode) -> Site | None:
    reason, matched = v.site_check(table, node, kind)
    target = _normalise(table, kind, matched) if reason == "ok" else None
    if target is None:
        return None
    r = target.range()
    return Site(file, kind, target.kind(), r.start.line + 1, r.start.column,
                r.end.line + 1, r.end.column, target.text())


def file_sites(table: dict, language: str, file: str, text: str) -> list[Site]:
    """Eligible sites of one file text, deduplicated by (span, kind), in order."""
    found: dict[tuple, Site] = {}
    for node in v._walk(SgRoot(text, language).root()):
        for kind in OPERATOR_KINDS:
            site = _site_at(table, file, kind, node)
            if site is not None:
                found.setdefault((_span(site), kind), site)
    return sorted(found.values(), key=site_order)


def eligible_sites(scope: v.Scope) -> list[Site]:
    """Every site the pack makes eligible, ordered by file, start position and kind."""
    table = v.load_table(scope.language)
    sites = [s for f in scope.files if not v.is_test_file(scope, f)
             for s in file_sites(table, scope.language, f, read_text(scope, f))]
    return sorted(sites, key=site_order)


def site_from_annotation(scope: v.Scope, annotation: dict) -> Site | None:
    """The normalised site of an annotation ``validator.validate`` accepted."""
    kind = annotation["kind"]
    if kind not in OPERATOR_KINDS:
        return None
    file = str(annotation["file"]).replace("\\", "/")
    root = SgRoot(read_text(scope, file), scope.language).root()
    nodes = v.anchor_nodes(root, annotation["start_line"], annotation["end_line"], annotation["anchor"])
    return _site_at(v.load_table(scope.language), file, kind, nodes[0]) if nodes else None


def _fill(template: str, table: dict, node: SgNode) -> str:
    values = {"text": lambda: node.text(), "noop": lambda: table["noop_statement"],
              "child0": lambda: _named_children(node, table)[0].text()}
    return re.sub(r"\{(\w+)\}", lambda m: values[m.group(1)](), template)


def _edits(table: dict, kind: str, node: SgNode) -> tuple[str, list[tuple[SgNode, str]]] | None:
    """(operator label, [(node to replace, replacement)]), or None without an operator."""
    ops = table.get("operators", {}).get(kind, {})
    parent = node.parent()
    if kind == "branch_condition" and "template" in ops:
        return "negate", [(node, _fill(ops["template"], table, node))]
    if kind == "failure_state" and node.kind() in ops.get("by_kind", {}):
        template = ops["by_kind"][node.kind()]
        label = "drop" if template == "{noop}" else "rewrite"
        return f"{label}:{node.kind()}", [(node, _fill(template, table, node))]
    if kind == "state_write" and "statement" in ops and parent is not None \
            and parent.kind() in table.get("statement_wrappers", ()):
        return "drop_statement", [(parent, _fill(ops["statement"], table, parent))]
    if kind == "error_order" and ops.get("swap_next"):
        chain = v._chain(node)
        sibling = v.next_check(chain, table["kinds"][kind], set(table["comment_kinds"]))
        if sibling is not None:
            return "swap_next", [(chain[0], sibling.text()), (sibling, chain[0].text())]
    return None


def _apply(text: str, edits: list[tuple[SgNode, str]]) -> tuple[str, tuple[tuple[int, int], ...]]:
    """Replace each node by exact character offsets, last first; never by searching text."""
    spans = []
    for node, new in sorted(edits, key=lambda e: e[0].range().start.index, reverse=True):
        start, end = node.range().start.index, node.range().end.index
        if text[start:end] != node.text():
            raise ValueError(f"offsets {start}:{end} do not address the node text")
        text = text[:start] + new + text[end:]
        spans.append((start, end))
    return text, tuple(sorted(spans))


def _node_of(root: SgNode, site: Site) -> SgNode | None:
    for node in v._walk(root):
        r = node.range()
        if node.kind() == site.node_kind and \
                (r.start.line + 1, r.start.column, r.end.line + 1, r.end.column) == _span(site):
            return node
    return None


def mutate(scope: v.Scope, site: Site) -> Mutant | None:
    """Apply the pack's operator for the site; None when the pack gives none (no_operator)."""
    table = v.load_table(scope.language)
    text = read_text(scope, site.file)
    node = _node_of(SgRoot(text, scope.language).root(), site)
    if node is None:
        raise ValueError(f"site no longer present in {site.file}: {site}")
    edits = _edits(table, site.kind, node)
    if edits is None:
        return None
    mutated, spans = _apply(text, edits[1])
    return Mutant(site, edits[0], mutated, spans)


def select(sites: list[Site], cap: int = 40, seed: int = 20261110) -> list[Site]:
    """Seeded shuffle, then round-robin across kinds (by name) until ``cap`` or exhaustion."""
    shuffled = sorted(sites, key=site_order)
    random.Random(seed).shuffle(shuffled)
    queues = [[s for s in shuffled if s.kind == k] for k in sorted({s.kind for s in shuffled})]
    chosen: list[Site] = []
    while len(chosen) < cap and any(queues):
        for queue in queues:
            if queue and len(chosen) < cap:
                chosen.append(queue.pop(0))
    return chosen
