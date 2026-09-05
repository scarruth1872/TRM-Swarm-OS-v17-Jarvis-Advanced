#!/usr/bin/env python3
"""
TRM Swarm OS — Instella-MoE-16B expert quantization (council d0eae153).
Quantizes a REPRESENTATIVE SAMPLE of Instella's 5,070 expert weights to
TurboVec 1.58-bit ternary. Reports measured compression + fidelity.
Full 31.7GB pass is multi-hour; sample gives the honest character.
On-device, no API.
"""
import os, sys, time, json, glob, hashlib
import numpy as np

MONAD = 0x4D4F4E4144000001
INST = r"F:\models\instella-moe-think"
ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
SAMPLE_EXPERTS = 40   # expert tensors to quantize (spread across layers)


def seal(s: str) -> int:
    h = int(hashlib.sha256(s.encode()).hexdigest()[:16], 16)
    return (MONAD ^ h) & 0xFFFFFFFFFFFFFFFF


def quantize_ternary(t_np: np.ndarray):
    orig = t_np.astype(np.float32)
    alpha = float(np.mean(np.abs(orig)))
    if alpha < 1e-9:
        return np.zeros_like(orig, dtype=np.int8), 0.0
    t = orig / alpha
    trits = np.where(t < -0.5, -1, np.where(t > 0.5, 1, 0)).astype(np.int8)
    return trits, alpha


def main():
    import torch
    from safetensors import safe_open
    shards = sorted(glob.glob(os.path.join(INST, "model-*.safetensors")))
    print(f"Instella shards: {len(shards)} | sampling {SAMPLE_EXPERTS} expert tensors")

    expert_keys = []
    # first collect expert key names from shard 1's index
    idx = json.load(open(os.path.join(INST, "model.safetensors.index.json"), encoding="utf-8"))
    wm = idx["weight_map"]
    for k, sh in wm.items():
        if "experts" in k and ("gate_proj" in k or "up_proj" in k or "down_proj" in k):
            expert_keys.append(k)
    print(f"expert weight tensors total: {len(expert_keys)}")
    import random
    random.seed(42)
    sample = random.sample(expert_keys, min(SAMPLE_EXPERTS, len(expert_keys)))

    # group sample keys by shard
    by_shard = {}
    for k in sample:
        sh = wm[k]
        by_shard.setdefault(sh, []).append(k)

    total_params = 0
    total_fp16 = 0
    total_trit = 0
    cosines = []
    t0 = time.time()
    for sh in sorted(by_shard):
        spath = os.path.join(INST, sh)
        with safe_open(spath, framework="pt") as f:
            for k in by_shard[sh]:
                t = f.get_tensor(k)
                if t.ndim < 2:
                    continue
                t_np = t.float().numpy()
                total_params += t.numel()
                total_fp16 += t.numel() * 2
                trits, alpha = quantize_ternary(t_np)
                total_trit += (trits.size + 3) // 4
                deq = trits.astype(np.float32) * alpha
                o = t_np.reshape(-1); d = deq.reshape(-1)
                step = max(1, o.size // 100000)
                cos = float(np.dot(o[::step], d[::step]) /
                            (np.linalg.norm(o[::step]) * np.linalg.norm(d[::step]) + 1e-9))
                cosines.append(cos)
                del t, t_np, trits, deq

    elapsed = time.time() - t0
    ratio = total_fp16 / total_trit if total_trit else 0
    ledger = {
        "protocol": "INSTELLA_EXPERT_TERNARY_QUANTIZATION",
        "model": "amd/Instella-MoE-16B-A3B-Think",
        "tensors_sampled": len(sample),
        "tensors_total_experts": len(expert_keys),
        "params_quantized": total_params,
        "fp16_bytes": total_fp16,
        "trit_bytes": total_trit,
        "compression_ratio": round(ratio, 2),
        "cosine_mean": round(float(np.mean(cosines)), 4) if cosines else None,
        "cosine_min": round(float(np.min(cosines)), 4) if cosines else None,
        "cosine_max": round(float(np.max(cosines)), 4) if cosines else None,
        "elapsed_s": round(elapsed, 1),
        "seal": hex(seal("instella-experts-sample")),
        "claim": "measured on real Instella expert weights (40-sample)",
    }
    out = os.path.join(ROOT, "instella_quantization_ledger.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(ledger, f, indent=1)
    print(json.dumps(ledger, indent=1))


if __name__ == "__main__":
    main()
