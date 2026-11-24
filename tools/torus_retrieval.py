"""Run the retrieval task THROUGH the 4D torus, against the cosine baseline it must beat or admit defeat.

The kernel's structure (trm_uefi_kernel/src/faceted_torus_ternary.rs, read not guessed):
    FacetedTrit { facet: i16 in [-108,108] }        from_f32: round(clamp(v,-1,1)*108)
    axis_index() = facet + 108        (0..216)
    to_4d_torus(turn) = ( r*cos(theta), r*sin(theta), r*cos(phi), r*sin(phi) )
        theta = 2*pi*k/217 - pi,  phi = 2*pi*turn,  r = 1/sqrt(2)

So one torus point carries exactly TWO numbers: a facet and a recurrence turn. This script asks the only
question that decides whether the torus is an index substrate: can those two numbers (or 217-level facets
per dimension) stand in for a 768-dim vector when retrieving?

Ground truth = the f32 cosine ranking itself. That is a real limitation and it is stated: this measures
whether the compressed representations PRESERVE the current retriever, not whether either is relevant to a
human. Recall@10 = fraction of the baseline's top-10 that the variant also returns in its top-10.
"""
import json
import numpy as np

ROOT = r"F:/Development sites/TRM-Swarm-OS-v2"
S = r"C:/Users/shawn/AppData/Local/hermes/cache/scratch/"

d = json.load(open(f"{ROOT}/data/council_evidence_index_v3.json"))
chunks = d["chunks"]
V = np.array([c["vec"] for c in chunks], dtype=np.float32)
V /= (np.linalg.norm(V, axis=1, keepdims=True) + 1e-9)
N, D = V.shape

probes = [
    "Which TCP port does the TRM council server listen on?",
    "What is axiom A2 in the council's framework about?",
    "How many modules does the TRM kernel have?",
    "Which TCP port does the embedding server use?",
] + [t["text"] for t in json.load(open(S + "blind_prereg.json"))["tasks"]]


def embed(text):
    import urllib.request
    body = json.dumps({"model": "nomic-embed-text", "input": [text]}).encode()
    r = urllib.request.Request("http://127.0.0.1:8593/v1/embeddings", data=body,
                               headers={"Content-Type": "application/json"})
    v = np.array(json.loads(urllib.request.urlopen(r, timeout=90).read())["data"][0]["embedding"],
                 dtype=np.float32)
    return v / (np.linalg.norm(v) + 1e-9)


Q = np.array([embed(p) for p in probes], dtype=np.float32)

# --- the kernel's facet quantiser, reimplemented exactly ---
def to_facet(v):
    return np.round(np.clip(v, -1.0, 1.0) * 108.0) / 108.0


FACET_V = to_facet(V)
FACET_Q = to_facet(Q)
TRIT_V = np.sign(V)
TRIT_Q = np.sign(Q)

# --- torus placement: two numbers per vector, via the corpus's own principal axes ---
mean = V.mean(axis=0)
Vc = V - mean
Qc = Q - mean
# top-4 directions, computed on the corpus (no query leakage)
U, sv, Vt = np.linalg.svd(Vc, full_matrices=False)
PC = Vt[:4]                       # (4, D)
p_v = Vc @ PC.T                   # (N, 4)
p_q = Qc @ PC.T                   # (Q, 4)
# scale each projection to [-1,1] using corpus percentiles, then to facets and turns
lo = np.percentile(p_v, 1, axis=0)
hi = np.percentile(p_v, 99, axis=0)
sc = lambda p: np.clip(2 * (p - lo) / (hi - lo + 1e-9) - 1, -1, 1)
f_v, f_q = sc(p_v), sc(p_q)
turn_v = (f_v + 1) / 2            # 0..1, the "recurrence turn"
turn_q = (f_q + 1) / 2
theta_v = 2 * np.pi * np.round(f_v * 108) / 217 - np.pi     # axis_index quantised to 217 states
theta_q = 2 * np.pi * np.round(f_q * 108) / 217 - np.pi
phi_v = 2 * np.pi * turn_v
phi_q = 2 * np.pi * turn_q


def angdiff(a, b):
    d = np.abs(a - b) % (2 * np.pi)
    return np.minimum(d, 2 * np.pi - d)


def torus_dist(Qx, Vx, two_circles=True):
    """Geodesic on S1 x S1 per torus point; sum over the pairs of circles used."""
    out = np.zeros((Qx.shape[0], Vx.shape[0]), dtype=np.float32)
    pairs = [(0, 2), (1, 3)] if two_circles else [(0, 2)]
    for a, b in pairs:
        out += angdiff(Qx[:, a][:, None], Vx[:, a][None, :]) if False else 0  # placeholder
    return out


def torus_distance_matrix(qang, vang):
    """qang, vang: (M, C) and (N, C) angle matrices; sum of circular distances over channels."""
    Dm = np.zeros((qang.shape[0], vang.shape[0]), dtype=np.float32)
    for c in range(qang.shape[1]):
        s = angdiff(qang[:, c][:, None], vang[:, c][None, :])
        Dm += s
    return Dm


def recall_at_k(scores, k=10):
    """scores: (Q, N) higher-is-better. Baseline = cosine."""
    base = np.argsort(-(Q @ V.T), axis=1)[:, :k]
    got = np.argsort(-scores, axis=1)[:, :k]
    rec = [len(set(b) & set(g)) / k for b, g in zip(base, got)]
    top1 = [b[0] == g[0] for b, g in zip(base, got)]
    return float(np.mean(rec)), float(np.mean(top1))


print(f"  index: {N} chunks, {D} dims, {len(probes)} queries\n")
print(f"  {'representation':46} {'recall@10':>9} {'top-1':>7}")
print(f"  {'f32 cosine (baseline, 768 floats/chunk)':46} {'1.000':>9} {'1.000':>7}")

r, t = recall_at_k((FACET_Q @ FACET_V.T), 10)
print(f"  {'facet 217-level per dim (FacetedTrit, 768 int8)':46} {r:9.3f} {t:7.3f}")

r, t = recall_at_k((TRIT_Q @ TRIT_V.T), 10)
print(f"  {'trit 3-level per dim (K3 ternary, 768 trits)':46} {r:9.3f} {t:7.3f}")

# torus: 1 point = (theta from PC1, phi from PC2) -> 2 numbers
D1 = torus_distance_matrix(np.stack([theta_q[:, 0], phi_q[:, 1]], 1),
                           np.stack([theta_v[:, 0], phi_v[:, 1]], 1))
r, t = recall_at_k(-D1, 10)
print(f"  {'torus, 1 point (2 numbers: facet+turn)':46} {r:9.3f} {t:7.3f}")

# torus: 2 points = PCs 1-2 and 3-4 -> 4 numbers
A_q = np.stack([theta_q[:, 0], phi_q[:, 1], theta_q[:, 2], phi_q[:, 3]], 1)
A_v = np.stack([theta_v[:, 0], phi_v[:, 1], theta_v[:, 2], phi_v[:, 3]], 1)
D2 = torus_distance_matrix(A_q, A_v)
r, t = recall_at_k(-D2, 10)
print(f"  {'torus, 2 points (4 numbers) + 217-state facets':46} {r:9.3f} {t:7.3f}")

# unquantised 4-number projection, to separate "few numbers" from "quantisation"
r, t = recall_at_k(-np.linalg.norm(p_q[:, None, :2] - p_v[None, :, :2], axis=2), 10)
print(f"  {'same 2 PCs, unquantised euclidean (control)':46} {r:9.3f} {t:7.3f}")
r, t = recall_at_k((sc(p_q)[:, :] @ sc(p_v)[:, :].T), 10)
print(f"  {'all 4 PCs, unquantised dot (control)':46} {r:9.3f} {t:7.3f}")
