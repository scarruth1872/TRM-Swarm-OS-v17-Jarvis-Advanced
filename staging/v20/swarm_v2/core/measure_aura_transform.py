"""Measure the AURA transform's retrieval quality against the same 7-query benchmark
used for the Phi-4 / Bonsai embedding stores, so the comparison is apples-to-apples.

The AURA transform is keyword -> metaconcept scoring. For each doc and query we build a
100-dim metaconcept score vector (the full score, not just top-k), then cosine-rank docs
against the query. Faithful reimplementation of find_relevant_metaconcepts from
swarm_v2/core/aura_cognition.py (same scoring constants, same boosts), parity-checked
against the live Rust /aura/cognition/branch endpoint.
"""
import ast, json, math, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(ROOT, "swarm_v2", "core", "aura_cognition.py")
CORPUS = os.path.join(ROOT, "swarm_v2_artifacts", "kernel_knowledge_index.json")

# ---- extract metaconcepts (same ast approach as the generator) ----
tree = ast.parse(open(SRC, encoding="utf-8").read())
metas = None
for node in tree.body:
    if isinstance(node, ast.Assign):
        for t in node.targets:
            if isinstance(t, ast.Name) and t.id == "SOVEREIGN_100_METACONCEPTS":
                metas = ast.literal_eval(node.value)
    elif isinstance(node, ast.AnnAssign):
        if isinstance(node.target, ast.Name) and node.target.id == "SOVEREIGN_100_METACONCEPTS":
            metas = ast.literal_eval(node.value)
assert metas, "metaconcepts not found"
KEYS = [m["key"] for m in metas]

QUERIES = [
    ("balanced ternary logic knowledge fusion", "papers/paper_148_turbovec_and_ternary_logic_knowledge_fusion_engine.md"),
    ("geometric resonance engine", "papers/paper_01_geometric_resonance.md"),
    ("voltage harmonization proof", "papers/paper_022_voltage_harmonization_proof.md"),
    ("voltage bound drug screening at scale", "papers/paper_024_voltage_bound_drug_screening_at_scale.md"),
    ("the free will directive", "papers/paper_025_the_free_will_directive.md"),
    ("five scale self knowledge", "papers/paper_051_five_scale_self_knowledge.md"),
    ("knowledge fabric RAG semantic search", "papers/paper_117_knowledge_fabric_rag_and_monad_journal_semantic_search.md"),
]


def score_vector(text):
    """Full 100-dim metaconcept score vector, mirroring find_relevant_metaconcepts."""
    t_low = text.lower()
    vec = [0.0] * len(metas)
    for i, m in enumerate(metas):
        s = 0.0
        if m["key"].lower() in t_low:
            s += 10.0
        if m["name"].lower() in t_low:
            s += 8.0
        if m["category"].lower() in t_low:
            s += 3.0
        for w in m["description"].lower().split():
            if len(w) > 4 and w in t_low:
                s += 1.5
        for it in m["intent"]:
            if it.lower() in t_low:
                s += 2.5
        d = m["description"].lower()
        if "voltage" in t_low and "voltage" in d:
            s += 4.0
        if "ternary" in t_low and ("ternary" in d or "k3" in d):
            s += 4.0
        if "resonance" in t_low and "resonance" in d:
            s += 3.0
        vec[i] = s
    return vec


def cosine(a, b):
    dot = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(y * y for y in b))
    return dot / (na * nb) if na and nb else 0.0


def main():
    idx = json.load(open(CORPUS, encoding="utf-8"))
    rows = idx.get("index", idx)
    keys = sorted(rows)

    # doc vectors (full text up to 5000 chars — the AURA transform has no context limit)
    doc_vecs = {}
    for k in keys:
        p = os.path.join(ROOT, k)
        t = open(p, encoding="utf-8", errors="replace").read() if os.path.exists(p) else " "
        doc_vecs[k] = score_vector(t[:5000])

    print("docs:", len(doc_vecs), "metaconcepts:", len(metas))
    top1 = top3 = 0
    for qtext, expected in QUERIES:
        qv = score_vector(qtext)
        scored = [(cosine(qv, doc_vecs[k]), k) for k in doc_vecs]
        scored.sort(key=lambda x: x[0], reverse=True)
        ranks = [k for _, k in scored[:3]]
        ok1 = ranks[0] == expected
        ok3 = expected in ranks
        top1 += ok1; top3 += ok3
        print("  [%s] '%s' -> #1 %s %s" % (
            "OK" if ok1 else "--", qtext,
            os.path.basename(ranks[0]),
            "" if ok1 else "(expected %s)" % os.path.basename(expected)))
    print("top-1 accuracy: %d/%d (%.0f%%)" % (top1, len(QUERIES), 100 * top1 / len(QUERIES)))
    print("top-3 accuracy: %d/%d (%.0f%%)" % (top3, len(QUERIES), 100 * top3 / len(QUERIES)))


if __name__ == "__main__":
    main()
