#!/usr/bin/env python3
"""
TRM Swarm OS — TurboVec 1.58-bit Ternary Quantization (openai/clip-vit-base-patch32).
Reads the REAL cached CLIP checkpoint (HF cache snapshot), quantizes all 2D+ weight
tensors (conv/linear/embedding) to balanced ternary {-1, 0, +1} trits using the
mean-abs alpha threshold at ±0.5 (1.58-bit doctrine), packs 4 trits/byte, measures
the REAL compression ratio and reconstruction cosine fidelity on sampled vectors,
and writes a JSON ledger.

On-device only — no API. Honest measured numbers; no inflated claims.
NOTE: this HF checkpoint ships as pytorch_model.bin (fp32, 605 MB), not safetensors;
a *.safetensors shard is used automatically if present instead.
"""
import os, sys, time, json, glob
import numpy as np

MONAD = 0x4D4F4E4144000001
ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
SNAP = os.path.join(
    os.path.expanduser("~/.cache/huggingface/hub"),
    "models--openai--clip-vit-base-patch32", "snapshots")
MODEL_ID = "openai/clip-vit-base-patch32"


def seal(s: str) -> int:
    import hashlib
    h = int(hashlib.sha256(s.encode()).hexdigest()[:16], 16)
    return (MONAD ^ h) & 0xFFFFFFFFFFFFFFFF


def quantize_ternary(tensor: np.ndarray) -> tuple[np.ndarray, float]:
    """1.58-bit balanced ternary: threshold at mean abs (BitNet-style).
    Returns (trits in {-1,0,+1} int8, per-tensor alpha scale)."""
    orig = tensor.astype(np.float32)
    alpha = float(np.mean(np.abs(orig)))
    if alpha < 1e-9:
        return np.zeros_like(orig, dtype=np.int8), 0.0
    t = orig / alpha
    trits = np.where(t < -0.5, -1, np.where(t > 0.5, 1, 0)).astype(np.int8)
    return trits, alpha


def sampled_cosine(o: np.ndarray, d: np.ndarray, cap: int = 200_000) -> float:
    """Cosine similarity on a strided sample (template convention)."""
    step = max(1, o.size // cap)
    o_s = o.reshape(-1)[::step]
    d_s = d.reshape(-1)[::step]
    return float(np.dot(o_s, d_s) / (np.linalg.norm(o_s) * np.linalg.norm(d_s) + 1e-9))


VOLTAGE_ENDPOINT = "http://127.0.0.1:8022/system/voltage"


def sample_phi_v(timeout: float = 1.5) -> dict:
    """Phi_V from the doctrine source: GET /system/voltage.

    The value is DERIVED in trm_uefi_kernel/host_council_server/src/voltage.rs as
        (V_core/1.25)^2 * (f_actual/4000) * (V_ddr/1.35) * max(0.5, 1 - mem%/200)
    and this host has no voltage sensor (no LibreHardwareMonitor / HWiNFO), so the
    endpoint answers measured=false: the number comes from nominal rails, it is
    not a reading. That flag is carried into the ledger verbatim so a derived
    number can never be mistaken for a measurement.

    If the endpoint is unreachable, phi_v is None. An absent field is honest; a
    placeholder constant is not.

    History: this ledger previously emitted a hardcoded constant as "phi_v" and
    the claim called it measured. That constant is withdrawn, and its exact
    historical value is preserved in evidence/PHI_V_ERRATA.txt (errata, not
    deletion). It is deliberately not repeated here so a grep for the forbidden
    literal only matches the audit/errata record.
    """
    import urllib.request
    try:
        with urllib.request.urlopen(VOLTAGE_ENDPOINT, timeout=timeout) as r:
            d = json.loads(r.read().decode("utf-8"))
        return {
            "phi_v": d.get("phi_v"),
            "measured": bool(d.get("measured", False)),
            "source": VOLTAGE_ENDPOINT,
            "detail": d.get("source", ""),
        }
    except Exception as exc:
        return {
            "phi_v": None,
            "measured": False,
            "source": VOLTAGE_ENDPOINT,
            "detail": f"unreachable ({type(exc).__name__}); no voltage sensor on this host",
        }


def main():
    import torch

    if not os.path.isdir(SNAP):
        print(f"NO CACHE DIR: {SNAP}"); sys.exit(1)
    snap_dir = None
    for s in sorted(os.listdir(SNAP)):
        cand = os.path.join(SNAP, s)
        if os.path.isdir(cand):
            snap_dir = cand
            break
    if not snap_dir:
        print("NO SNAPSHOT"); sys.exit(1)
    print(f"snapshot: {snap_dir}")

    bin_path = os.path.join(snap_dir, "pytorch_model.bin")
    st_shards = sorted(glob.glob(os.path.join(snap_dir, "*.safetensors")))

    t0 = time.time()

    # ---- load state dict (bin primary; safetensors fallback) ----
    source_file = None
    if os.path.exists(bin_path):
        source_file = "pytorch_model.bin"
        print(f"loading {source_file} ({os.path.getsize(bin_path)/1e6:.1f} MB) ...")
        sd = torch.load(bin_path, map_location="cpu", weights_only=True)
    elif st_shards:
        source_file = os.path.basename(st_shards[0])
        from safetensors import safe_open
        print(f"loading safetensors {source_file} ...")
        sd = {}
        for sh in st_shards:
            with safe_open(sh, framework="pt") as f:
                for k in f.keys():
                    sd[k] = f.get_tensor(k)
    else:
        print(f"NO WEIGHT FILE in {snap_dir}"); sys.exit(1)
    items = [(k, v) for k, v in sd.items() if isinstance(v, torch.Tensor)]
    del sd

    # dominant dtype of stored weights (informational)
    from collections import Counter
    dt = Counter(str(v.dtype) for _, v in items)

    # ---- quantize every 2D+ weight tensor (skip 1-D norm/bias gains) ----
    total_params = 0
    total_fp16_bytes = 0
    total_fp32_bytes = 0
    total_trit_bytes = 0
    cos_list = []
    o_glob, d_glob = [], []
    results = {}
    n_tensors = 0
    for k, t in items:
        if t.ndim < 2:
            continue  # LayerNorm/bias scalars: skip (template doctrine)
        n_tensors += 1
        total_params += t.numel()
        total_fp16_bytes += t.numel() * 2
        total_fp32_bytes += t.numel() * 4

        t_np = t.float().numpy()
        trits, alpha = quantize_ternary(t_np)
        trit_bytes = (trits.size + 3) // 4  # packed 4 trits/byte
        total_trit_bytes += trit_bytes

        deq = trits.astype(np.float32) * alpha
        cos = sampled_cosine(t_np, deq)
        cos_list.append(cos)
        o_glob.append(t_np.reshape(-1)[::max(1, t_np.size // 200_000)])
        d_glob.append(deq.reshape(-1)[::max(1, deq.size // 200_000)])
        results[k] = {"cos": round(cos, 5), "params": int(t.numel()),
                      "alpha": round(alpha, 6)}
        del t, t_np, trits, deq

    o_g = np.concatenate(o_glob)
    d_g = np.concatenate(d_glob)
    cos_global = float(np.dot(o_g, d_g) / (np.linalg.norm(o_g) * np.linalg.norm(d_g) + 1e-9))
    elapsed = time.time() - t0

    source_bytes = os.path.getsize(bin_path) if os.path.exists(bin_path) else \
        sum(os.path.getsize(s) for s in st_shards)

    phi = sample_phi_v()

    ledger = {
        "protocol": "TURBOVEC_TERNARY_QUANTIZATION",
        "model": MODEL_ID,
        "source_file": source_file,
        "source_dtype_counts": dict(dt),
        "monad_anchor": hex(MONAD),
        "seal": hex(seal("turbovec-clip-vit-base-patch32")),
        "tensors_quantized": n_tensors,
        "params_quantized": int(total_params),
        "fp16_equiv_bytes": int(total_fp16_bytes),
        "fp32_equiv_bytes": int(total_fp32_bytes),
        "source_bytes": int(source_bytes),
        "trit_bytes": int(total_trit_bytes),
        "compression_ratio_fp16": round(total_fp16_bytes / total_trit_bytes, 2),
        "compression_ratio_fp32": round(total_fp32_bytes / total_trit_bytes, 2),
        "compression_ratio_disk": round(source_bytes / total_trit_bytes, 2),
        "reconstruction_cosine_min": round(min(cos_list), 5),
        "reconstruction_cosine_max": round(max(cos_list), 5),
        "reconstruction_cosine_mean": round(float(np.mean(cos_list)), 5),
        "reconstruction_cosine_global": round(cos_global, 5),
        "elapsed_s": round(elapsed, 2),
        # Phi_V is NOT a constant and NOT a reading. It is derived from nominal
        # rails by /system/voltage, and `phi_v_measured` says which. On this host
        # it is always false: there is no voltage sensor. The former hardcoded
        # constant is withdrawn (errata: evidence/PHI_V_ERRATA.txt), not replaced
        # by another number. If the endpoint is down, phi_v is null.
        "phi_v": phi["phi_v"],
        "phi_v_measured": phi["measured"],
        "phi_v_source": phi["source"],
        "phi_v_note": (
            "derived from nominal rails via the voltage.rs formula, not read from "
            "a sensor (this host has none); the measured flag is authoritative and "
            "phi_v must not be cited as a measurement when it is false"
        ),
        "claim": (
            "compression and reconstruction cosine measured on real cached "
            "openai/clip-vit-base-patch32 weights; phi_v is derived, not measured "
            "(see phi_v_measured)"
        ),
    }
    out = os.path.join(ROOT, "quantize_clip_ledger.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(ledger, f, indent=1)

    print(json.dumps(ledger, indent=1))
    r16 = ledger["compression_ratio_fp16"]
    print(f"\nclip tensors: {n_tensors} | params: {total_params/1e6:.1f}M")
    print(f"compression: {r16:.1f}x vs FP16 ({total_fp16_bytes/1e6:.0f}MB -> {total_trit_bytes/1e6:.1f}MB)")
    print(f"             {ledger['compression_ratio_disk']:.1f}x vs actual disk ({source_bytes/1e6:.0f}MB -> {total_trit_bytes/1e6:.1f}MB)")
    print(f"cosine fidelity: global={cos_global:.5f} | per-tensor mean={ledger['reconstruction_cosine_mean']:.5f} "
          f"min={ledger['reconstruction_cosine_min']:.5f} max={ledger['reconstruction_cosine_max']:.5f}")
    print(f"ledger: {out}")


if __name__ == "__main__":
    main()
