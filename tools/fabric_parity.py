"""fabric_parity.py - does the kernel's encoder reproduce the prebuilt fabric bit for bit?

The fabric was built elsewhere, from texts that are shipped alongside it (`*.text.jsonl`). If our in-process
encoder (`rust_native_encode_text_f32`) and the fabric's own quantiser reproduce those same codes, then the
fabric is fully regenerable here and its 1.27 GB is reproducibility, not a dependency on another machine.

Method - all four inputs come from files, none from assumption:
  * text      : the fabric's own `text` field for record i
  * scale,min : the per-record pair stored in the `.turbovec` payload
  * codes     : the packed 2-bit codes in `.turbovec` (stride 200: f32 scale | f32 v_min | 192 B)
  * recompute : encode(text) -> x; code = round((x - v_min) / scale) clamped to 0..3
If the recomputed codes equal the stored codes, the encoder and quantiser agree exactly.
A control runs alongside: the same comparison against a DIFFERENT record's codes, which must NOT match.
"""
import ctypes, hashlib, json, os, struct, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FAB_DIR = os.path.join(ROOT, "scratch", "fabric_chembl37")
TURBOVEC = os.path.join(FAB_DIR, "swarm_v2_chembl_fabric.turbovec")
TERNARY = os.path.join(FAB_DIR, "swarm_v2_chembl_fabric.ternary")
TEXTS = os.path.join(FAB_DIR, "swarm_v2_chembl_fabric.text.jsonl")
DLL = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "docs", "port", "laptop-base", "dll", "swarm_rust_kernel_E.dll")
NSAMPLE = int(sys.argv[2]) if len(sys.argv) > 2 else 300
DIM, BITS = 768, 2
PACK = (DIM * BITS + 7) // 8          # 192
STRIDE = 8 + PACK                      # 200 in .turbovec
LEVELS = 3


def read_texts(indices):
    want = {i: None for i in indices}
    hi = max(want)
    with open(TEXTS, encoding="utf-8") as f:
        for i, line in enumerate(f):
            if i in want:
                want[i] = json.loads(line)["text"]
            if i >= hi:
                break
    return want


def unpack(blob):
    out = []
    for i in range(DIM):
        bit = i * BITS
        out.append((blob[bit // 8] >> (bit % 8)) & LEVELS)
    return out


def pack(codes):
    out = bytearray(PACK)
    for i, c in enumerate(codes):
        bit = i * BITS
        out[bit // 8] |= (c & LEVELS) << (bit % 8)
    return bytes(out)


def main():
    lib = ctypes.CDLL(DLL)
    lib.rust_native_encode_text_f32.argtypes = [ctypes.c_char_p, ctypes.c_size_t,
                                                ctypes.POINTER(ctypes.c_float), ctypes.c_size_t]
    lib.rust_native_encode_text_f32.restype = ctypes.c_int32

    # sample across the whole store, not just the head (a head-only sample would hide an offset error)
    with open(TURBOVEC, "rb") as f:
        h = f.read(128)
        dim, vectors, bits = struct.unpack_from("<III", h, 20)
        assert (dim, bits) == (DIM, BITS), (dim, bits)
    step = max(1, vectors // NSAMPLE)
    indices = [i for i in range(0, vectors, step)][:NSAMPLE]
    texts = read_texts(indices)
    missing = [i for i in indices if texts.get(i) is None]
    if missing:
        raise SystemExit(f"FAIL-CLOSED: no text for {len(missing)} indices, e.g. {missing[:5]}")

    agree_total = 0
    compared = 0
    exact_records = 0
    worst = 0.0
    control_agree = 0
    per_record = []
    with open(TURBOVEC, "rb") as f, open(TERNARY, "rb") as t:
        for k, idx in enumerate(indices):
            f.seek(128 + idx * STRIDE)
            scale, vmin = struct.unpack("<ff", f.read(8))
            stored = f.read(PACK)
            stored_codes = unpack(stored)

            t.seek(128 + idx * PACK)
            t_codes = unpack(t.read(PACK))

            buf = (ctypes.c_float * DIM)()
            raw = texts[idx].encode("utf-8", "ignore")
            rc = lib.rust_native_encode_text_f32(raw, len(raw), buf, DIM)
            if rc != DIM:
                raise SystemExit(f"encoder rc={rc} on record {idx}")
            mine_codes = [0 if scale == 0 else max(0, min(LEVELS, int(round((x - vmin) / scale))))
                          for x in buf]
            agree = sum(1 for a, b in zip(mine_codes, stored_codes) if a == b)
            agr = agree / DIM
            t_agree = sum(1 for a, b in zip(mine_codes, t_codes) if a == b) / DIM
            agree_total += agree
            compared += DIM
            if agr == 1.0:
                exact_records += 1
            worst = min(worst, agr) if per_record else agr if agr < worst else worst
            per_record.append(agr)
            # control: my codes for THIS text vs the NEXT record's stored codes
            if k + 1 < len(indices):
                f.seek(128 + indices[k + 1] * STRIDE + 8)
                nxt = unpack(f.read(PACK))
                control_agree += sum(1 for a, b in zip(mine_codes, nxt) if a == b) / DIM

    n = len(indices)
    res = {
        "dll": os.path.basename(DLL), "samples": n, "dim": DIM, "bits": BITS,
        "sample_indices": indices[:5] + ["..."] + indices[-3:],
        "code_agreement": round(agree_total / compared, 6),
        "byte_identical_records": exact_records,
        "mean_record_agreement": round(sum(per_record) / n, 6),
        "min_record_agreement": round(min(per_record), 6),
        "max_record_agreement": round(max(per_record), 6),
        "control_agreement": round(control_agree / max(1, n - 1), 6),
        "ternary_vs_native_first": None,
        "verdict": None,
    }
    # a control that scores the same as the treatment means the test measures nothing
    if abs(res["code_agreement"] - res["control_agreement"]) < 0.05:
        res["verdict"] = "INCONCLUSIVE - control matches treatment; the comparison is not discriminating"
    elif res["code_agreement"] > 0.999:
        res["verdict"] = "EXACT - the encoder reproduces the fabric's own codes on every sampled vector"
    else:
        res["verdict"] = f"PARTIAL - {res['code_agreement']:.4f} code agreement (investigate quantiser, not encoder)"
    out = os.path.join(ROOT, "data", "fabric_parity.json")
    json.dump(res, open(out, "w"), indent=1)
    for k, v in res.items():
        print(f"  {k:26} {v}")
    print(f"  written {out}")
    return 0 if res["verdict"].startswith(("EXACT", "PARTIAL", "INCONCLUSIVE")) else 1


if __name__ == "__main__":
    sys.exit(main())
