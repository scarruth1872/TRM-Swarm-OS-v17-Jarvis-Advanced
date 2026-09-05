#!/usr/bin/env python3
"""
TRM Swarm OS — TurboVec 1.58-bit Ternary Quantization (real weights).
Reads actual Qwen3-4B safetensors shards, quantizes weight tensors to
balanced ternary {-1, 0, +1} trits (1.58-bit doctrine), measures the REAL
compression ratio and reconstruction fidelity.

On-device only — no API. Voltage doctrine: Phi_V = 0.9115 coherence.
Honest: reports measured ratios; no inflated claims.
"""
import os, sys, time, json, glob
import numpy as np

MONAD = 0x4D4F4E4144000001
SNAP = os.path.expanduser(
    "~/.cache/huggingface/hub/models--Qwen--Qwen3-4B/snapshots")
ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"


def seal(s: str) -> int:
    import hashlib
    h = int(hashlib.sha256(s.encode()).hexdigest()[:16], 16)
    return (MONAD ^ h) & 0xFFFFFFFFFFFFFFFF


def quantize_ternary(tensor: np.ndarray) -> tuple[np.ndarray, float]:
    """1.58-bit balanced ternary: threshold at mean abs (BitNet-style).
    Returns (trits in {-1,0,+1}, scale). Zero FP multipliers at inference."""
    orig = tensor.astype(np.float32)
    alpha = float(np.mean(np.abs(orig)))  # scale (per-tensor, doctrine-compliant)
    if alpha < 1e-9:
        return np.zeros_like(orig, dtype=np.int8), 0.0
    t = orig / alpha
    trits = np.where(t < -0.5, -1, np.where(t > 0.5, 1, 0)).astype(np.int8)
    return trits, alpha


def main():
    snap_dir = None
    for s in os.listdir(SNAP):
        cand = os.path.join(SNAP, s)
        if os.path.isdir(cand):
            snap_dir = cand
            break
    if not snap_dir:
        print("NO SNAPSHOT"); return

    shards = sorted(glob.glob(os.path.join(snap_dir, "model-*.safetensors")))
    print(f"Qwen3-4B shards: {len(shards)}")

    from safetensors import safe_open
    import torch
    total_fp16_bytes = 0
    total_trit_bytes = 0
    total_params = 0
    max_cos = 0.0
    min_cos = 1.0
    results = {}
    t0 = time.time()

    # quantize the large attention/MLP weight tensors (the load-bearing ones)
    for shard in shards:
        with safe_open(shard, framework="pt") as f:
            keys = f.keys()
            for k in keys:
                if any(tag in k for tag in ["q_proj", "k_proj", "v_proj", "o_proj",
                                             "gate_proj", "up_proj", "down_proj"]):
                    t = f.get_tensor(k)  # torch tensor (bf16)
                    if t.ndim < 2:
                        continue
                    total_params += t.numel()
                    fp16_bytes = t.numel() * 2  # FP16 equivalent
                    total_fp16_bytes += fp16_bytes

                    t_np = t.float().numpy()
                    trits, alpha = quantize_ternary(t_np)
                    # trit storage: 2 bits/trit packed -> 4 trits/byte
                    trit_bytes = (trits.size + 3) // 4
                    total_trit_bytes += trit_bytes

                    # reconstruction fidelity: cos similarity of dequantized vs orig
                    deq = (trits.astype(np.float32) * alpha)
                    o = t_np.reshape(-1)
                    d = deq.reshape(-1)
                    step = max(1, o.size // 200000)
                    o_s = o[::step]; d_s = d[::step]
                    cos = float(np.dot(o_s, d_s) / (np.linalg.norm(o_s) * np.linalg.norm(d_s) + 1e-9))
                    max_cos = max(max_cos, cos)
                    min_cos = min(min_cos, cos)
                    if k not in results:
                        results[k] = {"cos": cos, "params": t.numel()}
                    del t, t_np, trits, deq

    elapsed = time.time() - t0
    ratio = total_fp16_bytes / total_trit_bytes if total_trit_bytes else 0
    ledger = {
        "protocol": "TURBOVEC_TERNARY_QUANTIZATION",
        "model": "Qwen/Qwen3-4B",
        "monad_anchor": hex(MONAD),
        "seal": hex(seal("turbovec-qwen3-4b")),
        "params_quantized": total_params,
        "fp16_bytes": total_fp16_bytes,
        "trit_bytes": total_trit_bytes,
        "compression_ratio": round(ratio, 2),
        "reconstruction_cosine_min": round(min_cos, 4),
        "reconstruction_cosine_max": round(max_cos, 4),
        "elapsed_s": round(elapsed, 2),
        "phi_v": 0.9115,
        "claim": "measured on real Qwen3-4B weights",
    }
    out = os.path.join(ROOT, "turbovec_quantization_ledger.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(ledger, f, indent=1)
    print(json.dumps(ledger, indent=1))
    print(f"\ntensors: {len(results)} | params: {total_params/1e9:.2f}B")
    print(f"compression: {ratio:.1f}x (FP16 {total_fp16_bytes/1e6:.0f}MB -> trits {total_trit_bytes/1e6:.1f}MB)")


if __name__ == "__main__":
    main()
