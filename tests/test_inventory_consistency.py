"""Inventory consistency: the graph and the docs must agree with the trace.

G12 exists because module_dependency_graph.json claimed 170 modules
"SEALED_AND_VERIFIED" while its entries were function names; only 43/170
resolved to a real .py file. These tests make that drift fail loudly.

The trace (evidence/MODULE_TRACE.json) is the authority. If it is absent the
tests skip rather than inventing a result.
"""
import json
import pathlib
import re

import pytest

ROOT = pathlib.Path(__file__).resolve().parent.parent
TRACE_PATH = ROOT / "evidence" / "MODULE_TRACE.json"
GRAPH_PATH = ROOT / "module_dependency_graph.json"
LEGACY_PATH = ROOT / "module_dependency_graph.legacy.json"
TRACE_DOC = "docs/CODEBASE-COMPONENT-TRACE.md"
BANNER_MARKER = "SUPERSEDED — NOT AUTHORITATIVE"

# Documents that make inventory claims. The trace document is the authority
# these are checked against, not another claim to audit.
INVENTORY_DOCS = ("COMPONENT_INVENTORY.md", "SYSTEM_MAP.md")

TOTAL_MODULE_PATTERNS = (
    re.compile(r"Total Python modules:\*\*\s*([\d,]+)"),
    re.compile(r"([\d,]+)\s+Python modules"),
    re.compile(r"\|\s*Modules\s*\|\s*\**([\d,]+)\**\s*\|"),
    re.compile(r'"total_modules"\s*:\s*(\d+)'),
)


def _load_trace():
    if not TRACE_PATH.exists():
        pytest.skip(f"trace absent: {TRACE_PATH}")
    return json.loads(TRACE_PATH.read_text(encoding="utf-8"))


def _module_file(name: str):
    """Resolve a dotted module name to a real .py file, or None."""
    if name == "__init__":
        candidate = ROOT / "__init__.py"
        return candidate if candidate.is_file() else None
    rel = name.replace(".", "/")
    for candidate in (ROOT / (rel + ".py"), ROOT / rel / "__init__.py"):
        if candidate.is_file():
            return candidate
    return None


def _claims(text: str):
    found = []
    for pattern in TOTAL_MODULE_PATTERNS:
        for match in pattern.finditer(text):
            found.append(int(match.group(1).replace(",", "")))
    return found


def test_graph_exists_and_names_its_source():
    trace = _load_trace()
    assert GRAPH_PATH.is_file(), f"missing {GRAPH_PATH}"
    graph = json.loads(GRAPH_PATH.read_text(encoding="utf-8"))
    assert graph.get("generated_by") == "tools/build_module_graph.py"
    assert "MODULE_TRACE.json" in graph.get("source", "")
    assert graph.get("source_generated_by") == trace.get("generated_by")


def test_every_module_name_resolves_to_a_real_py_file():
    _load_trace()
    graph = json.loads(GRAPH_PATH.read_text(encoding="utf-8"))
    modules = graph.get("modules", {})
    assert modules, "graph has no modules"

    unresolved = []
    mismatched = []
    for name, record in modules.items():
        path = _module_file(name)
        if path is None:
            unresolved.append(name)
            continue
        declared = ROOT / record["path"]
        if declared.resolve() != path.resolve():
            mismatched.append((name, record["path"], str(path.relative_to(ROOT))))

    assert not unresolved, (
        f"{len(unresolved)}/{len(modules)} module names do not resolve to a "
        f"real .py file: {unresolved[:20]}"
    )
    assert not mismatched, f"path field disagrees with name: {mismatched[:10]}"
    assert graph["resolution"]["resolved"] == len(modules)
    assert graph["resolution"]["unresolved"] == []


def test_graph_module_count_matches_trace():
    trace = _load_trace()
    graph = json.loads(GRAPH_PATH.read_text(encoding="utf-8"))
    py = trace["python"]

    assert graph["total_modules"] == py["modules"]
    assert len(graph["modules"]) == py["modules"]
    assert graph["counts"]["reachable"] == py["reachable"]
    assert graph["counts"]["unreachable"] == len(py["unreachable"])


def test_every_edge_endpoint_is_a_known_module():
    _load_trace()
    graph = json.loads(GRAPH_PATH.read_text(encoding="utf-8"))
    modules = set(graph["modules"])
    dangling = [(src, tgt) for src, tgt in graph["edges"]
                if src not in modules or tgt not in modules]
    assert not dangling, f"{len(dangling)} dangling edges: {dangling[:10]}"


def test_superseded_docs_carry_a_banner():
    _load_trace()
    for rel in INVENTORY_DOCS:
        path = ROOT / rel
        assert path.is_file(), f"missing {rel}"
        head = path.read_text(encoding="utf-8")[:2000]
        assert BANNER_MARKER in head, f"{rel} lacks a superseded banner"
        assert "CODEBASE-COMPONENT-TRACE" in head, (
            f"{rel} banner does not name the authoritative trace"
        )

    legacy = json.loads(LEGACY_PATH.read_text(encoding="utf-8"))
    banner = legacy.get("superseded")
    assert isinstance(banner, dict), "legacy banner must be a JSON field"
    assert banner.get("status") == "NOT_AUTHORITATIVE"
    assert banner.get("superseded_by") == TRACE_DOC
    # The original body is retained below the banner, including its own status.
    assert legacy.get("status") == "SEALED_AND_VERIFIED"
    assert legacy.get("total_modules") == 170


def test_no_inventory_doc_claims_a_contradictory_module_count():
    trace = _load_trace()
    expected = trace["python"]["modules"]

    for rel in INVENTORY_DOCS:
        text = (ROOT / rel).read_text(encoding="utf-8")
        claims = _claims(text)
        if not claims:
            continue
        superseded = BANNER_MARKER in text[:2000]
        if superseded:
            # Errata: the original numbers are retained but explicitly marked
            # non-authoritative, so they are not current claims.
            continue
        wrong = [c for c in claims if c != expected]
        assert not wrong, (
            f"{rel} claims module count(s) {wrong}; trace says {expected}"
        )

    graph = json.loads(GRAPH_PATH.read_text(encoding="utf-8"))
    assert graph["total_modules"] == expected

    legacy = json.loads(LEGACY_PATH.read_text(encoding="utf-8"))
    banner = legacy.get("superseded")
    if not (isinstance(banner, dict) and
            banner.get("status") == "NOT_AUTHORITATIVE"):
        assert legacy.get("total_modules") == expected
