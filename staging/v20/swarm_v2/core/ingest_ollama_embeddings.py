"""Ingest a local Ollama embedding model into a .turbovec NAND-fabric store.

Same corpus + same 2-bit TurboVec pack + same 128-byte header as
build_nand_fabric_index.py / ingest_real_model_embeddings.py, so the Rust
packed_ternary reader loads the result unchanged. The only difference: the
embedding source is Ollama's /api/embed (model passed on the command line),
because the embedders ready to ingest are Ollama-pulled, not llama.cpp-served.

Usage: python ingest_ollama_embeddings.py <ollama-model> <out-name> [limit]
"""
import hashlib, json, math, os, struct, sys, time, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(ROOT, "swarm_v2_artifacts", "kernel_knowledge_index.json")
OLLAMA = "http://127.0.0.1:11434/api/embed"
ANCHOR = 0x4D4F4E4144000001
BITS = 2
LEVELS = (1 << BITS) - 1
HEADER = 128
BATCH = 1


def embed(text, model):
    req = urllib.request.Request(
        OLLAMA, data=json.dumps({"model": model, "input": text}).encode(),
        headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as r:
        data = json.load(r)
    emb = data.get("embeddings") or []
    if not emb:
        raise RuntimeError("no embedding returned")
    return emb[0]


def quantize(vec):
    vmin, vmax = min(vec), max(vec)
    span = vmax - vmin
    scale = (span / LEVELS) if span else 1.0
    codes = []
    for x in vec:
        c = 0 if not span else int(round((x - vmin) / scale))
        codes.append(0 if c < 0 else (LEVELS if c > LEVELS else c))
    return codes, float(scale), float(vmin)


def pack(codes):
    out = bytearray((len(codes) * BITS + 7) // 8)
    for i, c in enumerate(codes):
        bit = i * BITS
        out[bit // 8] |= (c & LEVELS) << (bit % 8)
    return bytes(out)


def unpack(blob, n):
    return [(blob[i * BITS // 8] >> ((i * BITS) % 8)) & LEVELS for i in range(n)]


def cosine(a, b):
    d = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(y * y for y in b))
    return d / (na * nb) if na and nb else 0.0


def main():
    model = sys.argv[1]
    out_name = sys.argv[2]
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else None
    OUT = os.path.join(ROOT, "swarm_v2_nand_fabric.%s.turbovec" % out_name)

    src = json.load(open(SRC, encoding="utf-8"))
    rows = src.get("index", src)
    keys = sorted(rows)
    if limit:
        keys = keys[:limit]
    print("corpus keys:", len(keys), "model:", model, "->", os.path.basename(OUT))

    texts = []
    for k in keys:
        p = os.path.join(ROOT, k)
        if os.path.exists(p):
            t = open(p, encoding="utf-8", errors="replace").read()
        else:
            t = " "
        if not t.strip():
            t = " "
        texts.append(t[:1200])

    embs = []
    t0 = time.time()
    for i, t in enumerate(texts):
        got = None
        for cut in (1200, 800, 400, 200):
            try:
                embs.append(embed(t[:cut], model))
                got = True
                break
            except Exception:
                continue
        if not got:
            print("embed failed at doc %d (even at 200 chars)" % i, file=sys.stderr)
            return 2
        if i and i % 256 == 0:
            print("  embedded %d/%d (%.1fs)" % (i, len(texts), time.time() - t0))
    dim = len(embs[0]) if embs else 0
    print("dim:", dim, "elapsed %.1fs" % (time.time() - t0))

    payload = bytearray()
    fids = []
    for vec in embs:
        codes, scale, vmin = quantize(vec)
        payload += struct.pack("<ff", scale, vmin) + pack(codes)
        fids.append(cosine([vmin + c * scale for c in unpack(pack(codes), dim)], vec))

    table = json.dumps({"source": os.path.relpath(SRC, ROOT).replace("\\", "/"),
                        "model": model, "dim": dim, "vectors": len(keys),
                        "bits": BITS, "keys": keys}, sort_keys=True).encode("utf-8")
    keys_off = HEADER + len(payload)
    checksum = hashlib.sha256(bytes(payload) + table).digest()
    src_rel = os.path.relpath(SRC, ROOT).replace("\\", "/").encode("utf-8")[:47]

    header = bytearray(HEADER)
    header[0:8] = b"TURBOVEC"
    struct.pack_into("<I", header, 8, 2)
    struct.pack_into("<Q", header, 12, ANCHOR)
    struct.pack_into("<I", header, 20, dim)
    struct.pack_into("<I", header, 24, len(keys))
    struct.pack_into("<I", header, 28, BITS)
    struct.pack_into("<Q", header, 32, keys_off)
    struct.pack_into("<I", header, 40, len(table))
    header[44:76] = checksum
    header[76:76 + len(src_rel)] = src_rel

    with open(OUT, "wb") as fh:
        fh.write(bytes(header) + bytes(payload) + table)

    print("wrote", os.path.relpath(OUT, ROOT))
    print("  bytes        : %d (%.1f KB)" % (os.path.getsize(OUT), os.path.getsize(OUT) / 1024))
    print("  dim / vectors: %d / %d @ %d bits (%.1fx vs f32)" % (dim, len(keys), BITS, 32 / BITS))
    print("  round-trip cosine: min %.4f  mean %.4f  max %.4f" % (
        min(fids), sum(fids) / len(fids), max(fids)))
    print("  file sha256  :", hashlib.sha256(open(OUT, "rb").read()).hexdigest()[:32])
    return 0


if __name__ == "__main__":
    sys.exit(main())
