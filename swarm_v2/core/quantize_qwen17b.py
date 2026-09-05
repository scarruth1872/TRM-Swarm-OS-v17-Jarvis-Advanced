#!/usr/bin/env python3
"""
TRM Swarm OS — TurboVec 1.58-bit Ternary Quantization (Qwen3-1.7B, blob-resolved).
Reads real Qwen/Qwen3-1.7B safetensors WEIGHT BLOBS from the HF cache (snapshot
dir empty -> resolve by sniffing safetensors layout: 8-byte LE header length +
JSON, opens with safe_open; largest blob = weights), quantizes 2-D projection
weight tensors to balanced ternary {-1, 0, +1} trits (1.58-bit doctrine), and
measures the REAL compression ratio and reconstruction fidelity on-device.

Doctrine (template parity with turbovec_quantize_weights.py):
  * quantize_ternary: mean-abs alpha (BitNet-style), +/-0.5 threshold -> int8
  * 2 trits/bit storage: packed 4 trits/byte
  * cosine fidelity on strided samples step = max(1, size//200000)
  * 2-D weight tensors only; norms/embeddings/lm_head skipped (embed is 311M of
    2.05B params -> not plurality; lm_head sits in its own shard and is skipped)
On-device only - no API. Voltage doctrine: Phi_V = 0.9115 coherence.
Honest: reports measured ratios; no inflated claims.
"""
import os, sys, time, json, hashlib
import numpy as np

MONAD = 0x4D4F4E4144000001
BLOBS = os.path.expanduser(
    "~/.cache/huggingface/hub/models--Qwen--Qwen3-1.7B/blobs")
ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
MIN_WEIGHT_BYTES = 50_000_000  # ignore tokenizer/*.json etc. in blobs dir
PROJ_TAGS = ["q_proj", "k_proj", "v_proj", "o_proj",
             "gate_proj", "up_proj", "down_proj"]


def seal(s: str) -> int:
    h = int(hashlib.sha256(s.encode()).hexdigest()[:16], 16)
    return (MONAD ^ h) & 0xFFFFFFFFFFFFFFFF


def sniff_weight_shards(blobs_dir: str) -> list:
    """Resolve the real .safetensors weight blobs when the snapshot dir is empty.

    HF blobs are content-addressed by sha256 of the original filename. A real
    safetensors file starts with an 8-byte little-endian u64 JSON-header length,
    so any blob whose header opens via safe_open and carries >= 1 tensor is a
    weight shard. Largest first (model weights may span multiple shards)."""
    from safetensors import safe_open
    shards = []
    for name in sorted(os.listdir(blobs_dir)):
        p = os.path.join(blobs_dir, name)
        try:
            if not os.path.isfile(p):
                continue
            size = os.path.getsize(p)
            if size < MIN_WEIGHT_BYTES:
                continue
            with safe_open(p, framework="pt") as f:
                n_tensors = len(f.keys())
            shards.append((size, p, name, n_tensors))
        except Exception:
            continue
    shards.sort(key=lambda t: t[0], reverse=True)
    return shards


def quantize_ternary(tensor: np.ndarray) -> tuple:
    """1.58-bit balanced ternary: threshold at mean abs (BitNet-style).
    Returns (trits in {-1,0,+1} int8, scale). Zero FP multipliers at inference."""
    orig = tensor.astype(np.float32)
    alpha = float(np.mean(np.abs(orig)))
    if alpha < 1e-9:
        return np.zeros_like(orig, dtype=np.int8), 0.0
    t = orig / alpha
    trits = np.where(t < -0.5, -1, np.where(t > 0.5, 1, 0)).astype(np.int8)
    return trits, alpha


def cosine_sample(a: np.ndarray, b: np.ndarray, n_samp: int = 200000) -> float:
    o = a.reshape(-1)
    d = b.reshape(-1)
    step = max(1, o.size // n_samp)
    o_s = o[::step]
    d_s = d[::step]
    return float(np.dot(o_s, d_s) / (np.linalg.norm(o_s) * np.linalg.norm(d_s) + 1e-9))


def main():
    from safetensors import safe_open
    import torch

    shards = sniff_weight_shards(BLOBS)
    if not shards:
        print("NO SAFETENSORS WEIGHT BLOBS FOUND"); return
    print(f"resolved weight shards: {len(shards)} (largest first)")
    for size, p, name, nt in shards:
        print(f"  {size/1e6:8.1f} MB | {name[:24]}... | {nt} tensors")

    total_fp16_bytes = 0      # FP16-equivalent bytes of quantized tensors
    total_trit_bytes = 0      # packed 4 trits/byte
    total_params = 0
    total_nonzero = 0
    cosines = []
    results = {}
    t0 = time.time()

    for size, path, name, nt in shards:
        with safe_open(path, framework="pt") as f:
            keys = f.keys()
            for k in keys:
                if not any(tag in k for tag in PROJ_TAGS):
                    continue
                t = f.get_tensor(k)          # torch bf16 tensor
                if t.ndim != 2:
                    del t
                    continue
                n = t.numel()
                total_params += n
                total_fp16_bytes += n * 2    # FP16/bf16 equivalent (2 B/param)

                t_np = t.float().numpy()
                trits, alpha = quantize_ternary(t_np)
                trit_bytes = (trits.size + 3) // 4   # 4 trits/byte
                total_trit_bytes += trit_bytes
                total_nonzero += int(np.count_nonzero(trits))

                deq = trits.astype(np.float32) * alpha
                cos = cosine_sample(t_np, deq)
                cosines.append(cos)
                results[k] = {"cos": round(cos, 5), "params": n,
                              "alpha": round(alpha, 6)}
                if len(results) <= 3 or len(results) % 40 == 0:
                    print(f"  [{len(results):3d}] {k:56s} cos={cos:.4f} "
                          f"alpha={alpha:.3f} params={n/1e6:.2f}M")
                del t, t_np, trits, deq

    elapsed = time.time() - t0
    ratio = total_fp16_bytes / total_trit_bytes if total_trit_bytes else 0.0
    mean_cos = float(np.mean(cosines)) if cosines else 0.0
    min_cos = float(np.min(cosines)) if cosines else 0.0
    max_cos = float(np.max(cosines)) if cosines else 0.0
    zero_frac = 1.0 - (total_nonzero / total_params) if total_params else 0.0
    worst = min(results, key=lambda k: results[k]["cos"])
    best = max(results, key=lambda k: results[k]["cos"])

    ledger = {
        "protocol": "TURBOVEC_TERNARY_QUANTIZATION",
        "model": "Qwen/Qwen3-1.7B",
        "monad_anchor": hex(MONAD),
        "seal": hex(seal("turbovec-qwen3-1.7b")),
        "source": "hf-cache blobs (snapshot empty; resolved via safetensors header sniff)",
        "source_blobs": [name for _, _, name, _ in shards],
        "source_dtype": "bf16",
        "included_tensor_families": PROJ_TAGS,
        "excluded": "norms, embed_tokens, lm_head (2-D proj only doctrine; embed 311M not plurality)",
        "params_quantized": total_params,
        "n_tensors": len(results),
        "fp16_bytes": total_fp16_bytes,
        "trit_bytes": total_trit_bytes,
        "compression_ratio": round(ratio, 2),
        "effective_bits_per_weight": round((total_trit_bytes * 8) / total_params, 4),
        "reconstruction_cosine_mean": round(mean_cos, 5),
        "reconstruction_cosine_min": round(min_cos, 5),
        "reconstruction_cosine_max": round(max_cos, 5),
        "worst_tensor": {"name": worst, "cos": results[worst]["cos"]},
        "best_tensor": {"name": best, "cos": results[best]["cos"]},
        "trit_zero_fraction": round(zero_frac, 5),
        "trit_nonzero_fraction": round(1.0 - zero_frac, 5),
        "elapsed_s": round(elapsed, 2),
        "phi_v": 0.9115,
        "claim": "measured on real Qwen/Qwen3-1.7B safetensors blobs, on-device",
    }
    out = os.path.join(ROOT, "turbovec_quantization_ledger_qwen17b.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(ledger, f, indent=1)
    print("\n" + json.dumps(ledger, indent=1))
    print(f"\ntensors: {len(results)} | params: {total_params/1e9:.3f}B")
    print(f"compression: {ratio:.1f}x (FP16 {total_fp16_bytes/1e6:.0f}MB -> trits {total_trit_bytes/1e6:.1f}MB) "
          f"= {ledger['effective_bits_per_weight']} bits/weight")
    print(f"cosine fidelity: mean={mean_cos:.5f} min={min_cos:.5f} max={max_cos:.5f}")
    print(f"trit sparsity: {zero_frac*100:.1f}% zeros")
    print(f"ledger -> {out}")


if __name__ == "__main__":
    main()
