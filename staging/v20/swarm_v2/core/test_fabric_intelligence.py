"""Intelligence test: does each model's side store retrieve the semantically-correct corpus doc?

Embeds a fixed set of queries through a running embedder (EMBED_PORT) and checks whether the
expected paper ranks top-1 / top-3 in the corresponding .turbovec store. Reports top-1 and
top-3 accuracy, plus the actual top-3 hits, so the result is verifiable, not a pass/fail blob.
"""
import json, math, os, struct, sys, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BITS = 2
LEVELS = (1 << BITS) - 1

QUERIES = [
    ("balanced ternary logic knowledge fusion", "papers/paper_148_turbovec_and_ternary_logic_knowledge_fusion_engine.md"),
    ("geometric resonance engine", "papers/paper_01_geometric_resonance.md"),
    ("voltage harmonization proof", "papers/paper_022_voltage_harmonization_proof.md"),
    ("voltage bound drug screening at scale", "papers/paper_024_voltage_bound_drug_screening_at_scale.md"),
    ("the free will directive", "papers/paper_025_the_free_will_directive.md"),
    ("five scale self knowledge", "papers/paper_051_five_scale_self_knowledge.md"),
    ("knowledge fabric RAG semantic search", "papers/paper_117_knowledge_fabric_rag_and_monad_journal_semantic_search.md"),
]


def embed(port, text):
    req = urllib.request.Request(
        "http://127.0.0.1:%s/v1/embeddings" % port,
        data=json.dumps({"input": text}).encode(),
        headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.load(r)["data"][0]["embedding"]


def load_store(name):
    d = open(os.path.join(ROOT, "swarm_v2_nand_fabric.%s.turbovec" % name), "rb").read()
    ver, anchor, dim, vectors, bits = struct.unpack_from("<IQIII", d, 8)
    keys_off, keys_len = struct.unpack_from("<QI", d, 32)
    meta = json.loads(d[keys_off:keys_off + keys_len])
    # dequantize
    off = 128
    vecs = {}
    for k in meta["keys"]:
        scale, vmin = struct.unpack_from("<ff", d, off); off += 8
        nbytes = (dim * bits + 7) // 8
        blob = d[off:off + nbytes]; off += nbytes
        codes = [(blob[i * bits // 8] >> ((i * bits) % 8)) & LEVELS for i in range(dim)]
        vecs[k] = [vmin + c * scale for c in codes]
    return dim, vecs


def topk(q, vecs, k=3):
    nq = math.sqrt(sum(x * x for x in q))
    scored = []
    for key, v in vecs.items():
        dot = sum(a * b for a, b in zip(q, v))
        nv = math.sqrt(sum(y * y for y in v))
        scored.append((dot / (nq * nv) if nq and nv else 0.0, key))
    scored.sort(reverse=True)
    return scored[:k]


def main(port, name):
    dim, vecs = load_store(name)
    print("store=%s dim=%d vectors=%d" % (name, dim, len(vecs)))
    top1 = top3 = 0
    for qtext, expected in QUERIES:
        q = embed(port, qtext)
        hits = topk(q, vecs)
        ranks = [h[1] for h in hits]
        ok1 = (ranks[0] == expected)
        ok3 = (expected in ranks)
        top1 += ok1; top3 += ok3
        print("  [%s] '%s' -> #1 %s %s" % ("OK" if ok1 else "--", qtext,
              os.path.basename(ranks[0]), "" if ok1 else "(expected %s)" % os.path.basename(expected)))
    print("top-1 accuracy: %d/%d (%.0f%%)" % (top1, len(QUERIES), 100 * top1 / len(QUERIES)))
    print("top-3 accuracy: %d/%d (%.0f%%)" % (top3, len(QUERIES), 100 * top3 / len(QUERIES)))
    return top1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2]))
