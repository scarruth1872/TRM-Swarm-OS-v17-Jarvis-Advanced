#!/usr/bin/env python3
"""
TRM Swarm OS — Unified GGUF ternary ingestion (council 3be7308e).
Reads Ollama GGUF blobs, dequantizes Q4_0/Q6_K/F32 tensors to fp32, re-quantizes
2-D weight tensors to TurboVec 1.58-bit balanced ternary {-1,0,+1} with a
per-tensor mean-abs alpha and +/-0.5 threshold, packs 4 trits/byte (2 bits/trit),
and writes a real packed-trit artifact plus a ledger JSON.

Correctness notes (verified against llama.cpp ggml-quants.c dequantize_row_q4_0 /
dequantize_row_q6_K):
  * Q4_0: 18 B/block, 32 weights. Block = [f16 scale][16 nibble bytes qs].
          weights[0:16]  = (qs[j] & 0x0F) - 8, scaled
          weights[16:32] = (qs[j] >>   4) - 8, scaled
  * Q6_K: 210 B/block, 256 weights. Block = [f16 d][128 ql][64 qh][16 int8 scales].
          Per 128-weight half: for l in 0..31 (is = l//16):
            q1 = (ql[l]&0xF)    | ((qh[l]>>0)&3)<<4   -> w[l+0]
            q2 = (ql[l+32]&0xF) | ((qh[l]>>2)&3)<<4   -> w[l+32]
            q3 = (ql[l]>>4)     | ((qh[l]>>4)&3)<<4   -> w[l+64]
            q4 = (ql[l+32]>>4)  | ((qh[l]>>6)&3)<<4   -> w[l+96]
            each minus 32, times d * scales[is + {0,2,4,6}]
  * Tensor data is read from the file memmap at t.data_offset with an EXACT
    t.n_bytes / t.n_elements window — never through an unbounded reshape, so no
    giant arrays are ever materialized for F32 tensors.

Usage: python quantize_gguf.py <model-tag> [max-tensors]
"""
import os, sys, time, json, glob, hashlib
import numpy as np

MONAD = 0x4D4F4E4144000001
ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
OLL = os.path.expanduser(r"C:\Users\shawn\.ollama\models")


def seal(s: str) -> int:
    h = int(hashlib.sha256(s.encode()).hexdigest()[:16], 16)
    return (MONAD ^ h) & 0xFFFFFFFFFFFFFFFF


def log(*a):
    print(*a, flush=True)


def find_blob(tag: str) -> str:
    """Resolve an Ollama model tag to its GGUF blob path (EXACT tail match).

    Matches the tag's path segments against the tail of the manifest path
    (e.g. 'gemma2:2b' -> .../manifests/registry.ollama.ai/library/gemma2/2b).
    The previous substring test was wrong: short tags like 'gemma2:2b' matched
    unrelated manifests whose path merely CONTAINS '2b' (e.g. bitnet-b1.58-2B-4T),
    silently quantizing the wrong model.
    """
    want = [s for s in tag.lower().replace(":", "/").split("/") if s]
    for m in glob.glob(os.path.join(OLL, "manifests", "**", "*"), recursive=True):
        if not os.path.isfile(m):
            continue
        mp = [s for s in os.path.normpath(m).replace("\\", "/").split("/") if s]
        if len(mp) < len(want) or mp[-len(want):] != want:
            continue
        import json
        d = json.load(open(m, encoding="utf-8"))
        for l in d.get("layers", []):
            if "model" in l.get("mediaType", ""):
                digest = l["digest"].replace("sha256:", "")
                bdir = os.path.join(OLL, "blobs")
                for f in os.listdir(bdir):
                    if digest in f:
                            return os.path.join(bdir, f)
    return ""


def _f16_cols(b, nbytes_per_block, colbytes):
    """Flat little-endian f16 (f32) values for the first colbytes of each block row.
    NOTE: (n, colbytes).copy().view('<f2') has shape (n, colbytes//2) — must flatten
    to (n,) before broadcasting against (n, k) weight arrays."""
    return b[:, 0:colbytes].copy().view("<f2").astype(np.float32).reshape(-1)


def dequant_tensor(reader, t):
    """
    Dequantize tensor t to a flat fp32 np array of exactly t.n_elements.
    Returns (arr, type_str) or (None, reason) on unsupported types.
    Reads raw bytes straight off the file memmap at t.data_offset (exact window).
    """
    tt = int(t.tensor_type)
    ne = int(t.n_elements)
    nb = int(t.n_bytes)
    off = int(t.data_offset)
    raw = np.asarray(reader.data[off:off + nb])  # exact-length uint8 copy
    # Q4_0 (type 2)
    if tt == 2:
        nblk = nb // 18
        if ne != nblk * 32:
            return None, f"Q4_0 shape-mismatch ne={ne} blocks={nblk}"
        b = raw.reshape(nblk, 18)
        d = _f16_cols(b, 18, 2)                      # (nblk,) f32 scale
        qs = b[:, 2:18]                              # 16 nibble bytes
        lo = (qs & 0x0F).astype(np.float32) - 8.0    # weights  0..15
        hi = (qs >> 4).astype(np.float32) - 8.0      # weights 16..31
        w = np.empty((nblk, 32), dtype=np.float32)
        w[:, 0:16] = lo * d[:, None]
        w[:, 16:32] = hi * d[:, None]
        return w.reshape(-1), "Q4_0"
    # Q6_K (type 14)
    if tt == 14:
        nblk = nb // 210
        if ne != nblk * 256:
            return None, f"Q6_K shape-mismatch ne={ne} blocks={nblk}"
        b = raw.reshape(nblk, 210)
        d = _f16_cols(b, 210, 2)                     # (nblk,) f32 global scale
        ql = b[:, 2:130]                             # 128 low-nibble bytes
        qh = b[:, 130:194]                           # 64 high-bit bytes
        sc = b[:, 194:210].copy().view(np.int8)      # 16 int8 scales
        out = np.empty((nblk, 256), dtype=np.float32)
        for h in (0, 1):
            l0 = ql[:, h * 64 + 0:h * 64 + 32]       # (nblk,32)
            l32 = ql[:, h * 64 + 32:h * 64 + 64]
            hb = qh[:, h * 32:h * 32 + 32]
            q1 = (((l0 & 0x0F) | (((hb >> 0) & 3) << 4)).astype(np.int16) - 32)
            q2 = (((l32 & 0x0F) | (((hb >> 2) & 3) << 4)).astype(np.int16) - 32)
            q3 = (((l0 >> 4) | (((hb >> 4) & 3) << 4)).astype(np.int16) - 32)
            q4 = (((l32 >> 4) | (((hb >> 6) & 3) << 4)).astype(np.int16) - 32)
            sh = sc[:, h * 8:h * 8 + 8]              # (nblk,8)
            g = np.repeat([0, 1], 16)                # is = l//16 pattern
            s1 = sh[:, g + 0].astype(np.float32)
            s2 = sh[:, g + 2].astype(np.float32)
            s3 = sh[:, g + 4].astype(np.float32)
            s4 = sh[:, g + 6].astype(np.float32)
            dd = d[:, None]
            w = np.empty((nblk, 128), dtype=np.float32)
            w[:, 0:32] = q1.astype(np.float32) * s1 * dd
            w[:, 32:64] = q2.astype(np.float32) * s2 * dd
            w[:, 64:96] = q3.astype(np.float32) * s3 * dd
            w[:, 96:128] = q4.astype(np.float32) * s4 * dd
            out[:, h * 128:h * 128 + 128] = w
        return out.reshape(-1), "Q6_K"
    # Q4_K (type 12): 144 B/block, 256 weights. Layout per block:
    #   [0:2]  f16 d (global scale), [2:4] f16 dmin (global min)
    #   [4:16] 12 B scales (8 scales+8 mins, 6-bit packed, via get_scale_min_k4)
    #   [16:144] 128 B qs (4-bit nibbles, 256 weights)
    # dequant per llama.cpp dequantize_row_q4_K:
    #   for j in 0..4 (4 groups of 64): sc,m = get_scale_min_k4(is,scales)
    #     d1=d*sc, m1=min*m; y[0:32]=d1*(qs&0xF)-m1; y[32:64]=d1*(qs>>4)-m1
    if tt == 12:
        nblk = nb // 144
        if ne != nblk * 256:
            return None, f"Q4_K shape-mismatch ne={ne} blocks={nblk}"
        b = raw.reshape(nblk, 144)
        d = b[:, 0:2].copy().view("<f2").astype(np.float32).reshape(-1)     # (nblk,)
        dmin = b[:, 2:4].copy().view("<f2").astype(np.float32).reshape(-1)  # (nblk,)
        scales = b[:, 4:16]                                     # (nblk,12)
        qs = b[:, 16:144]                                       # (nblk,128)
        out = np.empty((nblk, 256), dtype=np.float32)

        def gsmk(jj, q):
            """get_scale_min_k4(j, scales) -> (sc, m) 1:1 with llama.cpp."""
            jj = int(jj)
            if jj < 4:
                sc = (q[:, jj] & 63).astype(np.float32).reshape(-1)
                m = (q[:, jj + 4] & 63).astype(np.float32).reshape(-1)
            else:
                sc = ((q[:, jj + 4] & 0x0F) | ((q[:, jj - 4] >> 6) << 4)).astype(np.float32).reshape(-1)
                m = ((q[:, jj + 4] >> 4) | ((q[:, jj] >> 6) << 4)).astype(np.float32).reshape(-1)
            return sc, m

        is_ = 0
        for g in range(4):  # 4 groups of 32 qs bytes = 64 weights each
            sc1, m1 = gsmk(is_ + 0, scales)
            sc2, m2 = gsmk(is_ + 1, scales)
            d1 = (d * sc1)[:, None]
            mm1 = (dmin * m1)[:, None]
            d2 = (d * sc2)[:, None]
            mm2 = (dmin * m2)[:, None]
            qg = qs[:, g * 32:(g + 1) * 32]  # 32 bytes -> 64 weights
            lo = (qg & 0x0F).astype(np.float32)
            hi = (qg >> 4).astype(np.float32)
            out[:, g * 64:g * 64 + 32] = d1 * lo - mm1
            out[:, g * 64 + 32:g * 64 + 64] = d2 * hi - mm2
            is_ += 2
        return out.reshape(-1), "Q4_K"


    if tt == 0:
        if nb != ne * 4:
            return None, f"F32 n_bytes {nb} != n_elements*4 {ne*4}"
        return raw.view("<f4").astype(np.float32), "F32"
    if tt == 1:
        if nb != ne * 2:
            return None, f"F16 n_bytes {nb} != n_elements*2 {ne*2}"
        return raw.view("<f2").astype(np.float32), "F16"
    # Q8_0 (type 8): per-32 block: [2B f16 d][32B int8 qs], w = d*qs
    if tt == 8:
        nblk = nb // 34
        if ne != nblk * 32:
            return None, f"Q8_0 shape-mismatch ne={ne} blocks={nblk}"
        b = raw.reshape(nblk, 34)
        d = b[:, 0:2].copy().view("<f2").astype(np.float32).reshape(-1)
        qs = b[:, 2:34].astype(np.int8).astype(np.float32)  # (nblk,32)
        return (qs * d[:, None]).reshape(-1), "Q8_0"
    # Q5_K (type 6): 176 B/block, 256 weights. Layout (llama.cpp block_q5_K):
    #   [0:2] f16 d, [2:4] f16 dmin, [4:16] 12B scales, [16:48] 32B qh,
    #   [48:176] 128B qs (4-bit nibbles). weight via get_scale_min_k4 scales.
    if tt == 6:
        nblk = nb // 176
        if ne != nblk * 256:
            return None, f"Q5_K shape-mismatch ne={ne} blocks={nblk}"
        b = raw.reshape(nblk, 176)
        d = b[:, 0:2].copy().view("<f2").astype(np.float32).reshape(-1)
        dmin = b[:, 2:4].copy().view("<f2").astype(np.float32).reshape(-1)
        scarr = b[:, 4:16]
        qh = b[:, 16:48]   # (nblk,32)
        qs = b[:, 48:176]  # (nblk,128)
        out = np.empty((nblk, 256), dtype=np.float32)

        def gsmk5(j, q):
            j = int(j)
            if j < 4:
                sc = (q[:, j] & 63).astype(np.float32).reshape(-1)
                m = (q[:, j + 4] & 63).astype(np.float32).reshape(-1)
            else:
                sc = ((q[:, j + 4] & 0x0F) | ((q[:, j - 4] >> 6) << 4)).astype(np.float32).reshape(-1)
                m = ((q[:, j + 4] >> 4) | ((q[:, j] >> 6) << 4)).astype(np.float32).reshape(-1)
            return sc, m

        is_ = 0
        for g in range(4):  # 4 groups of 32 ql bytes -> 64 weights
            sc1, m1 = gsmk5(is_ + 0, scarr)
            sc2, m2 = gsmk5(is_ + 1, scarr)
            d1 = (d * sc1)[:, None]
            mm1 = (dmin * m1)[:, None]
            d2 = (d * sc2)[:, None]
            mm2 = (dmin * m2)[:, None]
            qg = qs[:, g * 32:(g + 1) * 32]  # 32 nibble bytes
            # qh bit for group g: low weight uses bit (2g), high uses bit (2g+1)
            lo_bit = ((qh >> (2 * g)) & 1).astype(np.float32)
            hi_bit = ((qh >> (2 * g + 1)) & 1).astype(np.float32)
            lo = (qg & 0x0F).astype(np.float32) + 16.0 * lo_bit
            hi = (qg >> 4).astype(np.float32) + 16.0 * hi_bit
            out[:, g * 64:g * 64 + 32] = d1 * lo - mm1
            out[:, g * 64 + 32:g * 64 + 64] = d2 * hi - mm2
            is_ += 2
        return out.reshape(-1), "Q5_K"
    return None, f"unsupported-type-{tt}"


def quantize_gguf_model(tag: str, max_tensors: int = 400, direct_blob: str = ""):
    if direct_blob:
        blob = direct_blob
    else:
        blob = find_blob(tag)
    if not blob:
        return {"ok": False, "error": f"model tag '{tag}' not found in Ollama"}
    log(f"blob: {os.path.getsize(blob)/1e6:.0f}MB | tag: {tag}")
    t_load = time.time()
    from gguf import GGUFReader
    reader = GGUFReader(blob)
    log(f"GGUFReader init: {time.time()-t_load:.1f}s | {len(reader.tensors)} tensors")

    t0 = time.time()
    n = 0
    total_params = 0
    total_src_bytes = 0          # original on-disk GGUF bytes of processed tensors
    total_fp32_bytes = 0         # dequantized fp32 bytes (4 B/elem)
    total_trit_bytes = 0
    cosines = []
    skipped = []
    details = []
    art_path = os.path.join(ROOT, f"gguf_{tag.replace(':', '_').replace('/', '_')}_trits.bin")
    art = open(art_path, "wb")
    try:
        for t in reader.tensors:
            if n >= max_tensors:
                break
            dims = [int(s) for s in t.shape]
            if len(dims) < 2:
                skipped.append({"name": t.name, "reason": "1-D (norm/scale), not a weight matrix"})
                continue
            arr, ttype = dequant_tensor(reader, t)
            if arr is None:
                skipped.append({"name": t.name, "reason": ttype})
                continue
            ne = int(t.n_elements)
            if arr.size != ne:
                skipped.append({"name": t.name, "reason": f"size-mismatch {arr.size} vs {ne}"})
                continue
            if not np.all(np.isfinite(arr)):
                skipped.append({"name": t.name, "reason": f"NaN/inf in dequant ({ttype})"})
                continue
            alpha = float(np.mean(np.abs(arr)))
            if alpha < 1e-12:
                skipped.append({"name": t.name, "reason": "zero tensor"})
                continue
            # ---- balanced ternary {-1,0,+1}, per-tensor mean-abs alpha, +/-0.5 threshold
            tq = arr / alpha
            trits = np.where(tq < -0.5, np.int8(-1), np.where(tq > 0.5, np.int8(1), np.int8(0)))
            codes = (trits.astype(np.uint8) + 1)          # {0,1,2}
            # ---- pack 4 trits/byte (2 bits per trit)
            pad = (-codes.size) % 4
            if pad:
                codes = np.concatenate([codes, np.zeros(pad, dtype=np.uint8)])
            packed = (codes.reshape(-1, 4) * np.array([1, 4, 16, 64], dtype=np.uint8)).sum(axis=1).astype(np.uint8)
            # ---- cosine fidelity on a strided sample (~<=80k points)
            step = max(1, arr.size // 80000)
            a_s = arr[::step]
            t_s = trits[::step].astype(np.float32) * alpha
            cos = float(np.dot(a_s, t_s) / (np.linalg.norm(a_s) * np.linalg.norm(t_s) + 1e-9))
            n_pos = int(np.count_nonzero(codes == 2))
            n_neg = int(np.count_nonzero(codes == 0))
            n_zer = int(codes.size - pad) - n_pos - n_neg
            # ---- write packed trits (4-byte alpha prefix per tensor)
            boff = art.tell()
            art.write(np.float32(alpha).tobytes())
            art.write(packed.tobytes())
            blen = 4 + packed.size
            # ---- stats
            tbytes = (arr.size + 3) // 4
            total_params += arr.size
            total_src_bytes += int(t.n_bytes)
            total_fp32_bytes += arr.size * 4
            total_trit_bytes += tbytes
            cosines.append(cos)
            n += 1
            details.append({
                "name": t.name, "gguf_type": ttype, "dims": dims,
                "n_elements": ne, "src_bytes": int(t.n_bytes),
                "alpha": round(alpha, 6),
                "cosine": round(cos, 4),
                "trit_bytes": tbytes, "packed_bytes": int(packed.size),
                "bin_offset": boff, "bin_len": blen,
                "counts": {"-1": n_neg, "0": n_zer, "+1": n_pos},
            })
            log(f"[{n:3d}] {t.name:48s} {ttype:4s} ne={ne:>10d} alpha={alpha:.5f} "
                f"cos={cos:.4f} tritB={tbytes:>9d} srcB={int(t.n_bytes):>10d} ({time.time()-t0:.1f}s)")
            del arr, tq, trits, codes, packed
    finally:
        art.close()
    elapsed = time.time() - t0
    src_bytes_used = total_src_bytes if total_src_bytes else 1
    result = {
        "ok": True,
        "model_tag": tag,
        "blob": blob,
        "tensors_total": len(reader.tensors),
        "tensors_quantized": n,
        "tensors_skipped": len(skipped),
        "skipped": skipped,
        "params_quantized": total_params,
        "dequant_fp32_bytes": total_fp32_bytes,
        "source_gguf_bytes": total_src_bytes,
        "trit_bytes": total_trit_bytes,
        "compression_vs_fp32": round(total_fp32_bytes / (total_trit_bytes or 1), 2),
        "compression_vs_source_gguf": round(src_bytes_used / (total_trit_bytes or 1), 2),
        "bits_per_weight": round(total_trit_bytes * 8 / total_params, 3) if total_params else None,
        "cosine_mean": round(float(np.mean(cosines)), 4) if cosines else None,
        "cosine_min": round(float(np.min(cosines)), 4) if cosines else None,
        "cosine_max": round(float(np.max(cosines)), 4) if cosines else None,
        "artifact": art_path,
        "artifact_bytes": os.path.getsize(art_path),
        "elapsed_s": round(elapsed, 1),
        "seal": hex(seal(f"gguf-{tag}")),
        "honesty_note": ("GGUF source is already Q4_0/Q6_K pre-quantized; cosine is the REAL "
                         "added loss of ternary vs dequantized fp32 weights; compression_vs_source_gguf "
                         "is vs the original quantized bytes on disk, compression_vs_fp32 is the "
                         "dequantized-fp32 baseline."),
    }
    if details:
        result["tensors"] = details
    ledger = os.path.join(ROOT, f"gguf_{tag.replace(':', '_').replace('/', '_')}_ledger.json")
    with open(ledger, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=1)
    log(json.dumps({k: v for k, v in result.items() if k not in ("skipped", "tensors")}, indent=1))
    return result


if __name__ == "__main__":
    tag = sys.argv[1] if len(sys.argv) > 1 else "tinyllama"
    mx = int(sys.argv[2]) if len(sys.argv) > 2 else 400
    quantize_gguf_model(tag, mx)
