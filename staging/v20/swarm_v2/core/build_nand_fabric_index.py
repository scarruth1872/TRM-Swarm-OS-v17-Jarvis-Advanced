"""Build the NAND fabric's packed vector index from the kernel's REAL corpus.

The design (`swarm_v2/core/nand_fabric_paths.py`) calls for three stores: the Rust-owned `.k3`
trit store, a Python-owned packed vector index (`swarm_v2_nand_fabric.turbovec`), and a Python
text sidecar. The `.turbovec` store was never built -- and the script that was supposed to build
it (`ingest_sovereign_datasets.py`) writes a 57 MB SYNTHETIC byte pattern sized from the literal
241,800,000, with sine-wave "shard vectors". That fabricates a store instead of indexing one.

This builder packs what actually exists: the 768-dim vectors in
`swarm_v2_artifacts/kernel_knowledge_index.json`, produced by the kernel's own model-free encoder
(`rust_native_encode_text_f32`; meta.model = native-turbovec-hdc). No model server, no synthetic
payload, and every count written into the header is derived from the data.

Format (little-endian):
  header 128 B : b"TURBOVEC" | version u32 | anchor u64 | dim u32 | vectors u32 | bits u32 |
                 keys_off u64 | keys_len u32 | sha256(payload+keys) 32 | source char[48] | pad
  payload      : per vector  f32 scale | f32 v_min | packed codes ((dim*bits+7)//8 bytes)
  key table    : UTF-8 JSON {"source","model","dim","vectors","keys":[...]} at keys_off
"""
import hashlib, json, math, os, struct, sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(ROOT, "swarm_v2_artifacts", "kernel_knowledge_index.json")
OUT = os.path.join(ROOT, "swarm_v2_nand_fabric.turbovec")
TEXT_OUT = os.path.join(ROOT, "swarm_v2_nand_fabric.text.jsonl")
ANCHOR = 0x4D4F4E4144000001
BITS = 2
LEVELS = (1 << BITS) - 1
HEADER = 128


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
    codes = []
    for i in range(n):
        bit = i * BITS
        codes.append((blob[bit // 8] >> (bit % 8)) & LEVELS)
    return codes


def cosine(a, b):
    d = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(y * y for y in b))
    return d / (na * nb) if na and nb else 0.0


def iter_docs(path):
    """Stream the indent=2 JSON: yield (key, [768 floats]) per doc, no full load. The f32 index is
    4.49 GB at full corpus, so json.load would not fit; this reads it one document at a time."""
    with open(path, encoding="utf-8") as f:
        in_index = False
        key = None
        emb = None
        for line in f:
            s = line.rstrip("\n")
            st = s.strip()
            if not in_index:
                if st == '"index": {':
                    in_index = True
                continue
            ind = len(s) - len(s.lstrip(" "))
            if st == "}" and ind == 0:
                return
            if ind == 4 and st.endswith('": {'):
                key = st[1:st.index('": {')]
                emb = None
            elif ind == 6 and st.startswith('"embedding": ['):
                emb = []
            elif ind == 8 and emb is not None:
                emb.append(float(st.rstrip(",")))
            elif ind == 6 and st == "]," and key is not None and emb is not None:
                yield key, emb
                key = None
                emb = None


def meta_model(path):
    """Read meta.model from the head of the index without loading the whole 4.49 GB file."""
    import re
    with open(path, encoding="utf-8") as f:
        head = f.read(4096)
    m = re.search(r'"model"\s*:\s*"([^"]+)"', head)
    return m.group(1) if m else "unknown"


def main():
    if not os.path.exists(SRC):
        print("missing source corpus: %s" % SRC, file=sys.stderr)
        return 2

    # Stream the f32 index one document at a time; build payload + key table + text sidecar in the
    # SAME pass so the keys stay keyed identically across all three (MOTION-007: one owner).
    keys = []
    payload = bytearray()
    fids = []
    dim = None
    model = meta_model(SRC)
    written = unresolved = 0

    with open(TEXT_OUT, "w", encoding="utf-8") as text_fh:
        for k, emb in iter_docs(SRC):
            if dim is None:
                dim = len(emb)
            vec = [float(x) for x in emb]
            if len(vec) != dim:
                print("ragged dimension for %s" % k, file=sys.stderr)
                return 2
            codes, scale, vmin = quantize(vec)
            blob = pack(codes)
            payload += struct.pack("<ff", scale, vmin) + blob
            fids.append(cosine([vmin + c * scale for c in unpack(blob, dim)], vec))
            keys.append(k)
            p = os.path.join(ROOT, k)
            if os.path.exists(p):
                body = open(p, encoding="utf-8", errors="replace").read()
                rec = {"key": k, "bytes": len(body.encode("utf-8")),
                       "sha256": hashlib.sha256(body.encode("utf-8")).hexdigest(), "text": body}
                written += 1
            else:
                rec = {"key": k, "bytes": 0, "sha256": None, "text": None,
                       "note": "source is not a file on disk (index key without text)"}
                unresolved += 1
            text_fh.write(json.dumps(rec, sort_keys=True) + "\n")

    if not keys:
        print("source corpus is empty", file=sys.stderr)
        return 2

    table = json.dumps({"source": os.path.relpath(SRC, ROOT).replace("\\", "/"), "model": model,
                        "dim": dim, "vectors": len(keys), "bits": BITS, "keys": keys},
                       sort_keys=True).encode("utf-8")
    keys_off = HEADER + len(payload)
    src_rel = os.path.relpath(SRC, ROOT).replace("\\", "/").encode("utf-8")[:47]
    # bytes 44..76 : SHA-256 over payload + key table (a structure-only check let a flipped
    # payload byte verify "intact" -- the gate must see content, not just shape)
    checksum = hashlib.sha256(bytes(payload) + table).digest()

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
        fh.write(bytes(header))
        fh.write(bytes(payload))
        fh.write(table)

    print("wrote %s" % os.path.relpath(TEXT_OUT, ROOT))
    print("  text records : %d resolvable, %d without on-disk text" % (written, unresolved))
    print("wrote %s" % os.path.relpath(OUT, ROOT))
    print("  bytes        : %d (header %d + payload %d + keys %d)" % (
        os.path.getsize(OUT), HEADER, len(payload), len(table)))
    print("  dim / vectors: %d / %d @ %d bits (%.1fx vs f32)" % (
        dim, len(keys), BITS, (dim * 32) / (dim * BITS)))
    print("  model        : %s" % model)
    print("  round-trip cosine: min %.4f  mean %.4f  max %.4f" % (
        min(fids), sum(fids) / len(fids), max(fids)))
    print("  payload sha  : %s (stored in the header and checked on read)" % checksum.hex()[:32])
    print("  file sha256  : %s" % hashlib.sha256(open(OUT, "rb").read()).hexdigest()[:32])
    return 0


if __name__ == "__main__":
    sys.exit(main())
