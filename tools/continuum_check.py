"""Does the Rust continuum retrieve as well as the f32 index? Two measurements, both Rust-served.

A) the four platform-fact probes, through GET /continuum/search (Rust, i8 facets, i32 dot)
B) self-retrieval on 60 held-out chunks: the query is the chunk's own first sentence, the target is that
   chunk. This is a harder, blind-by-construction test of the ranking geometry than the 16 tuning probes,
   because no probe was ever chosen with the index in view. Ground truth for B is the chunk id itself, not
   agreement with another retriever.

Limitation stated: B's queries are the documents' own words, so lexical overlap helps; it tests the
representation's ability to keep a chunk nearest to its own text, not user-phrased recall.
"""
import json
import random
import urllib.parse
import urllib.request

BASE = "http://127.0.0.1:8022"
ROOT = r"F:/Development sites/TRM-Swarm-OS-v2"


def search(q, k=10):
    u = f"{BASE}/continuum/search?q={urllib.parse.quote(q)}&k={k}"
    return json.loads(urllib.request.urlopen(u, timeout=90).read())


print("  == A) platform-fact probes through the Rust continuum ==")
probes = [
    ("Which TCP port does the TRM council server listen on?", "8022"),
    ("What is axiom A2 in the council's framework about?", "free will"),
    ("How many modules does the TRM kernel have?", "322"),
    ("Which TCP port does the embedding server use?", "8593"),
]
a_hits = 0
for q, key in probes:
    d = search(q, 10)
    rs = d.get("results") or []
    top = (rs[0]["text"].lower() if rs else "")
    ok = key.lower() in top
    a_hits += ok
    print(f"    {'PASS' if ok else 'miss'}  {q[:50]:52} cell={rs[0]['cell'] if rs else '-'} "
          f"score={rs[0]['score'] if rs else 0:>8}  {rs[0]['source'][:26] if rs else d.get('error','')}")

print("\n  == B) self-retrieval: 60 held-out chunks, query = own first sentence ==")
chunks = [json.loads(l) for l in open(f"{ROOT}/data/continuum_v3_chunks.jsonl", encoding="utf-8")]
random.seed(20261124)
sample = random.sample(range(len(chunks)), 60)
r1 = r10 = 0
for i in sample:
    text = chunks[i]["text"].strip().replace("\n", " ")
    q = text.split(". ")[0][:180]
    if len(q) < 25:
        continue
    d = search(q, 10)
    ids = [r["i"] for r in (d.get("results") or [])]
    r1 += (bool(ids) and ids[0] == i)
    r10 += (i in ids)
print(f"    recall@1  {r1}/60 = {r1/60:.3f}")
print(f"    recall@10 {r10}/60 = {r10/60:.3f}")

print(f"\n  A) {a_hits}/4 platform facts via the Rust path (the f32 path scored 4/4 on index v3)")
