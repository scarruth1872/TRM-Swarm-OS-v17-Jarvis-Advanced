#!/usr/bin/env python3
"""TurboVec ternary pass over a real GGUF, with honest compression baselines.

Why this exists: `swarm_v2/core/quantize_gguf.py` only understands Q4_0/Q6_K/F32, so the fleet's Q4_K_M and Q3_K_M
files could not be measured at all. This reads any of the common block types straight out of the file at the tensor's
own data_offset and applies the TurboVec 1.58-bit recipe:

    alpha = mean|w|          (per tensor)
    trit  = clamp(w/alpha, -1..1) thresholded at +/-0.5   -> {-1, 0, +1}
    packed 2 bits/trit, 4 trits/byte

Three baselines are reported separately, because a single "Nx" number is the classic overclaim:
    compression_vs_source     file bytes -> trit bytes for the tensors actually converted
    compression_vs_fp16_basis params*2B  -> trit bytes (the honest "vs half precision" figure)
    bits_per_weight_after     trit bytes*8 / params

Dequantisation follows llama.cpp ggml-quants.c block layouts exactly (see skill
model-weight-quantization/references/gguf-dequant-algorithms.md). A tensor type this file does not implement is
SKIPPED with its type id recorded - never guessed.

Usage: python tools/turbovec_gguf_apply.py <file.gguf> [--max-tensors N] [--out ledger.json]
"""
from __future__ import annotations

import argparse
import hashlib
import json
import struct
import sys
import time
from pathlib import Path

import numpy as np

MONAD = 0x4D4F4E4144000001
QK_K = 256

# type id -> (block elements, block bytes)
BLOCKS = {
    0: (1, 4), 1: (1, 2), 2: (32, 18), 8: (32, 34), 12: (256, 144), 14: (256, 210), 30: (1, 2),
}
NAMES = {0: "F32", 1: "F16", 2: "Q4_0", 8: "Q8_0", 12: "Q4_K", 14: "Q6_K", 30: "BF16"}
SCALAR = {0: "<B", 1: "<b", 2: "<H", 3: "<h", 4: "<I", 5: "<i", 6: "<f", 7: "<?", 10: "<Q", 11: "<q", 12: "<d"}


def seal(text: str) -> int:
    h = int(hashlib.sha256(text.encode()).hexdigest()[:16], 16)
    return (MONAD ^ h) & 0xFFFFFFFFFFFFFFFF


# ── GGUF container ────────────────────────────────────────────────────────────
def read_str(f) -> str:
    n = struct.unpack("<Q", f.read(8))[0]
    return f.read(n).decode("utf-8", "replace")


def skip_value(f, vtype: int) -> None:
    if vtype == 8:
        read_str(f)
    elif vtype == 9:
        itype, count = struct.unpack("<IQ", f.read(12))
        if itype == 8:
            for _ in range(count):
                read_str(f)
        else:
            f.seek(struct.calcsize(SCALAR[itype]) * count, 1)
    else:
        f.seek(struct.calcsize(SCALAR[vtype]), 1)


def read_gguf(path: Path) -> dict:
    with path.open("rb") as f:
        if f.read(4) != b"GGUF":
            raise SystemExit("not GGUF")
        version, n_tensors, n_kv = struct.unpack("<IQQ", f.read(20))
        meta = {}
        for _ in range(n_kv):
            key = read_str(f)
            vtype = struct.unpack("<I", f.read(4))[0]
            if key in ("general.alignment", "general.file_type"):
                meta[key] = struct.unpack(SCALAR[vtype], f.read(struct.calcsize(SCALAR[vtype])))[0]
            else:
                skip_value(f, vtype)
        tensors = []
        for _ in range(n_tensors):
            name = read_str(f)
            n_dims = struct.unpack("<I", f.read(4))[0]
            dims = struct.unpack("<" + "Q" * n_dims, f.read(8 * n_dims))
            ttype, off = struct.unpack("<IQ", f.read(12))
            n_elem = 1
            for d in dims:
                n_elem *= d
            tensors.append({"name": name, "dims": dims, "type": ttype, "offset": off, "elements": n_elem})
        align = int(meta.get("general.alignment", 32) or 32)
        pos = f.tell()
        data_start = (pos + align - 1) // align * align
    return {"version": version, "tensors": tensors, "data_start": data_start, "meta": meta,
            "n_tensors": n_tensors}


# ── dequantisation (1:1 with ggml-quants.c) ───────────────────────────────────
def _get_scale_min_k4(j: int, scales: np.ndarray):
    if j < 4:
        sc = (scales[:, j] & 63).astype(np.float32).reshape(-1)
        m = (scales[:, j + 4] & 63).astype(np.float32).reshape(-1)
    else:
        sc = ((scales[:, j + 4] & 0x0F) | ((scales[:, j - 4] >> 6) << 4)).astype(np.float32).reshape(-1)
        m = ((scales[:, j + 4] >> 4) | ((scales[:, j] >> 6) << 4)).astype(np.float32).reshape(-1)
    return sc, m


def dequant(ttype: int, raw: bytes, n_elem: int) -> np.ndarray:
    bs, bb = BLOCKS[ttype]
    buf = np.frombuffer(raw, dtype=np.uint8)
    nblk = n_elem // bs
    b = buf[: nblk * bb].reshape(nblk, bb)
    if ttype == 0:
        return np.frombuffer(raw, dtype="<f4", count=n_elem).astype(np.float32)
    if ttype == 1:
        return np.frombuffer(raw, dtype="<f2", count=n_elem).astype(np.float32)
    if ttype == 30:  # bf16 -> f32 by shifting the exponent into place
        u = np.frombuffer(raw, dtype="<u2", count=n_elem).astype(np.uint32) << 16
        return u.view(np.float32)
    if ttype == 2:   # Q4_0
        d = b[:, 0:2].copy().view("<f2").astype(np.float32).reshape(-1)
        nib = b[:, 2:18]
        lo = (nib & 0x0F).astype(np.float32) - 8.0
        hi = ((nib >> 4) & 0x0F).astype(np.float32) - 8.0
        out = np.empty((nblk, 32), np.float32)
        out[:, 0::2] = lo
        out[:, 1::2] = hi
        return (out * d[:, None]).reshape(-1)
    if ttype == 8:   # Q8_0
        d = b[:, 0:2].copy().view("<f2").astype(np.float32).reshape(-1)
        qs = b[:, 2:34].astype(np.int8).astype(np.float32)
        return (qs * d[:, None]).reshape(-1)
    if ttype == 12:  # Q4_K
        d = b[:, 0:2].copy().view("<f2").astype(np.float32).reshape(-1)
        dmin = b[:, 2:4].copy().view("<f2").astype(np.float32).reshape(-1)
        scales = b[:, 4:16]
        qs = b[:, 16:144]
        out = np.empty((nblk, QK_K), np.float32)
        is_ = 0
        for g in range(4):
            sc1, m1 = _get_scale_min_k4(is_, scales)
            sc2, m2 = _get_scale_min_k4(is_ + 1, scales)
            d1 = (d * sc1).reshape(-1, 1)
            mm1 = (dmin * m1).reshape(-1, 1)
            d2 = (d * sc2).reshape(-1, 1)
            mm2 = (dmin * m2).reshape(-1, 1)
            qg = qs[:, g * 32:(g + 1) * 32]
            lo = (qg & 0x0F).astype(np.float32)
            hi = (qg >> 4).astype(np.float32)
            out[:, g * 64:g * 64 + 32] = d1 * lo - mm1
            out[:, g * 64 + 32:g * 64 + 64] = d2 * hi - mm2
            is_ += 2
        return out.reshape(-1)
    if ttype == 14:  # Q6_K
        # MEASURED 2026-11-13: ggml's block_q6_K puts the super-block scale d LAST:
        #   [ql 128][qh 64][scales 16 int8][d 2]   -- total 210
        # Reading d from [0:2] (as the skill reference stated) yields median|d| = 0.44 and 24,228 non-finite
        # scales; reading it from [208:210] yields median |d| = 2e-5, the same magnitude as a known-good Q4_K
        # block on the same model. The order below is the one that measures sane.
        ql = b[:, 0:128]
        qh = b[:, 128:192]
        sc = b[:, 192:208].copy().view(np.int8).astype(np.float32)
        d = b[:, 208:210].copy().view("<f2").astype(np.float32).reshape(-1)
        out = np.empty((nblk, QK_K), np.float32)
        # llama.cpp walks 128 weights at a time: ql advances 64 bytes, qh 32, sc 8.
        for h in (0, 1):
            base = h * 128
            l0 = ql[:, h * 64:h * 64 + 32]
            l32 = ql[:, h * 64 + 32:h * 64 + 64]
            qhh = qh[:, h * 32:h * 32 + 32]
            sch = sc[:, h * 8:h * 8 + 8]
            q1 = ((l0 & 0x0F) | (((qhh >> 0) & 3) << 4)).astype(np.float32) - 32
            q2 = ((l32 & 0x0F) | (((qhh >> 2) & 3) << 4)).astype(np.float32) - 32
            q3 = ((l0 >> 4) | (((qhh >> 4) & 3) << 4)).astype(np.float32) - 32
            q4 = ((l32 >> 4) | (((qhh >> 6) & 3) << 4)).astype(np.float32) - 32
            for k, q in enumerate((q1, q2, q3, q4)):
                s_ = sch[:, k * 2:k * 2 + 2].repeat(16, axis=1).reshape(nblk, 32)
                out[:, base + k * 32: base + k * 32 + 32] = q * s_ * d.reshape(-1, 1)
        return out.reshape(-1)
    raise NotImplementedError(ttype)


# ── TurboVec ternary ──────────────────────────────────────────────────────────
def to_ternary(w: np.ndarray) -> tuple[np.ndarray, float]:
    alpha = float(np.mean(np.abs(w)))
    if not np.isfinite(alpha) or alpha < 1e-12:
        return np.zeros_like(w, dtype=np.int8), 0.0
    t = w / alpha
    return np.where(t < -0.5, -1, np.where(t > 0.5, 1, 0)).astype(np.int8), alpha


def pack_trits(trits: np.ndarray) -> bytes:
    codes = (trits + 1).astype(np.uint8)          # {-1,0,1} -> {0,1,2}
    pad = (-len(codes)) % 4
    if pad:
        codes = np.concatenate([codes, np.zeros(pad, np.uint8)])
    q = codes.reshape(-1, 4).astype(np.uint16)
    packed = (q[:, 0] + q[:, 1] * 4 + q[:, 2] * 16 + q[:, 3] * 64).astype(np.uint8)
    return packed.tobytes()


def cosine(a: np.ndarray, b: np.ndarray, max_samples: int = 200_000) -> float:
    step = max(1, a.size // max_samples)
    x, y = a[::step].astype(np.float64), b[::step].astype(np.float64)
    nx, ny = np.linalg.norm(x), np.linalg.norm(y)
    if nx == 0 or ny == 0:
        return 0.0
    return float(np.dot(x, y) / (nx * ny))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("gguf")
    ap.add_argument("--max-tensors", type=int, default=60, help="0 = every 2-D tensor")
    ap.add_argument("--out", default="")
    args = ap.parse_args()

    path = Path(args.gguf)
    g = read_gguf(path)
    t0 = time.time()
    rows, skipped, per_type = [], {}, {}
    src_bytes = trit_bytes = params = 0
    cos_all = []

    with path.open("rb") as f:
        for t in g["tensors"]:
            if len(t["dims"]) != 2 or t["type"] not in BLOCKS:
                key = f"type_{t['type']}" if t["type"] not in BLOCKS else "1d"
                skipped[key] = skipped.get(key, 0) + 1
                continue
            if args.max_tensors and len(rows) >= args.max_tensors:
                break
            bs, bb = BLOCKS[t["type"]]
            nbytes = (t["elements"] // bs) * bb
            f.seek(g["data_start"] + t["offset"])
            raw = f.read(nbytes)
            try:
                w = dequant(t["type"], raw, t["elements"])
            except NotImplementedError:
                skipped[f"type_{t['type']}"] = skipped.get(f"type_{t['type']}", 0) + 1
                continue
            if not np.all(np.isfinite(w)):
                skipped["nonfinite"] = skipped.get("nonfinite", 0) + 1
                continue
            trits, alpha = to_ternary(w)
            rec = alpha * trits.astype(np.float32)
            cos = cosine(w, rec)
            packed = pack_trits(trits)
            nz = int(np.count_nonzero(trits))
            rows.append({"name": t["name"], "type": NAMES[t["type"]], "elements": int(t["elements"]),
                         "source_bytes": nbytes, "trit_bytes": len(packed), "alpha": alpha,
                         "cosine": round(cos, 5), "nonzero_pct": round(100 * nz / trits.size, 2)})
            cos_all.append(cos)
            src_bytes += nbytes
            trit_bytes += len(packed)
            params += int(t["elements"])
            per_type[NAMES[t["type"]]] = per_type.get(NAMES[t["type"]], 0) + nbytes

    if not rows:
        print("nothing convertible - every tensor was skipped; see the skip reasons", file=sys.stderr)
        print(json.dumps(skipped, indent=2))
        return 2

    ledger = {
        "protocol": "TURBOVEC_TERNARY_GGUF",
        "monad_anchor": hex(MONAD),
        "source_file": str(path),
        "source_bytes_total": path.stat().st_size,
        "tensors_converted": len(rows),
        "skipped": skipped,
        "params_quantized": params,
        "source_bytes_converted": src_bytes,
        "trit_bytes": trit_bytes,
        "compression_vs_source": round(src_bytes / trit_bytes, 3),
        "compression_vs_fp16_basis": round(params * 2 / trit_bytes, 3),
        "bits_per_weight_after": round(trit_bytes * 8 / params, 3),
        "cosine_mean": round(float(np.mean(cos_all)), 5),
        "cosine_min": round(float(np.min(cos_all)), 5),
        "cosine_worst_tensor": min(rows, key=lambda r: r["cosine"])["name"],
        "source_bytes_by_type": per_type,
        "elapsed_s": round(time.time() - t0, 1),
        "tensors": rows[:400],
    }
    ledger["seal"] = hex(seal(json.dumps({k: v for k, v in ledger.items() if k != "seal"}, sort_keys=True)))
    out = Path(args.out) if args.out else path.with_suffix(".turbovec_ledger.json")
    out.parent.mkdir(parents=True, exist_ok=True)   # a redirected ledger dir does not exist yet
    out.write_text(json.dumps(ledger, indent=2), encoding="utf-8")
    print(json.dumps({k: v for k, v in ledger.items() if k != "tensors"}, indent=2))
    print(f"ledger -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
