"""Measure top-1/top-3 retrieval for every ingested store, each queried through ITS OWN encoder
(Ollama /api/embed for the Ollama embedders, llama.cpp /v1/embeddings for Phi-4). One honest
gradient table instead of per-store pass/fail blobs."""
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


def ollama_embed(model, text):
    req = urllib.request.Request(
        "http://127.0.0.1:11434/api/embed",
        data=json.dumps({"model": model, "input": text}).encode(),
        headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as r:
        return json.load(r)["embeddings"][0]


def llama_embed(port, text):
    req = urllib.request.Request(
        "http://127.0.0.1:%s/v1/embeddings" % port,
        data=json.dumps({"input": text}).encode(),
        headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.load(r)["data"][0]["embedding"]


def load_store(name):
    d = open(os.path.join(ROOT, "swarm_v2_nand_fabric.%s.turbovec" % name), "rb").read()
    _, _, dim, vectors, bits = struct.unpack_from("<IQIII", d, 8)
    keys_off, keys_len = struct.unpack_from("<QI", d, 32)
    meta = json.loads(d[keys_off:keys_off + keys_len])
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
    return [k for _, k in scored[:k]]


def measure(name, enc):
    dim, vecs = load_store(name)
    top1 = top3 = 0
    for qtext, expected in QUERIES:
        q = enc(qtext)
        hits = topk(q, vecs)
        top1 += hits[0] == expected
        top3 += expected in hits
    return name, dim, top1, top3


def main():
    ollama_stores = [
        ("nomic", "nomic-embed-text"),
        ("allminilm", "all-minilm"),
        ("mxbai", "mxbai-embed-large"),
        ("qwen3emb", "qwen3-embedding:0.6b"),
        ("embgemma", "embeddinggemma"),
        ("nomicv2", "nomic-embed-text-v2-moe"),
    ]
    rows = []
    for name, model in ollama_stores:
        try:
            rows.append(measure(name, lambda t, m=model: ollama_embed(m, t)))
        except Exception as e:
            print("  [skip] %s: %s" % (name, e))
    try:
        rows.append(measure("realmodel", lambda t: llama_embed(8588, t)))
    except Exception as e:
        print("  [skip] realmodel: %s" % e)
    print("store        dim   top-1   top-3")
    for name, dim, t1, t3 in rows:
        print("%-11s %-5d %d/7 %3.0f%%  %d/7 %3.0f%%" % (name, dim, t1, 100*t1/7, t3, 100*t3/7))


if __name__ == "__main__":
    main()
