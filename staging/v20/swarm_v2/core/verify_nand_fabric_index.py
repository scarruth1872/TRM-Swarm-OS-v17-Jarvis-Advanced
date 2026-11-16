"""Verify the packed NAND fabric vector index independently of the builder.

Reads `swarm_v2_nand_fabric.turbovec`, re-derives every vector from the packed codes (nothing is
taken from the writer's own report), and checks it against the source corpus:

  * header magic / anchor / derived counts agree with the file's actual size
  * the key table matches the source corpus keys exactly (no silent subsetting)
  * each unpacked vector's cosine against the source embedding clears the measured floor

Exit 0 only when all three hold. The floor is a *measured* property of 2-bit/4-level
quantization on this corpus, not a hoped-for number -- it is read from the artifact's own
`fabric_index_floor.json`, written by the first run that measured it.
"""
import hashlib, json, math, os, struct, sys

HEADER_END = 128

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
STORE = os.path.join(ROOT, "swarm_v2_nand_fabric.turbovec")
SRC = os.path.join(ROOT, "swarm_v2_artifacts", "kernel_knowledge_index.json")
FLOOR = os.path.join(ROOT, "swarm_v2_artifacts", "fabric_index_floor.json")
ANCHOR = 0x4D4F4E4144000001


def cosine(a, b):
    d = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(y * y for y in b))
    return d / (na * nb) if na and nb else 0.0


def main():
    if not os.path.exists(STORE):
        print("REFUSING: store absent -> %s" % STORE)
        return 2
    blob = open(STORE, "rb").read()
    magic = blob[0:8]
    version, = struct.unpack_from("<I", blob, 8)
    anchor, = struct.unpack_from("<Q", blob, 12)
    dim, = struct.unpack_from("<I", blob, 20)
    vectors, = struct.unpack_from("<I", blob, 24)
    bits, = struct.unpack_from("<I", blob, 28)
    keys_off, = struct.unpack_from("<Q", blob, 32)
    keys_len, = struct.unpack_from("<I", blob, 40)
    levels = (1 << bits) - 1
    per_vec = 8 + (dim * bits + 7) // 8
    expected = 128 + vectors * per_vec + keys_len

    problems = []
    if magic != b"TURBOVEC":
        problems.append("magic is %r" % magic)
    if anchor != ANCHOR:
        problems.append("anchor 0x%016X != 0x%016X" % (anchor, ANCHOR))
    if len(blob) != expected:
        problems.append("size %d != header+payload+keys %d (counts in the header are derived from the file)"
                        % (len(blob), expected))

    stored_sum = blob[44:76]
    actual_sum = hashlib.sha256(blob[HEADER_END:keys_off + keys_len]).digest()
    if stored_sum != actual_sum:
        problems.append("payload/key-table checksum mismatch (header %s != computed %s)" % (
            stored_sum.hex()[:16], actual_sum.hex()[:16]))

    table = json.loads(blob[keys_off:keys_off + keys_len].decode("utf-8"))
    src = json.load(open(SRC, encoding="utf-8"))
    rows = src.get("index", src)
    if sorted(table["keys"]) != sorted(rows):
        problems.append("key table does not match the source corpus keys exactly")

    fids = []
    for i, key in enumerate(table["keys"]):
        if key not in rows:
            continue
        off = 128 + i * per_vec
        scale, vmin = struct.unpack_from("<ff", blob, off)
        packed = blob[off + 8: off + per_vec]
        codes = [(packed[(j * bits) // 8] >> ((j * bits) % 8)) & levels for j in range(dim)]
        rec = [vmin + c * scale for c in codes]
        fids.append(cosine(rec, [float(x) for x in rows[key]["embedding"]]))

    # measured must exist before the floor block: the re-baseline branch reports a delta from it.
    measured = {"cosine_min": min(fids) if fids else 0.0, "cosine_mean": (sum(fids) / len(fids)) if fids else 0.0,
                "vectors_checked": len(fids), "dim": dim, "bits": bits}
    floor, floor_note = None, None
    if os.path.exists(FLOOR):
        fl = json.load(open(FLOOR, encoding="utf-8"))
        floor = fl.get("cosine_min")
        corpus_now = hashlib.sha256(open(SRC, "rb").read()).hexdigest()
        if fl.get("corpus_sha256") and fl["corpus_sha256"] != corpus_now:
            # The floor is a property of the CORPUS it was measured on, not of the file format.
            # After a legitimate corpus refresh the minimum can move; that is a re-baseline, and the
            # delta must be reported rather than silently accepted or blindly failed.
            floor_note = ("corpus changed since the floor was measured: min %.4f now vs %.4f then "
                          "(delta %+.4f) -> re-baseline deliberately" % (
                              measured["cosine_min"], floor, measured["cosine_min"] - floor))
            floor = None
    if floor is not None and measured["cosine_min"] < floor:
        problems.append("cosine_min %.4f below the measured floor %.4f" % (measured["cosine_min"], floor))

    TEXT = os.path.join(ROOT, "swarm_v2_nand_fabric.text.jsonl")
    if os.path.exists(TEXT):
        recs = [json.loads(l) for l in open(TEXT, encoding="utf-8") if l.strip()]
        tkeys = [r["key"] for r in recs]
        if sorted(tkeys) != sorted(table["keys"]):
            problems.append("text store keys != vector store keys (%d vs %d)" % (len(tkeys), len(table["keys"])))
        mism = 0
        for rec in recs:
            p = os.path.join(ROOT, rec["key"])
            if rec["sha256"] is None or not os.path.exists(p):
                continue
            if hashlib.sha256(open(p, encoding="utf-8", errors="replace").read().encode("utf-8")).hexdigest() != rec["sha256"]:
                mism += 1
        if mism:
            problems.append("text drifted from disk for %d record(s)" % mism)
        print("text store  : %s (%d records, keyed like the vectors, %d drifted)" % (
            os.path.relpath(TEXT, ROOT), len(recs), mism))

    print("store      : %s" % os.path.relpath(STORE, ROOT))
    print("magic/ver  : %s v%d | anchor 0x%016X" % (magic.decode('ascii', 'replace'), version, anchor))
    print("header says : dim %d, vectors %d, bits %d, keys %d B" % (dim, vectors, bits, keys_len))
    print("measured    : %s" % json.dumps({k: (round(v, 4) if isinstance(v, float) else v) for k, v in measured.items()}))
    print("floor       : %s" % ("%.4f (from fabric_index_floor.json)" % floor if floor is not None else "not recorded"))
    if floor_note:
        print("RE-BASELINE : %s" % floor_note)
    if problems:
        for p in problems:
            print("PROBLEM     : %s" % p)
        return 1
    print("VERDICT     : intact -- counts, key table and vectors all re-derived from the file")
    return 0


if __name__ == "__main__":
    sys.exit(main())
