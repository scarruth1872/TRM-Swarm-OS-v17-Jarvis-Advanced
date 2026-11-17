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
PDBBIND_LABELS = "F:/Development sites/datasets/pdbbind_labels/pdbbind2019_labels.json"
LPDBBIND_CSV = "C:/Users/shawn/AppData/Local/hermes/cache/scratch/laptop_ds/LP_PDBBind.csv"
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


def work_arc(job):
    """job = (root, split, lo, hi, dll). Serialisation comes from tools/arc_serialize.py so this worker cannot
    drift from the serial ingest."""
    root, split, lo, hi, dll, want_fid = job
    import sys as _sys
    _sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    from arc_serialize import ArcSplit
    sp = ArcSplit(root, split)
    lib = _load(dll)
    buf = (ctypes.c_float * DIM)()
    out = []
    for i in range(lo, hi):
        key, text = sp.record(i)
        raw = text.encode("utf-8", "ignore")
        if lib.rust_native_encode_text_f32(raw, len(raw), buf, DIM) != DIM:
            raise RuntimeError(f"encoder failed on {key}")
        packed, span, loq = quantize(list(buf))
        cos = None
        if want_fid:
            # control: a shard that decodes to nothing sensible is a fast way to store noise
            dec = dequantize(packed, span, loq)
            nv = math.sqrt(sum(x * x for x in buf)) or 1.0
            cos = sum(a * bb for a, bb in zip([x / nv for x in buf], dec))
        out.append((True, key, packed, span, loq, cos))
    return out


def work_ct(job):
    """job = (zip_path, start_idx, end_idx, dll, want_fid). Members by index: the 612,025-study export is a flat
    zip, so index ranges are the natural split and the central directory is read once per worker."""
    zip_path, start, end, dll, want_fid = job
    import sys as _sys
    import zipfile as _zf
    _sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    from clinicaltrials_serialize import study_text
    lib = _load(dll)
    buf = (ctypes.c_float * DIM)()
    out = []
    skipped = 0
    seen = 0
    z = _zf.ZipFile(zip_path)
    names = z.namelist()
    for i in range(start, min(end, len(names))):
        nm = names[i]
        if not nm.lower().endswith(".xml"):
            continue
        seen += 1
        with z.open(nm) as fh:
            rec = study_text(fh.read())
        if rec is None:
            skipped += 1
            continue
        key, text = rec
        raw = text.encode("utf-8", "ignore")
        if lib.rust_native_encode_text_f32(raw, len(raw), buf, DIM) != DIM:
            raise RuntimeError(f"encoder failed on {key}")
        packed, span, loq = quantize(list(buf))
        cos = None
        if want_fid and len(out) < 24:
            dec = dequantize(packed, span, loq)
            nv = math.sqrt(sum(x * x for x in buf)) or 1.0
            cos = sum(a * bb for a, bb in zip([x / nv for x in buf], dec))
        out.append((True, key, packed, span, loq, cos))
    return {"records": out, "skipped": skipped, "seen": seen}


def work_pdbbind(job):
    """job = (tar_path, dll, want_fid). One tarball per worker: v2020-refined ships as 6 parts, so the archive
    itself is the natural unit of parallelism (no byte-range splitting inside a gzip stream)."""
    tar_path, dll, want_fid = job
    import sys as _sys
    _sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    from pdbbind_serialize import iter_complexes, load_labels, load_resolution, record_text
    labels = load_labels(PDBBIND_LABELS)
    resolutions = load_resolution(LPDBBIND_CSV)
    lib = _load(dll)
    buf = (ctypes.c_float * DIM)()
    out = []
    unmatched = 0
    seen = 0
    for pid, smi, _ in iter_complexes(tar_path):
        seen += 1
        text = record_text(pid, smi, labels, resolutions)
        if text is None:
            unmatched += 1
            continue
        raw = text.encode("utf-8", "ignore")
        if lib.rust_native_encode_text_f32(raw, len(raw), buf, DIM) != DIM:
            raise RuntimeError(f"encoder failed on {pid}")
        packed, span, loq = quantize(list(buf))
        cos = None
        if want_fid and len(out) < 24:
            dec = dequantize(packed, span, loq)
            nv = math.sqrt(sum(x * x for x in buf)) or 1.0
            cos = sum(a * bb for a, bb in zip([x / nv for x in buf], dec))
        out.append((True, f"pdbbind/{pid}", packed, span, loq, cos))
    return {"records": out, "skipped": unmatched, "seen": seen}


def work_tsv(job):
    """job = (path, start_byte, end_byte, dll, want_fid, chunk_id).

    Byte ranges, line aligned: a worker that starts mid-file discards its first partial line, and one that is
    not last stops after the line that begins before end_byte. Every row is processed exactly once.
    """
    path, start, end, dll, want_fid, chunk_id = job
    import sys as _sys
    _sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    from bindingdb_serialize import header_index, row_to_record
    lib = _load(dll)
    buf = (ctypes.c_float * DIM)()
    with open(path, "r", encoding="utf-8", errors="replace") as fh:
        ix = header_index(fh.readline())
        if start == 0:
            pass                               # already positioned after the header - do NOT seek(0), which would
        else:                                  # re-read the header as a data row (found: 1 bogus record/shard)
            fh.seek(start)
            fh.readline()                      # discard the partial line
        out = []
        skipped = 0
        seen = 0
        while True:
            pos = fh.tell()
            if pos > end and not (chunk_id == -1):
                break
            line = fh.readline()
            if not line:
                break
            seen += 1
            rec = row_to_record(line.rstrip("\n").split("\t"), ix, len(out))
            if rec is None:
                skipped += 1
                continue
            key, text = rec
            raw = text.encode("utf-8", "ignore")
            if lib.rust_native_encode_text_f32(raw, len(raw), buf, DIM) != DIM:
                raise RuntimeError(f"encoder failed on {key}")
            packed, span, loq = quantize(list(buf))
            cos = None
            if want_fid and len(out) < 24:
                dec = dequantize(packed, span, loq)
                nv = math.sqrt(sum(x * x for x in buf)) or 1.0
                cos = sum(a * bb for a, bb in zip([x / nv for x in buf], dec))
            out.append((True, f"{key}#{chunk_id}", packed, span, loq, cos))
    # Report drops. A source with N rows and a shard with N-74 records must say where the 74 went, or the
    # count difference becomes an unexplained discrepancy in every future audit.
    return {"records": out, "skipped": skipped, "seen": seen}


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
    ap.add_argument("--mode", choices=["chembl", "arc", "bindingdb", "pdbbind", "clinicaltrials"], default="chembl")
    ap.add_argument("--ct-zip", default="F:/Development sites/datasets/clinicaltrials/AllPublicXML.zip")
    ap.add_argument("--tars", default=None, help="comma-separated PDBbind tarballs, or a glob is not needed")
    ap.add_argument("--tsv", default="F:/Development sites/datasets/bindingdb/BindingDB_All.tsv")
    ap.add_argument("--chunk-mb", type=int, default=64)
    ap.add_argument("--arc-root", default="F:/Development sites/TRM agent swarm/data/arc1concept-aug-1000")
    ap.add_argument("--file", default=None)
    ap.add_argument("--out", required=True)
    ap.add_argument("--dll", required=True)
    ap.add_argument("--workers", type=int, default=1)
    ap.add_argument("--limit", type=int, default=None)
    ap.add_argument("--chunk", type=int, default=400)
    a = ap.parse_args()

    if a.mode == "clinicaltrials":
        import zipfile as _zf2
        with _zf2.ZipFile(a.ct_zip) as _z:
            n_members = len(_z.namelist())
        if a.limit:
            n_members = min(n_members, a.limit)
        step = a.chunk
        chunks = [(a.ct_zip, i, min(i + step, n_members), a.dll, i == 0)
                  for i in range(0, n_members, step)]
        worker = work_ct
        total = n_members
        print(f"clinicaltrials {n_members:,} members  chunks {len(chunks)}  workers {a.workers}", flush=True)
    elif a.mode == "pdbbind":
        import glob as _glob
        tars = (a.tars.split(",") if a.tars else
                sorted(_glob.glob("F:/Development sites/datasets/pdbbind/v2020-refined_*.tar.gz")))
        if not tars:
            raise SystemExit("FAIL-CLOSED: no PDBbind tarballs found")
        chunks = [(t_, a.dll, i == 0) for i, t_ in enumerate(tars)]
        worker = work_pdbbind
        total = len(tars)
        print(f"pdbbind {len(tars)} tarballs  workers {a.workers}", flush=True)
    elif a.mode == "bindingdb":
        total_bytes = os.path.getsize(a.tsv)
        step = a.chunk_mb * 1024 * 1024
        chunks = []
        i = 0
        pos = 0
        last = (total_bytes // step)
        while pos < total_bytes:
            end = min(pos + step, total_bytes)
            chunks.append((a.tsv, pos, end, a.dll, not chunks, -1 if pos + step >= total_bytes else i))
            pos = end
            i += 1
        worker = work_tsv
        total = 0
        print(f"tsv {os.path.basename(a.tsv)}  {total_bytes/1e9:.2f} GB  chunks {len(chunks)}  workers {a.workers}")
    elif a.mode == "arc":
        import sys as _sys
        _sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
        from arc_serialize import ArcSplit
        chunks = []
        for split in ("train", "test"):
            if not os.path.isdir(os.path.join(a.arc_root, split)):
                continue
            n = len(ArcSplit(a.arc_root, split))
            for i in range(0, n, a.chunk):
                chunks.append((a.arc_root, split, i, min(i + a.chunk, n), a.dll, not chunks))
        if a.limit:
            chunks = chunks[:max(1, a.limit // a.chunk)]
        worker = work_arc
        total = sum(c[3] - c[2] for c in chunks)
        print(f"arc root {a.arc_root}  puzzles {total:,}  workers {a.workers}  chunks {len(chunks)}")
    else:
        with open(a.file, encoding="utf-8", errors="ignore") as f:
            lines = [l for l in f if l.strip()]
        if a.limit:
            lines = lines[:a.limit]
        chunks = [(lines[i:i + a.chunk], a.dll, i == 0) for i in range(0, len(lines), a.chunk)]
        worker = work_chunk
        total = len(lines)
        print(f"file {os.path.basename(a.file)}  lines {total:,}  workers {a.workers}  chunks {len(chunks)}")

    payload = bytearray()
    spans = []
    fidelity = []
    bad = []
    done = 0
    skipped_total = 0
    seen_total = 0
    t0 = time.time()
    if a.workers <= 1:
        results = map(worker, chunks)
        pool = None
    else:
        pool = Pool(a.workers)
        results = pool.imap(worker, chunks)
    try:
        for res in results:
            if isinstance(res, dict):
                skipped_total += res["skipped"]
                seen_total += res.get("seen", 0)
                res = res["records"]
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
                print(f"  {done:,}/{total:,}  {done / max(time.time() - t0, 1e-9):.0f}/s", flush=True)
    finally:
        if pool is not None:
            pool.close()
            pool.join()

    n_records = len(payload) // PACK
    if seen_total and (n_records + skipped_total) != seen_total:
        raise SystemExit(f"FAIL-CLOSED: {seen_total} lines in, {n_records} records + {skipped_total} skipped "
                         f"= {n_records + skipped_total}; a line was dropped or duplicated")
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
    src = {"arc": "arc_concept_aug_1000_parallel", "bindingdb": "bindingdb_parallel",
           "pdbbind": "pdbbind_v2020_refined_parallel",
           "clinicaltrials": "clinicaltrials_public_xml_parallel"}.get(
        a.mode, "chembl37_desc_parallel")
    meta = {"source": src, "mode": a.mode,
            "input": {"arc": a.arc_root, "bindingdb": a.tsv, "clinicaltrials": a.ct_zip}.get(
                a.mode, a.tars or "F:/Development sites/datasets/pdbbind"),
            "model": os.path.basename(a.dll), "dim": DIM,
            "records": n_records, "trits": n_records * DIM, "payload_bytes": len(payload),
            "file_bytes": os.path.getsize(a.out), "padding_bytes": 0, "seal_sha256": seal.hex(),
            "quantization": "2-bit symmetric per vector, 4 codes/byte",
            "fidelity_cosine_sample": round(sum(fidelity) / len(fidelity), 4) if fidelity else None,
            "fidelity_n": len(fidelity), "workers": a.workers, "seconds": round(el, 1),
            "records_per_s": round(n_records / el, 1), "unparseable": len(bad), "unparseable_ids": bad[:5],
            "rows_seen": seen_total, "rows_skipped": skipped_total,
            "rows_skipped_reason": {"bindingdb": "no ligand SMILES / short row",
                                    "pdbbind": "complex with no affinity label",
                                    "clinicaltrials": "XML unparseable or no NCT id",
                                    "arc": "puzzle with no examples"}.get(a.mode, "row not representable"),
            "built_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
    json.dump(meta, open(a.out + ".json", "w"), indent=1)
    print(json.dumps(meta, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
