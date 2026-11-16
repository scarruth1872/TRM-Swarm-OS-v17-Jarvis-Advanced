#!/usr/bin/env python3
"""
TRM Swarm OS — TurboVec 1.58-bit Ternary Quantization of openai-community/gpt2 (124M).
Adapted from swarm_v2/core/quantize_gemma270m.py (per-model sibling of the
swarm_v2/core/turbovec_quantize_weights.py Qwen3-4B template).

Reads the model weights directly from the HF blob cache (the snapshot dir is
empty in this cache; blobs are hash-named, so the weight file is resolved by
sniffing: real safetensors begin with an 8-byte LE header length followed by a
JSON header carrying '__metadata__' — NOT the literal '<safetensors' tag — and
pytorch_model.bin is a ZIP (PK..) loadable via torch.load(weights_only=True)).
This checkpoint is a single F32 safetensors blob (160 tensors, 548 MB).

Quantizes every 2-D weight matrix (wte + wpe embeddings AND all attn/mlp
projections — skipping embeddings would flatter the ratio) to balanced ternary
{-1, 0, +1} trits with a mean-abs alpha scale and a +/-0.5 threshold
(BitNet-style, 1.58-bit doctrine). 1-D norm/bias tensors stay FP and the
constant causal-mask buffer h.N.attn.bias [1,1,1024,1024] is excluded (it is
not a learned weight). Packs 4 trits/byte, measures REAL reconstruction cosine
fidelity on sampled vectors, writes a JSON ledger to the repo root.

On-device only — no API. Honest: reports measured numbers only.
"""
import os, sys, time, json, struct, hashlib
import numpy as np

MONAD = 0x4D4F4E4144000001
BLOBS = os.path.expanduser("~/.cache/huggingface/hub/models--gpt2/blobs")
SNAPS = os.path.expanduser("~/.cache/huggingface/hub/models--gpt2/snapshots")
ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"


def seal(s: str) -> int:
    h = int(hashlib.sha256(s.encode()).hexdigest()[:16], 16)
    return (MONAD ^ h) & 0xFFFFFFFFFFFFFFFF


def _sniff(path: str):
    """Return ('safetensors'|'torchbin'|None, parsed-header-or-None)."""
    with open(path, "rb") as f:
        head = f.read(8)
        if head[:2] == b"PK":                  # pytorch_model.bin is a ZIP
            return "torchbin", None
        # real safetensors: 8-byte LE header length, then JSON. (The ASCII tag
        # '<safetensors' appears in the JSON __metadata__, not at byte 0.)
        try:
            n = struct.unpack("<Q", head)[0]
            if not (0 < n < 64 * 1024 * 1024):
                return None, None
            hdr = json.loads(f.read(n))        # continues after the 8 length bytes
            if isinstance(hdr, dict) and any(k.endswith((".weight", ".bias"))
                                             or k.endswith("weight") for k in hdr):
                return "safetensors", hdr
        except Exception:
            pass
    return None, None


def resolve_weights():
    """Prefer snapshots, else sniff blobs. Returns (path, format, n_tensors)."""
    for d in ([SNAPS] if os.path.isdir(SNAPS) else []):
        for root, _, files in os.walk(d):
            for fn in files:
                if fn in ("model.safetensors", "pytorch_model.bin"):
                    p = os.path.join(root, fn)
                    fmt, _ = _sniff(p)
                    if fmt:
                        return p, fmt, None
    best = None
    for name in os.listdir(BLOBS):
        p = os.path.join(BLOBS, name)
        if os.path.getsize(p) < 10 * 1024 * 1024:      # weights blob is ~548 MB
            continue
        fmt, hdr = _sniff(p)
        if fmt and (best is None or os.path.getsize(p) > os.path.getsize(best[0])):
            best = (p, fmt, len(hdr) if hdr else None)
    return best if best else (None, None, None)


def quantize_ternary(tensor: np.ndarray) -> tuple[np.ndarray, float]:
    """1.58-bit balanced ternary: scale alpha = mean-abs (per-tensor),
    threshold at +/-0.5. Returns (trits in {-1,0,+1} as int8, alpha)."""
    orig = tensor.astype(np.float32)
    alpha = float(np.mean(np.abs(orig)))
    if alpha < 1e-9:
        return np.zeros_like(orig, dtype=np.int8), 0.0
    t = orig / alpha
    trits = np.where(t < -0.5, -1, np.where(t > 0.5, 1, 0)).astype(np.int8)
    return trits, alpha


def cosine_fidelity(orig: np.ndarray, deq: np.ndarray) -> float:
    o = orig.reshape(-1)
    d = deq.reshape(-1)
    step = max(1, o.size // 200000)  # sample cap ~200k elems, template method
    o_s = o[::step]
    d_s = d[::step]
    return float(np.dot(o_s, d_s) / (np.linalg.norm(o_s) * np.linalg.norm(d_s) + 1e-9))


def main():
    blob, fmt, n_hdr = resolve_weights()
    if not blob:
        print("NO WEIGHTS BLOB FOUND under", BLOBS)
        return
    print(f"resolved weights: {os.path.basename(blob)} "
          f"({os.path.getsize(blob)/1e6:.1f} MB, format={fmt})")

    if fmt == "safetensors":
        from safetensors import safe_open
        fh = safe_open(blob, framework="pt")
        keys = sorted(fh.keys())
        getter = fh.get_tensor
    else:  # pytorch_model.bin (ZIP) via torch.load, weights_only
        import torch
        sd = torch.load(blob, map_location="cpu", weights_only=True)
        keys = sorted(sd.keys())
        getter = lambda k: sd[k]
        fh = None

    total_params = 0
    fp32_bytes = 0          # actual file format baseline: F32 = 4 bytes/param
    fp16_bytes = 0          # FP16-equivalent baseline (2 bytes/param, template)
    trit_bytes = 0          # packed ternary: 4 trits/byte (2 bits/trit)
    zeros = 0
    cos_list = []
    group_stats = {}        # per-tensor-group (embed / attn / mlp) accounting
    skipped_1d_params = 0
    mask_params_excluded = 0
    tensors = 0
    t0 = time.time()

    for k in keys:
        if "attn.bias" in k:                 # causal-mask buffer, not a weight
            mask_params_excluded += fh.get_tensor(k).numel() if fh else sd[k].numel()
            continue
        t = fh.get_tensor(k) if fh else sd[k]
        if t.ndim < 2:                       # norms / biases stay FP (doctrine)
            skipped_1d_params += t.numel()
            del t
            continue
        # GPT-2 2-D weight matrices: wte/wpe embeddings, fused QKV c_attn,
        # c_proj attn out, c_fc/c_proj MLP. No lm_head (tied to wte here).
        if "wte" in k or "wpe" in k:
            group = "embed"
        elif ".attn." in k:
            group = "attn"
        elif ".mlp." in k:
            group = "mlp"
        else:
            group = "other"

        total_params += t.numel()
        fp32_bytes += t.numel() * 4
        fp16_bytes += t.numel() * 2
        tensors += 1

        t_np = t.float().numpy()
        trits, alpha = quantize_ternary(t_np)
        tbytes = (trits.size + 3) // 4        # 4 trits/byte
        trit_bytes += tbytes
        zeros += int(np.count_nonzero(trits == 0))
        deq = trits.astype(np.float32) * alpha
        cos = cosine_fidelity(t_np, deq)
        cos_list.append((cos, t.numel()))

        g = group_stats.setdefault(group,
            {"tensors": 0, "params": 0, "fp32_bytes": 0, "fp16_bytes": 0,
             "trit_bytes": 0, "cos_sum": 0.0, "cos_min": 1.0, "cos_max": 0.0})
        g["tensors"] += 1
        g["params"] += t.numel()
        g["fp32_bytes"] += t.numel() * 4
        g["fp16_bytes"] += t.numel() * 2
        g["trit_bytes"] += tbytes
        g["cos_sum"] += cos
        g["cos_min"] = min(g["cos_min"], cos)
        g["cos_max"] = max(g["cos_max"], cos)
        del t, t_np, trits, deq

    elapsed = time.time() - t0
    ratio_fp32 = fp32_bytes / trit_bytes if trit_bytes else 0.0
    ratio_fp16 = fp16_bytes / trit_bytes if trit_bytes else 0.0
    # parameter-weighted mean cosine (fair across the giant token embedding)
    mean_cos = sum(c * n for c, n in cos_list) / total_params
    max_cos = max(c for c, _ in cos_list)
    min_cos = min(c for c, _ in cos_list)
    sparsity = zeros / total_params

    ledger = {
        "protocol": "TURBOVEC_TERNARY_QUANTIZATION",
        "model": "openai-community/gpt2",
        "source": "hf_blob_cache_direct_resolution",
        "weights_file": os.path.basename(blob),
        "weights_format": fmt,
        "weights_dtype": "F32",
        "monad_anchor": hex(MONAD),
        "seal": hex(seal("turbovec-gpt2")),
        "weights_quantized_2d": tensors,
        "params_quantized": total_params,
        "params_1d_kept_fp": skipped_1d_params,
        "params_causal_mask_buffers_excluded": mask_params_excluded,
        "fp32_bytes": fp32_bytes,
        "fp16_bytes": fp16_bytes,
        "trit_bytes": trit_bytes,
        "compression_ratio": round(ratio_fp32, 2),
        "compression_ratio_fp16_basis": round(ratio_fp16, 2),
        "ternary_sparsity_zeros": round(sparsity, 4),
        "reconstruction_cosine_min": round(min_cos, 4),
        "reconstruction_cosine_max": round(max_cos, 4),
        "reconstruction_cosine_mean_param_weighted": round(mean_cos, 4),
        "per_group": {g: {kk: (round(v, 4) if isinstance(v, float) else v)
                          for kk, v in s.items()} | {
                              "compression_ratio": round(
                                  s["fp32_bytes"] / s["trit_bytes"], 2)}
                       for g, s in group_stats.items()},
        "elapsed_s": round(elapsed, 2),
        "phi_v": None,
        "phi_v_note": "not measured in this run: ledger covers weight quantization only; live Phi_V is served by host_council_server /system/voltage and derived in voltage.rs from measured rails, clock and memory",
        "claim": ("measured on real openai-community/gpt2 F32 safetensors "
                  "(blob-resolved, wte+wpe embeddings included, causal-mask "
                  "buffers excluded)"),
    }
    out = os.path.join(ROOT, "turbovec_quantization_ledger_gpt2.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(ledger, f, indent=1)
    print(json.dumps(ledger, indent=1))
    print(f"\nquantized 2-D weight tensors: {tensors} | "
          f"params: {total_params/1e6:.2f}M | 1-D norm params kept FP: "
          f"{skipped_1d_params/1e6:.3f}M | causal-mask buffer params excluded: "
          f"{mask_params_excluded/1e6:.2f}M")
    print(f"ternary zeros (sparsity): {sparsity*100:.1f}%")
    print(f"compression: {ratio_fp32:.2f}x REAL (F32 {fp32_bytes/1e6:.0f} MB -> "
          f"packed trits {trit_bytes/1e6:.1f} MB, 4 trits/byte) | "
          f"{ratio_fp16:.2f}x on FP16 basis")
    print(f"cosine fidelity: min {min_cos:.4f} | mean {mean_cos:.4f} | "
          f"max {max_cos:.4f}")
    print(f"ledger: {out} | elapsed {elapsed:.1f}s")


if __name__ == "__main__":
    main()
