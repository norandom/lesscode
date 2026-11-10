"""Deterministic checks for the annotation-driver prototype gate.

Two responsibilities, both model-free:

* ``validate`` accepts or rejects located annotations against the syntax tree
  of a frozen scope (annotation contract version 0, five rules);
* ``classify_mutant`` says whether a mutated file is a real, parseable change.

Everything language-specific comes from ``tables/<language>.json``, or from
the directory named by the environment variable ``AD_TABLES_DIR`` when it is
set and non-empty. This file contains no language names beyond what it reads
from the scope manifest.
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass
from pathlib import Path

from ast_grep_py import SgNode, SgRoot

HERE = Path(__file__).parent
KINDS = ("failure_state", "branch_condition", "error_order", "state_write", "pure_shell_boundary")


def tables_dir() -> Path:
    """The pack directory: ``AD_TABLES_DIR`` when set and non-empty, else ``tables/`` next to this file."""
    override = os.environ.get("AD_TABLES_DIR")
    return Path(override) if override else HERE / "tables"


def load_table(language: str) -> dict:
    return json.loads((tables_dir() / f"{language}.json").read_text(encoding="utf-8"))


@dataclass(frozen=True)
class Scope:
    """One frozen scope: a root directory, its language and the files in it."""

    name: str
    language: str
    root: Path
    files: tuple[str, ...]
    test_paths: tuple[str, ...]

    def source(self, file: str) -> str:
        return (self.root / file).read_text(encoding="utf-8")


def load_scopes(scopes_dir: Path) -> dict[str, Scope]:
    manifest = json.loads((HERE / "scopes.json").read_text(encoding="utf-8"))
    return {
        s["name"]: Scope(s["name"], s["language"], scopes_dir / s["repository"],
                         tuple(s["files"]), tuple(s["test_paths"]))
        for s in manifest["scopes"]
    }


def _span(node: SgNode) -> tuple[int, int, int, int]:
    r = node.range()
    return (r.start.line, r.start.column, r.end.line, r.end.column)


def _walk(node: SgNode):
    yield node
    for child in node.children():
        yield from _walk(child)


def _ancestors(node: SgNode):
    parent = node.parent()
    while parent is not None:
        yield parent
        parent = parent.parent()


def _chain(node: SgNode) -> list[SgNode]:
    """The node plus the wrappers and wrapped nodes that cover the same text."""
    span = _span(node)
    chain = [node]
    for parent in _ancestors(node):
        if _span(parent) != span:
            break
        chain.insert(0, parent)
    inner = node
    while True:
        same = [c for c in inner.children() if _span(c) == span]
        if not same:
            break
        inner = same[0]
        chain.append(inner)
    return chain


def _only_named_child(parent: SgNode, node: SgNode) -> bool:
    named = [c for c in parent.children() if c.is_named()]
    return len(named) == 1 and named[0].kind() == node.kind() and _span(named[0]) == _span(node)


def _returned(node: SgNode, parent: SgNode, rule: dict) -> bool:
    """Pack format version 1: a returned expression of a listed kind with one of its own operator tokens listed."""
    returned = rule.get("returned") or {}
    return parent.kind() in returned.get("of", ()) and node.kind() in returned.get("kinds", ()) \
        and any(not c.is_named() and c.text() in returned.get("tokens", ()) for c in node.children())


def _kind_match(chain: list[SgNode], rule: dict, unwrap: frozenset[str] = frozenset(),
                wrappers: frozenset[str] = frozenset()) -> SgNode | None:
    """The first node of the chain (or its parenthesis wrapper) the rule allows.

    ``wrappers`` are the pack's statement wrapper kinds, for the rule key
    ``call_statements`` (pack format version 1).
    """
    # A condition is often wrapped in parentheses that belong to the statement;
    # an anchor on the inner expression then counts as the wrapped node.
    wrapper = chain[0].parent()
    candidates = [*chain, wrapper] if wrapper is not None and wrapper.kind() in unwrap else chain
    for node in candidates:
        if node.kind() in rule.get("nodes", ()):
            return node
        for prefix in rule.get("nodes_with_prefix", {}).get(node.kind(), ()):
            if node.text().startswith(prefix):
                return node
        parent = node.parent()
        if parent is not None:
            for parent_kind, field in rule.get("field_of", ()):
                target = parent.field(field) if parent.kind() == parent_kind else None
                if target is not None and _span(target) == _span(node):
                    return node
            if _only_named_child(parent, node):
                # A call that is a statement of its own; a returned yes-or-no expression.
                if node.kind() in rule.get("call_statements", ()) and parent.kind() in wrappers:
                    return node
                if _returned(node, parent, rule):
                    return node
    return None


def next_check(chain: list[SgNode], rule: dict, skip: set[str]) -> SgNode | None:
    """The following named sibling, if it is of a kind the rule names."""
    sibling = chain[0].next()
    while sibling is not None and (not sibling.is_named() or sibling.kind() in skip):
        sibling = sibling.next()
    if sibling is None or _kind_match(_chain(sibling), {"nodes": rule.get("nodes", ())}) is None:
        return None
    return sibling


def _in_test_region(node: SgNode, markers: tuple[str, ...]) -> bool:
    if not markers:
        return False
    for candidate in [node, *_ancestors(node)]:
        previous = candidate.prev()
        while previous is not None and "attribute" in previous.kind():
            if any(marker in previous.text() for marker in markers):
                return True
            previous = previous.prev()
    return False


def is_test_file(scope: Scope, normalized: str) -> bool:
    return any(normalized.startswith(p) or f"/{p}" in f"/{normalized}" for p in scope.test_paths)


def anchor_nodes(root: SgNode, start: int, end: int, anchor: str) -> list[SgNode]:
    """Nodes starting on line ``start`` and ending on line ``end`` whose text is ``anchor``.

    Text is compared with universal newlines, as ``Scope.source`` reads it, so
    a tree parsed from the exact file bytes matches the same anchors. Only
    named nodes count: a lone bracket or operator token is not a site.
    """
    return [n for n in _walk(root)
            if n.is_named() and n.range().start.line == start - 1 and n.range().end.line == end - 1
            and n.text().replace("\r\n", "\n").replace("\r", "\n") == anchor]


def match_anchor(table: dict, root: SgNode, start: int, end: int, anchor: str) -> tuple[list[SgNode], bool]:
    """Rule 2: the nodes the anchor matches, and whether the statement terminator was appended.

    Pack format version 1: when no node matches and the pack's
    ``statement_terminator`` is non-empty, the anchor plus the terminator is
    tried under the same rules.
    """
    nodes = anchor_nodes(root, start, end, anchor)
    terminator = table.get("statement_terminator") or ""
    if nodes or not terminator:
        return nodes, False
    completed = anchor_nodes(root, start, end, anchor + terminator)
    return completed, bool(completed)


def site_check(table: dict, node: SgNode, kind: str) -> tuple[str, SgNode | None]:
    """Rules 3 and 4 and the test-region rule for one node: (reason, allowed node)."""
    # An anchor that includes the statement terminator names the whole statement;
    # what the statement wraps is the site (gate Amendment 12).
    if node.kind() in table.get("statement_wrappers", ()):
        inner = [c for c in node.children() if c.is_named() and c.kind() not in table["comment_kinds"]]
        if len(inner) == 1:
            node = inner[0]
    chain = _chain(node)
    hidden = set(table["comment_kinds"]) | set(table["string_kinds"])
    if any(n.kind() in hidden for n in [*chain, *_ancestors(chain[0])]):
        return "in_comment_or_string", None
    if _in_test_region(chain[0], tuple(table.get("test_attribute_markers", ()))):
        return "test_region", None
    rule = table["kinds"].get(kind, {})
    matched = _kind_match(chain, rule, frozenset(table.get("unwrap_kinds", ())),
                          frozenset(table.get("statement_wrappers", ())))
    if matched is None:
        return "kind_mismatch", None
    if rule.get("requires_next_sibling") and next_check(chain, rule, set(table["comment_kinds"])) is None:
        return "kind_mismatch", None
    return "ok", matched


def validate(scope: Scope, annotations: list[dict]) -> list[dict]:
    """Return one decision per annotation: ``accepted`` plus a ``reason``.

    Rules (contract version 0): 1 file in scope and not a test; 2 anchor is
    exactly the text of one syntax node at the stated lines; 3 that node's
    kind is allowed for the annotation kind; 4 not inside a comment or string;
    5 not a duplicate of an accepted annotation. Pack format version 1 extends
    rule 2 (``match_anchor``); a decision whose anchor matched only with the
    statement terminator appended carries ``"anchor_completed": true``.
    """
    table = load_table(scope.language)
    roots: dict[str, SgNode] = {}
    accepted_sites: set[tuple] = set()
    decisions = []

    def decide(annotation: dict) -> tuple[str, bool]:
        file, kind = annotation.get("file"), annotation.get("kind")
        anchor = annotation.get("anchor")
        start, end = annotation.get("start_line"), annotation.get("end_line")
        if kind not in KINDS or not isinstance(anchor, str) or not anchor.strip() \
                or not isinstance(start, int) or not isinstance(end, int) or start < 1 or end < start:
            return "malformed", False
        normalized = str(file).replace("\\", "/")
        if is_test_file(scope, normalized):
            return "test_file", False
        if normalized not in scope.files:
            return "outside_scope", False
        if normalized not in roots:
            roots[normalized] = SgRoot(scope.source(normalized), scope.language).root()
        matches, completed = match_anchor(table, roots[normalized], start, end, anchor)
        spans = {_span(n) for n in matches}
        if not spans:
            return "anchor_mismatch", completed
        if len(spans) > 1:
            return "anchor_ambiguous", completed
        reason, _ = site_check(table, matches[0], kind)
        if reason != "ok":
            return reason, completed
        site = (normalized, spans.pop(), kind)
        if site in accepted_sites:
            return "duplicate", completed
        accepted_sites.add(site)
        return "ok", completed

    for annotation in annotations:
        reason, completed = decide(annotation)
        decision = {"annotation": annotation, "accepted": reason == "ok", "reason": reason}
        if completed:
            decision["anchor_completed"] = True
        decisions.append(decision)
    return decisions


def _tokens(root: SgNode, skip: set[str]) -> list[str]:
    out: list[str] = []

    def visit(node: SgNode) -> None:
        if node.kind() in skip:
            return
        children = node.children()
        if not children:
            out.append(node.text())
        for child in children:
            visit(child)

    visit(root)
    return out


def classify_mutant(language: str, original: str, mutated: str) -> str:
    """``changed`` for a real, parseable change; otherwise why it is invalid."""
    table = load_table(language)
    skip = set(table["comment_kinds"])
    mutated_root = SgRoot(mutated, language).root()
    if any(n.kind() == "ERROR" for n in _walk(mutated_root)):
        return "parse_error"
    if _tokens(SgRoot(original, language).root(), skip) == _tokens(mutated_root, skip):
        return "no_op"
    return "changed"
