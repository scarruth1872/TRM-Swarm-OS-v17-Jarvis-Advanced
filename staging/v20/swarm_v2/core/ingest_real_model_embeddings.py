"""Ingest REAL-model embeddings into the NAND fabric .turbovec store.

build_nand_fabric_index.py packs the kernel's own model-free encoder
(native-turbovec-hdc, 768-dim, sovereign_zero_llm). This script ingests the SAME
corpus through a real running model's /v1/embeddings endpoint, then TurboVec-packs
(2-bit, 16x) into a NEW .turbovec store -- so the fabric holds actual distilled
model intelligence, not the zero-LLM encoder, without destroying the existing store.

Reuses the exact .turbovec format (128B header + payload + key table) from
build_nand_fabric_index.py, only the source embeddings and dim change.
"""
import hashlib, json, math, os, struct, sys, time, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(ROOT, "swarm_v2_artifacts", "kernel_knowledge_index.json")
OUT = os.path.join(ROOT, "swarm_v2_nand_fabric.%s.turbovec" % os.environ.get("OUT_NAME", "realmodel"))
EMBED_URL = "http://127.0.0.1:%s/v1/embeddings" % os.environ.get("EMBED_PORT", "8588")
ANCHOR = 0x4D4F4E4144000001
BITS = 2
LEVELS = (1 << BITS) - 1
HEADER = 128
BATCH = 1


def embed(texts):
    req = urllib.request.Request(
        EMBED_URL, data=json.dumps({"input": texts}).encode(),
        headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as r:
        data = json.load(r)
    return [d["embedding"] for d in data["data"]]


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


def main(limit=None):
    src = json.load(open(SRC, encoding="utf-8"))
    rows = src.get("index", src)
    keys = sorted(rows)
    if limit:
        keys = keys[:limit]
    print("corpus keys:", len(keys))

    # pull real text for each key (same resolution as the zero-LLM builder)
    texts = []
    for k in keys:
        p = os.path.join(ROOT, k)
        if os.path.exists(p):
            t = open(p, encoding="utf-8", errors="replace").read()
        else:
            t = " "  # missing on disk -> embed a blank (honest: no content, no signal)
        if not t.strip():
            t = " "  # empty file -> the 1.7B embedder rejects '' with HTTP 400
        texts.append(t[:1200])  # Phi-4 embedder context; 1200 chars safe, retry shorter below

    # embed in batches (single, with progressive truncation on context overflow)
    embs = []
    t0 = time.time()
    for i in range(0, len(texts), BATCH):
        chunk = texts[i:i + BATCH]
        got = None
        for cut in (1200, 800, 400, 200):
            try:
                embs.extend(embed([c[:cut] for c in chunk]))
                got = True
                break
            except Exception:
                continue
        if not got:
            print("embed failed at batch %d (even at 200 chars)" % i, file=sys.stderr)
            return 2
        if i % 256 == 0:
            print("  embedded %d/%d (%.1fs)" % (i + len(chunk), len(texts), time.time() - t0))
    dim = len(embs[0]) if embs else 0
    print("dim:", dim, "elapsed %.1fs" % (time.time() - t0))

    # quantize + pack
    payload = bytearray()
    fids = []
    for k, vec in zip(keys, embs):
        codes, scale, vmin = quantize(vec)
        payload += struct.pack("<ff", scale, vmin) + pack(codes)
        fids.append(cosine([vmin + c * scale for c in unpack(pack(codes), dim)], vec))

    # model name from the endpoint's own props
    try:
        with urllib.request.urlopen("http://127.0.0.1:%s/props" % os.environ.get("EMBED_PORT", "8588"), timeout=10) as r:
            props = json.load(r)
        model = props.get("model_path", "unknown")
    except Exception:
        model = "unknown"

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
    print("  model        :", model)
    print("  round-trip cosine: min %.4f  mean %.4f  max %.4f" % (
        min(fids), sum(fids) / len(fids), max(fids)))
    print("  file sha256  :", hashlib.sha256(open(OUT, "rb").read()).hexdigest()[:32])
    return 0


if __name__ == "__main__":
    lim = int(sys.argv[1]) if len(sys.argv) > 1 else None
    sys.exit(main(lim))
