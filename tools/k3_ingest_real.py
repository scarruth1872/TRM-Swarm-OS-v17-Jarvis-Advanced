"""k3_ingest_real.py - build K3 shards from records that actually exist.

Replaces the sine-wave + padding generator (G163) for any corpus that is genuinely on this machine.

Contract:
  * every record written comes from a real source file, one embedding per record;
  * the header's trit count equals records * dim of data actually written - never a padding target;
  * the file is not padded: payload length = ceil(records * dim / 4) bytes exactly;
  * the seal is SHA-256 over the payload that was written;
  * quantization fidelity is measured (decode -> cosine vs original unit vector) and reported,
    because 2-bit codes are lossy and claiming otherwise would be the same lie in a new shape.

Usage:
  python tools/k3_ingest_real.py --source chembl --limit 5000
  python tools/k3_ingest_real.py --source chembl            # all records
  python tools/k3_ingest_real.py --verify build/chembl_real.k3
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import os
import struct
import sys
import time
import urllib.error
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EMBED_URL = "http://127.0.0.1:8593/v1/embeddings"
MODELS_URL = "http://127.0.0.1:8593/v1/models"
MONAD_ANCHOR = 0x4D4F4E4144000001
CODES_PER_BYTE = 4

SOURCES = {
    "chembl": {
        "shard_id": 101,
        "files": ["scratch/wt_memory/swarm_v2_artifacts/chembl_smiles.json",
                  "scratch/wt_models/swarm_v2_artifacts/chembl_smiles.json"],
        "kind": "smiles_map",
    },
    # Full ChEMBL-37 SMILES dump recovered from the laptop's external SSD (D:\chembl_smiles_full.txt,
    # 203.6 MB, ~5.2 M lines, format "SMILES<TAB>CHEMBL_ID"). G198 found it after a 6.4 M-file sweep had
    # reported ChEMBL-37 "absent" because the sweep never enumerated removable volumes.
    "chembl37": {
        "shard_id": 104,
        "files": ["scratch/chembl37/chembl_smiles_full.txt",
                  "F:/Development sites/datasets/chembl_smiles_full.txt"],
        "kind": "smiles_lines",
    },
    # ARC-AGI concept-aug-1000 (145,889 train + 145,889 test puzzles, Kaggle npy layout).
    # Encoding measured, not assumed: 900 tokens = 30x30 grid, token 1..10 = ARC colour 0..9, token 0 = padding,
    # token 11 = an extra vocabulary entry whose meaning is NOT established here and is rendered as 'x' rather
    # than guessed. Each row is one example input; the row at the same index in `all__labels` is its output;
    # `all__puzzle_indices` gives each puzzle's row range.
    # Real work history from the UQI kanban store (4,135 tasks measured, 0 blank titles). These are the
    # platform's own goals and their descriptions - the only corpus here that records what was actually asked.
    # Property-grounded ChEMBL text, matching the fabric's own sidecar format exactly. The SDF carries only
    # `chembl_id` (measured: 1,472 tags / 1,472 records in an 8 MB sample), so the properties were computed at
    # build time and are recomputed here with RDKit. Recipe confirmed against the fabric's published numbers:
    # NumHAcceptors=3, NumHDonors=2, GetNumHeavyAtoms=16, MolLogP=1.362, Descriptors.MolWt=235.316 for
    # Cc1cc(-c2csc(N=C(N)N)n2)cn1C - all five matched. Requires an interpreter that HAS rdkit (desktop: py3.12).
    # Real EPA Tox21 12-endpoint matrix (7,831 compounds x 12 assays, measured: 7,832 lines incl header).
    # Blank cells are rendered 'na' - an assay that was not run is not an inactive, and inventing 0 would be
    # a false negative in a safety corpus.
    # Real BindingDB affinity matrix. Measured from the archive itself: 243 columns, one row per ligand-target
    # measurement, with Ki / IC50 / Kd / EC50 in nM, SMILES, target name + organism, UniProt ids, PDB ids, DOI
    # and PMID. Only populated affinity fields are rendered - a blank cell is omitted, never written as 0.
    "bindingdb": {
        "shard_id": 109,
        "files": ["F:/Development sites/datasets/bindingdb/BindingDB_All.zip",
                  "F:/Development sites/datasets/bindingdb/BindingDB_All.tsv"],
        "kind": "bindingdb_tsv",
    },
    "tox21": {
        "shard_id": 108,
        "files": ["F:/Development sites/datasets/tox21/tox21.csv.gz",
                  "F:/Development sites/datasets/tox21/tox21.csv"],
        "kind": "tox21_csv",
    },
    "chembl37_desc": {
        "shard_id": 107,
        "files": ["scratch/chembl37/chembl_smiles_full.txt",
                  "F:/Development sites/datasets/chembl_smiles_full.txt"],
        "kind": "smiles_rdkit",
    },
    "kanban": {
        "shard_id": 106,
        "files": ["F:/Development sites/ub3/uqi-platform/data/kanban-config/db.v2.sqlite"],
        "kind": "sqlite_rows",
        "table": "tasks",
    },
    "arc": {
        "shard_id": 105,
        "files": ["F:/Development sites/TRM agent swarm/data/arc1concept-aug-1000"],
        "kind": "arc_puzzles",
    },
    "chembl_lpxc": {
        "shard_id": 102,
        "files": ["evidence/chembl_lpxc_actives.json"],
        "kind": "actives_list",
    },
    # F:\Development sites\datasets\smpdb_proteins.csv is a DIRECTORY whose 21,121 per-pathway
    # CSVs all carry the same 966-byte shape - name-based sweeps never look inside it (G165).
    "smpdb": {
        "shard_id": 103,
        "files": ["F:/Development sites/datasets/smpdb_proteins.csv"],
        "kind": "smpdb_dir",
    },
}


def _post(url: str, payload: dict, timeout: int = 180) -> dict:
    req = urllib.request.Request(url, data=json.dumps(payload).encode(),
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def model_id() -> str:
    with urllib.request.urlopen(MODELS_URL, timeout=15) as r:
        m = json.loads(r.read())["models"][0]
    # this server reports name/model, not id (checked 2026-11-16) - fall back rather than return None
    return m.get("id") or m.get("model") or m.get("name")


def embed(texts: list[str], model: str, dim: int) -> list[list[float]]:
    resp = _post(EMBED_URL, {"model": model, "input": texts})
    rows = [d["embedding"] for d in resp["data"]]
    for r in rows:
        if len(r) != dim:
            raise RuntimeError(f"dimension mismatch: got {len(r)}, expected {dim}")
    return rows


def load_records(name: str, limit: int | None) -> list[tuple[str, str]]:
    spec = SOURCES[name]
    path = next((f if os.path.isabs(f) else os.path.join(ROOT, f)
                 for f in spec["files"] if os.path.exists(f if os.path.isabs(f) else os.path.join(ROOT, f))), None)
    if not path:
        raise SystemExit(f"FAIL-CLOSED: none of {spec['files']} exist - refusing to synthesise records")
    if spec["kind"] == "smpdb_dir":
        out: list[tuple[str, str]] = []
        for dp, _dn, fn in os.walk(path):
            for f in sorted(fn):
                if not f.lower().endswith(".csv"):
                    continue
                with open(os.path.join(dp, f), encoding="utf-8", errors="ignore") as fh:
                    rows = list(csv.DictReader(fh))
                for r in rows:
                    prot = (r.get("Protein Name") or "").strip()
                    acc = (r.get("Uniprot ID") or "").strip()
                    if not prot and not acc:
                        continue
                    out.append((f"{r.get('SMPDB ID','?')}:{acc}",
                                f"SMPDB pathway {r.get('Pathway Name','?')} ({r.get('Pathway Subject','?')}) "
                                f"protein {prot} uniprot {acc} gene {r.get('Gene Name','?')} "
                                f"hmdb {r.get('HMDBP ID','?')} drugbank {r.get('DrugBank ID','?')}"))
                    if limit and len(out) >= limit:
                        return out
        if not out:
            raise SystemExit(f"FAIL-CLOSED: {path} yielded 0 protein rows")
        return out
    if spec["kind"] == "bindingdb_tsv":
        # Row -> text lives in tools/bindingdb_serialize.py so the byte-ranged parallel path cannot drift.
        sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
        from bindingdb_serialize import open_text, header_index, row_to_record
        stream = open_text(path)
        ix = header_index(stream.readline())
        out7: list[tuple[str, str]] = []
        skipped = 0
        for line in stream:
            rec = row_to_record(line.rstrip("\n").split("\t"), ix, len(out7))
            if rec is None:
                skipped += 1
                continue
            out7.append(rec)
            if limit and len(out7) >= limit:
                break
        if not out7:
            raise SystemExit(f"FAIL-CLOSED: {path} yielded 0 BindingDB rows")
        print(f"  bindingdb: {len(out7)} rows, {skipped} skipped (no SMILES/short row)", flush=True)
        return out7
    if spec["kind"] == "tox21_csv":
        import csv as _csv
        import gzip as _gz
        opener = _gz.open if path.endswith(".gz") else open
        out6: list[tuple[str, str]] = []
        with opener(path, "rt", encoding="utf-8", errors="ignore") as fh:
            rd = _csv.DictReader(fh)
            fields = [f for f in rd.fieldnames if f not in ("mol_id", "smiles")]
            for row in rd:
                mid = (row.get("mol_id") or "").strip()
                smi = (row.get("smiles") or "").strip()
                if not mid or not smi:
                    continue
                parts, act = [], 0
                for f in fields:
                    v = (row.get(f) or "").strip()
                    if v not in ("0", "1"):
                        v = "na"          # not measured stays not measured
                    else:
                        act += int(v)
                    parts.append(f"{f}={v}")
                text = (f"Tox21 assay compound {mid} SMILES {smi} "
                        f"{len(fields)} endpoints active {act} of {sum(1 for p_ in parts if not p_.endswith('=na'))} "
                        + " ".join(parts))
                out6.append((f"tox21/{mid}", text))
                if limit and len(out6) >= limit:
                    break
        if not out6:
            raise SystemExit(f"FAIL-CLOSED: {path} yielded 0 Tox21 records")
        print(f"  tox21: {len(out6)} compounds, {len(fields)} endpoints", flush=True)
        return out6
    if spec["kind"] == "smiles_rdkit":
        try:
            from rdkit import Chem
            from rdkit.Chem import Descriptors, Crippen, Lipinski
        except ImportError:
            raise SystemExit("FAIL-CLOSED: rdkit not importable in THIS interpreter - run with one that has it "
                             "(desktop: Python312; laptop: py -3 / Python313). Refusing to invent descriptors.")
        out5: list[tuple[str, str]] = []
        bad = 0
        with open(path, encoding="utf-8", errors="ignore") as fh:
            for line in fh:
                line = line.rstrip("\n\r")
                if not line:
                    continue
                smi, _, cid = line.partition("\t")
                smi, cid = smi.strip(), cid.strip()
                if not smi:
                    continue
                m = Chem.MolFromSmiles(smi)
                if m is None:
                    bad += 1
                    if bad <= 5:
                        print(f"  unparseable SMILES (skipped): {cid} {smi[:40]}", flush=True)
                    continue
                def _f3(x: float) -> str:
                    """The fabric prints `str(round(x, 3))`: mw=2883.39, mw=857.0, logp=0.0. Determined by
                    diffing 5,000 records against its sidecar - `:.3f` gives 2883.390/857/0 and matches none."""
                    return str(round(x, 3))

                text = (f"chembl_id: {cid}\nsmiles: {smi}\n"
                        f"hba={Lipinski.NumHAcceptors(m)} hbd={Lipinski.NumHDonors(m)} "
                        f"heavy_atoms={m.GetNumHeavyAtoms()} "
                        f"logp={_f3(Crippen.MolLogP(m))} mw={_f3(Descriptors.MolWt(m))}")
                out5.append((f"chembl/{cid}", text))
                if limit and len(out5) >= limit:
                    break
        if not out5:
            raise SystemExit(f"FAIL-CLOSED: {path} yielded 0 descriptor records")
        print(f"  smiles_rdkit: {len(out5)} records, {bad} unparseable skipped", flush=True)
        return out5
    if spec["kind"] == "sqlite_rows":
        import sqlite3 as _sq
        if not os.path.isabs(path) and not os.path.exists(path):
            raise SystemExit(f"FAIL-CLOSED: {path} missing")
        con = _sq.connect(f"file:{path}?mode=ro", uri=True)
        tbl = spec.get("table", "tasks")
        cur = con.execute(f'select * from "{tbl}"')
        names = [d[0] for d in cur.description]
        out4: list[tuple[str, str]] = []
        for row in cur:
            d = dict(zip(names, row))
            title = str(d.get("title") or "").strip()
            if not title:
                continue
            desc = str(d.get("description") or "").strip()
            key = f"{tbl}/{d.get('id')}"
            text = (f"kanban {tbl} task {title} status {d.get('status')} "
                    f"created {d.get('created_at')} description {desc}")
            out4.append((key, text[:4000]))
            if limit and len(out4) >= limit:
                break
        con.close()
        if not out4:
            raise SystemExit(f"FAIL-CLOSED: {path}:{tbl} yielded 0 rows")
        print(f"  {tbl}: {len(out4)} rows serialised", flush=True)
        return out4
    if spec["kind"] == "arc_puzzles":
        # Serialisation lives in tools/arc_serialize.py so the parallel ingest cannot drift from this one.
        sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
        from arc_serialize import ArcSplit
        out3: list[tuple[str, str]] = []
        for split in ("train", "test"):
            if not os.path.isdir(os.path.join(path, split)):
                continue
            sp = ArcSplit(path, split)
            for i in range(len(sp)):
                out3.append(sp.record(i))
                if limit and len(out3) >= limit:
                    return out3
        if not out3:
            raise SystemExit(f"FAIL-CLOSED: {path} yielded 0 ARC puzzles")
        print(f"  arc: {len(out3)} puzzles serialised", flush=True)
        return out3
    if spec["kind"] == "smiles_lines":
        # Streamed: the file holds ~5.2 M lines, so it is read line by line and stops at `limit`.
        out2: list[tuple[str, str]] = []
        with open(path, encoding="utf-8", errors="ignore") as fh:
            for line in fh:
                line = line.rstrip("\n\r")
                if not line or line.startswith("#"):
                    continue
                if "\t" in line:
                    smi, cid = line.split("\t", 1)
                else:
                    smi, cid = line, f"line{len(out2)}"
                smi = smi.strip()
                cid = cid.strip() or f"line{len(out2)}"
                if not smi:
                    continue
                out2.append((cid, f"ChEMBL molecule {cid} SMILES {smi}"))
                if limit and len(out2) >= limit:
                    break
        if not out2:
            raise SystemExit(f"FAIL-CLOSED: {path} yielded 0 SMILES lines")
        return out2
    data = json.load(open(path, encoding="utf-8"))
    out: list[tuple[str, str]] = []
    if spec["kind"] == "smiles_map":
        for cid, smi in data.items():
            if not smi:
                continue
            out.append((cid, f"ChEMBL molecule {cid} SMILES {smi}"))
    elif spec["kind"] == "actives_list":
        for r in data:
            smi = r.get("smiles") or ""
            val = r.get("value_nM")
            if not smi:
                continue
            out.append((r.get("chembl_id", "?"),
                        f"ChEMBL {r.get('chembl_id','?')} measured {r.get('type','Ki')} "
                        f"{val} nM against {r.get('assay','?')[:90]} SMILES {smi}"))
    else:
        raise SystemExit(f"unknown source kind {spec['kind']}")
    if not out:
        raise SystemExit(f"FAIL-CLOSED: {path} produced 0 usable records")
    if limit:
        out = out[:limit]
    return out


def quantize(vec: list[float]) -> tuple[bytearray, float, float]:
    """Per-vector symmetric 2-bit quantization. Returns (packed, scale, min) - same on-disk layout
    as the legacy shards so existing readers still work, but the codes come from a real vector."""
    n = math.sqrt(sum(x * x for x in vec)) or 1.0
    v = [x / n for x in vec]
    lo, hi = min(v), max(v)
    span = (hi - lo) if (hi - lo) > 1e-8 else 1.0
    codes = [int(round(((x - lo) / span) * 3.0)) & 0x03 for x in v]
    packed = bytearray(len(codes) // CODES_PER_BYTE)
    for i in range(0, len(codes) - 3, CODES_PER_BYTE):
        packed[i // CODES_PER_BYTE] = (codes[i] | (codes[i + 1] << 2) |
                                       (codes[i + 2] << 4) | (codes[i + 3] << 6))
    return packed, span, lo


def dequantize(packed: bytes, dim: int, span: float, lo: float) -> list[float]:
    codes = []
    for b in packed[: dim // CODES_PER_BYTE]:
        codes += [b & 3, (b >> 2) & 3, (b >> 4) & 3, (b >> 6) & 3]
    v = [lo + (c / 3.0) * span for c in codes[:dim]]
    n = math.sqrt(sum(x * x for x in v)) or 1.0
    return [x / n for x in v]


class NativeEncoder:
    """The kernel's own model-free text encoder, called in-process instead of over HTTP.

    `rust_native_encode_text_f32(text, len, out, dim)` returns `dim` on success (768), not 0 - a caller that
    checks `rc == 0` will see a failure on every successful call. Measured on this box at 4,135 records/s
    (242 us), versus ~16/s for the HTTP embedding server: the 768-dim projection is arithmetic, and the
    server's 60 ms per request was queueing and JSON, not mathematics.

    IMPORTANT: this encoder produces `native-turbovec-hdc` vectors. The HTTP path produces a different model's
    vectors. The two are different vector spaces and must never be mixed inside one shard - the engine is
    therefore recorded in the shard's provenance.
    """

    def __init__(self, dll: str | None, dim: int = 768):
        import ctypes
        if not dll or not os.path.exists(dll):
            raise SystemExit(f"FAIL-CLOSED: native encoder dll not found: {dll}")
        self.dim = dim or 768
        self.lib = ctypes.CDLL(dll)
        self.lib.rust_native_encode_text_f32.argtypes = [ctypes.c_char_p, ctypes.c_size_t,
                                                        ctypes.POINTER(ctypes.c_float), ctypes.c_size_t]
        self.lib.rust_native_encode_text_f32.restype = ctypes.c_int32
        self.dll = os.path.basename(dll)
        self.calls = 0

    def one(self, text: str) -> list[float]:
        import ctypes
        b = text.encode("utf-8", "ignore")
        buf = (ctypes.c_float * self.dim)()
        rc = self.lib.rust_native_encode_text_f32(b, len(b), buf, self.dim)
        if rc != self.dim:
            raise RuntimeError(f"native encoder rc={rc} (success is rc == dim == {self.dim})")
        self.calls += 1
        return list(buf)

    def many(self, texts: list[str]) -> list[list[float]]:
        return [self.one(t) for t in texts]


def build(source: str, limit: int | None, out_path: str, batch: int = 16,
          engine: str = "http", dll: str | None = None) -> dict:
    records = load_records(source, limit)
    native = NativeEncoder(dll) if engine == "native" else None
    model = native.dll if native else model_id()
    print(f"model={model[:48]}…  records={len(records)}", flush=True)
    dim = None
    payload = bytearray()
    scales = []
    fidelity = []
    started = time.time()
    counters = {"pooled": 0, "skipped": 0}

    def is_too_large(err: urllib.error.HTTPError) -> bool:
        detail = err.read().decode("utf-8", "ignore")
        return err.code == 500 and ("too large" in detail or "batch size" in detail)

    def embed_one(text: str) -> list[float]:
        """Embed one record. The server's physical batch is 512 tokens, so anything longer is split
        into pieces that are shrunk until they fit, then mean-pooled and renormalised. Never dropped."""
        try:
            return embed([text], model, dim or 768)[0]
        except urllib.error.HTTPError as e:
            if not is_too_large(e):
                raise
        for size in (600, 400, 250, 120):
            pieces = [text[j:j + size] for j in range(0, len(text), size)]
            try:
                pv = embed(pieces, model, dim or 768)
            except urllib.error.HTTPError as e:
                if is_too_large(e):
                    continue
                raise
            counters["pooled"] += 1
            n = len(pv)
            mean = [sum(col) / n for col in zip(*pv)]
            norm = math.sqrt(sum(x * x for x in mean)) or 1.0
            return [x / norm for x in mean]
        raise RuntimeError("record could not be embedded even at 120 chars - refusing to skip silently")

    for i in range(0, len(records), batch):
        chunk = records[i:i + batch]
        if native is not None:
            vecs = native.many([t for _, t in chunk])
        else:
            try:
                vecs = embed([t for _, t in chunk], model, dim or 768)
            except urllib.error.HTTPError as e:
                if not is_too_large(e):
                    raise
                vecs = [embed_one(t) for _, t in chunk]
        if dim is None:
            dim = len(vecs[0])
        for vec in vecs:
            packed, span, lo = quantize(vec)
            payload.extend(packed)
            scales.append((round(span, 6), round(lo, 6)))
            if len(fidelity) < 24:  # measure on a sample - decoding all 44k is wasted work
                dec = dequantize(packed, dim, span, lo)
                nv = math.sqrt(sum(x * x for x in vec)) or 1.0
                orig = [x / nv for x in vec]
                fidelity.append(sum(a * b for a, b in zip(orig, dec)))
        done = min(i + batch, len(records))
        if done % (batch * 20) == 0:
            rate = done / max(time.time() - started, 1e-9)
            print(f"  {done}/{len(records)}  {rate:.1f}/s", flush=True)

    if not payload or dim is None:
        raise SystemExit("FAIL-CLOSED: no payload produced")
    n_records = len(payload) // (dim // CODES_PER_BYTE)

    header = bytearray(128)
    header[0:8] = b"TURBOVEC"
    struct.pack_into("<Q", header, 8, MONAD_ANCHOR)
    struct.pack_into("<Q", header, 16, n_records * dim)      # trits actually written
    struct.pack_into("<I", header, 24, SOURCES[source]["shard_id"])
    struct.pack_into("<I", header, 28, dim)
    struct.pack_into("<I", header, 32, 16)                    # 32-bit float -> 2-bit code
    struct.pack_into("<Q", header, 36, 128)                   # payload offset
    struct.pack_into("<I", header, 44, n_records)             # record count, real
    struct.pack_into("<I", header, 48, 0)                     # padding bytes: always 0 (G164)

    seal = hashlib.sha256(payload).digest()
    header[96:128] = seal

    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    with open(out_path, "wb") as f:
        f.write(header)
        f.write(payload)

    meta = {
        "source": source,
        "source_files": SOURCES[source]["files"],
        "model": model,
        "dim": dim,
        "records": n_records,
        "trits": n_records * dim,
        "payload_bytes": len(payload),
        "file_bytes": os.path.getsize(out_path),
        "padding_bytes": 0,
        "seal_sha256": seal.hex(),
        "quantization": "2-bit symmetric per vector, 4 codes/byte",
        "fidelity_cosine_sample": round(sum(fidelity) / len(fidelity), 4) if fidelity else None,
        "fidelity_n": len(fidelity),
        "chunk_pooled_records": counters["pooled"],
        "seconds": round(time.time() - started, 1),
        "built_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    with open(out_path + ".json", "w", encoding="utf-8") as fh:
        json.dump(meta, fh, indent=2)
    return meta


def verify(path: str) -> dict:
    raw = open(path, "rb").read()
    if raw[:8] != b"TURBOVEC":
        raise SystemExit("not a TURBOVEC shard")
    trits = struct.unpack_from("<Q", raw, 16)[0]
    dim = struct.unpack_from("<I", raw, 28)[0]
    records = struct.unpack_from("<I", raw, 44)[0]
    pad = struct.unpack_from("<I", raw, 48)[0]
    seal = raw[96:128]
    payload = raw[128:]
    ok = hashlib.sha256(payload).digest() == seal
    return {"path": path, "dim": dim, "records": records, "trits": trits,
            "payload_bytes": len(payload), "expected_payload": records * dim // CODES_PER_BYTE,
            "padding_bytes": pad, "seal_matches": ok,
            "exact": len(payload) == records * dim // CODES_PER_BYTE}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", choices=sorted(SOURCES), help="corpus to ingest")
    ap.add_argument("--limit", type=int, default=None)
    ap.add_argument("--engine", choices=["http", "native"], default="http",
                    help="native = in-process rust_native_encode_text_f32 (4,135 rec/s); "
                         "http = the embedding server (~16 rec/s). Different vector spaces.")
    ap.add_argument("--dll", default=None,
                    help="path to swarm_rust_kernel.dll for --engine native")
    ap.add_argument("--out", default=None)
    ap.add_argument("--verify", default=None, help="verify an existing shard instead of building")
    a = ap.parse_args()
    if a.verify:
        print(json.dumps(verify(a.verify), indent=2))
        return 0
    if not a.source:
        ap.error("--source or --verify is required")
    out = a.out or os.path.join(ROOT, "build", f"{a.source}_real.k3")
    print(json.dumps(build(a.source, a.limit, out, engine=a.engine, dll=a.dll), indent=2))
    print(json.dumps(verify(out), indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
