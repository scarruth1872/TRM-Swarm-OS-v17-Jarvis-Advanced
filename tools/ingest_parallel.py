"""ingest_parallel.py - same shard format, but descriptors + encoding + quantisation all run in worker processes.

Why this exists: `k3_ingest_real.py` is single-threaded and RDKit measures 783 mol/s on one core, so the full
2,897,805-record ChEMBL pass projects to ~62 min while the box has 12C/24T idle. The serial work is real work,
not I/O - it is a good candidate for processes (not threads: RDKit and ctypes calls hold the GIL).

Design decisions that matter:
  * Workers do the WHOLE per-record job (SMILES -> RDKit descriptors -> text -> encode -> 2-bit codes) so the
    parent never becomes the bottleneck. If the parent still encoded, parallel descriptors would only move the
    wall to the encoder.
  * `imap` (ordered), not `imap_unordered`: record order must match the source file or every downstream offset
    is wrong.
  * The shard header is byte-identical to `k3_ingest_real.py`'s, so `--verify` on that tool checks this output.
  * Text formatting is `str(round(x,3))` and the descriptor set is the one measured against the fabric (G206).
  * FAIL-CLOSED: an unparseable SMILES is counted and reported, never silently replaced.
"""
import argparse, ctypes, hashlib, json, math, os, struct, sys, time
from multiprocessing import Pool

DIM = 768
BITS = 2
CODES_PER_BYTE = 8 // BITS        # 4
PACK = (DIM * BITS + 7) // 8      # 192
MONAD_ANCHOR = 0x4D4F4E4144000001
SHARD_ID = 107

_LIB = None
_LIB_PATH = None


def _load(dll):
    global _LIB, _LIB_PATH
    if _LIB is None or _LIB_PATH != dll:
        lib = ctypes.CDLL(dll)
        lib.rust_native_encode_text_f32.argtypes = [ctypes.c_char_p, ctypes.c_size_t,
                                                   ctypes.POINTER(ctypes.c_float), ctypes.c_size_t]
        lib.rust_native_encode_text_f32.restype = ctypes.c_int32
        _LIB, _LIB_PATH = lib, dll
    return _LIB


def quantize(vec):
    n = math.sqrt(sum(x * x for x in vec)) or 1.0
    v = [x / n for x in vec]
    lo, hi = min(v), max(v)
    span = (hi - lo) if (hi - lo) > 1e-8 else 1.0
    codes = [int(round(((x - lo) / span) * 3.0)) & 0x03 for x in v]
    packed = bytearray(len(codes) // CODES_PER_BYTE)
    for i in range(0, len(codes) - 3, CODES_PER_BYTE):
        packed[i // CODES_PER_BYTE] = (codes[i] | (codes[i + 1] << 2) |
                                       (codes[i + 2] << 4) | (codes[i + 3] << 6))
    return bytes(packed), span, lo


def dequantize(packed, span, lo):
    codes = []
    for b in packed[:DIM // CODES_PER_BYTE]:
        codes += [b & 3, (b >> 2) & 3, (b >> 4) & 3, (b >> 6) & 3]
    v = [lo + (c / 3.0) * span for c in codes[:DIM]]
    n = math.sqrt(sum(x * x for x in v)) or 1.0
    return [x / n for x in v]


def _f3(x):
    return str(round(x, 3))


def work_chunk(job):
    """job = (lines, dll, want_fidelity). Returns per-record (ok, packed, span, lo, cos)."""
    lines, dll, want_fid = job
    from rdkit import Chem
    from rdkit.Chem import Descriptors, Crippen, Lipinski
    lib = _load(dll)
    out = []
    buf = (ctypes.c_float * DIM)()
    for line in lines:
        line = line.rstrip("\n\r")
        if not line:
            continue
        smi, _, cid = line.partition("\t")
        smi, cid = smi.strip(), cid.strip()
        if not smi:
            continue
        m = Chem.MolFromSmiles(smi)
        if m is None:
            out.append((False, cid, None, None, None))
            continue
        text = (f"chembl_id: {cid}\nsmiles: {smi}\n"
                f"hba={Lipinski.NumHAcceptors(m)} hbd={Lipinski.NumHDonors(m)} "
                f"heavy_atoms={m.GetNumHeavyAtoms()} "
                f"logp={_f3(Crippen.MolLogP(m))} mw={_f3(Descriptors.MolWt(m))}")
        raw = text.encode("utf-8", "ignore")
        rc = lib.rust_native_encode_text_f32(raw, len(raw), buf, DIM)
        if rc != DIM:
            raise RuntimeError(f"encoder rc={rc} for {cid} (success is rc == dim == {DIM})")
        packed, span, lo = quantize(list(buf))
        cos = None
        if want_fid:
            dec = dequantize(packed, span, lo)
            nv = math.sqrt(sum(x * x for x in buf)) or 1.0
            cos = sum(a * b for a, b in zip([x / nv for x in buf], dec))
        out.append((True, cid, packed, span, lo, cos) if want_fid else (True, cid, packed, span, lo, None))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--file", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--dll", required=True)
    ap.add_argument("--workers", type=int, default=1)
    ap.add_argument("--limit", type=int, default=None)
    ap.add_argument("--chunk", type=int, default=400)
    a = ap.parse_args()

    with open(a.file, encoding="utf-8", errors="ignore") as f:
        lines = [l for l in f if l.strip()]
    if a.limit:
        lines = lines[:a.limit]
    chunks = [(lines[i:i + a.chunk], a.dll, i == 0) for i in range(0, len(lines), a.chunk)]
    print(f"file {os.path.basename(a.file)}  lines {len(lines):,}  workers {a.workers}  chunks {len(chunks)}")

    payload = bytearray()
    spans = []
    fidelity = []
    bad = []
    done = 0
    t0 = time.time()
    if a.workers <= 1:
        results = map(work_chunk, chunks)
        pool = None
    else:
        pool = Pool(a.workers)
        results = pool.imap(work_chunk, chunks)
    try:
        for res in results:
            for ok, cid, packed, span, lo, cos in res:
                if not ok:
                    bad.append(cid)
                    continue
                payload.extend(packed)
                spans.append((round(span, 6), round(lo, 6)))
                if cos is not None and len(fidelity) < 24:
                    fidelity.append(cos)
            done += len(res)
            if done % (a.chunk * 5) == 0:
                print(f"  {done:,}/{len(lines):,}  {done / max(time.time() - t0, 1e-9):.0f}/s", flush=True)
    finally:
        if pool is not None:
            pool.close()
            pool.join()

    n_records = len(payload) // PACK
    if n_records == 0:
        raise SystemExit("FAIL-CLOSED: no payload produced")
    if len(payload) % PACK:
        raise SystemExit(f"FAIL-CLOSED: payload {len(payload)} not a multiple of {PACK}")

    header = bytearray(128)
    header[0:8] = b"TURBOVEC"
    struct.pack_into("<Q", header, 8, MONAD_ANCHOR)
    struct.pack_into("<Q", header, 16, n_records * DIM)
    struct.pack_into("<I", header, 24, SHARD_ID)
    struct.pack_into("<I", header, 28, DIM)
    struct.pack_into("<I", header, 32, 16)
    struct.pack_into("<Q", header, 36, 128)
    struct.pack_into("<I", header, 44, n_records)
    struct.pack_into("<I", header, 48, 0)
    seal = hashlib.sha256(payload).digest()
    header[96:128] = seal
    os.makedirs(os.path.dirname(a.out) or ".", exist_ok=True)
    with open(a.out, "wb") as f:
        f.write(header)
        f.write(payload)

    el = time.time() - t0
    meta = {"source": "chembl37_desc_parallel", "model": os.path.basename(a.dll), "dim": DIM,
            "records": n_records, "trits": n_records * DIM, "payload_bytes": len(payload),
            "file_bytes": os.path.getsize(a.out), "padding_bytes": 0, "seal_sha256": seal.hex(),
            "quantization": "2-bit symmetric per vector, 4 codes/byte",
            "fidelity_cosine_sample": round(sum(fidelity) / len(fidelity), 4) if fidelity else None,
            "fidelity_n": len(fidelity), "workers": a.workers, "seconds": round(el, 1),
            "records_per_s": round(n_records / el, 1), "unparseable": len(bad), "unparseable_ids": bad[:5],
            "built_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
    json.dump(meta, open(a.out + ".json", "w"), indent=1)
    print(json.dumps(meta, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
