"""bench_native_encoder.py - how fast is the kernel's own text encoder, the one that built the fabric?

`rust_native_encode_text_f32(text, len, out, dim) -> i32` is exported by swarm_rust_kernel.dll and is the
encoder named in the fabric's own header (`model: native-turbovec-hdc`). The HTTP embedding path used by
`k3_ingest_real.py` measured ~60 ms/record (16-17 vec/s). This measures the native one on the same box with
the same corpus lines, so the comparison is like for like: same texts, same 768 dims, same machine.

It reports records/s and, crucially, checks that the output is a real vector (not zeros, not a constant) -
an encoder that returns the same vector for every input would look infinitely fast and mean nothing.
"""
import ctypes, json, os, sys, time

DLL = sys.argv[1] if len(sys.argv) > 1 else None
N = int(sys.argv[2]) if len(sys.argv) > 2 else 5000
DIM = 768
SMI = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                   "scratch", "chembl37", "chembl_smiles_full.txt")


def load(dll_path):
    lib = ctypes.CDLL(dll_path)
    lib.rust_native_encode_text_f32.argtypes = [ctypes.c_char_p, ctypes.c_size_t,
                                                ctypes.POINTER(ctypes.c_float), ctypes.c_size_t]
    lib.rust_native_encode_text_f32.restype = ctypes.c_int32
    return lib


def read_texts(n):
    out = []
    with open(SMI, encoding="utf-8", errors="ignore") as f:
        for line in f:
            line = line.rstrip("\n\r")
            if not line:
                continue
            smi, _, cid = line.partition("\t")
            out.append(f"ChEMBL molecule {cid} SMILES {smi}")
            if len(out) >= n:
                break
    return out


def main():
    if not DLL or not os.path.exists(DLL):
        raise SystemExit(f"FAIL-CLOSED: dll path not given or missing: {DLL}")
    texts = read_texts(N)
    if not texts:
        raise SystemExit("FAIL-CLOSED: no corpus lines read")
    lib = load(DLL)
    buf = (ctypes.c_float * DIM)()
    raw = texts[0].encode("utf-8")
    rc = lib.rust_native_encode_text_f32(raw, len(raw), buf, DIM)
    first = list(buf)
    if rc != DIM:
        raise SystemExit(f"FAIL-CLOSED: encoder returned rc={rc} (success is rc == dim == {DIM})")
    norm = sum(x * x for x in first) ** 0.5
    if norm == 0.0:
        raise SystemExit("FAIL-CLOSED: encoder produced an all-zero vector")
    print(f"dll           {os.path.basename(DLL)}")
    print(f"first vector  norm={norm:.6f}  min={min(first):.6f} max={max(first):.6f} distinct={len(set(first))}")
    # speed
    t0 = time.perf_counter()
    vecs = []
    for t in texts:
        b = t.encode("utf-8")
        v = (ctypes.c_float * DIM)()
        rc = lib.rust_native_encode_text_f32(b, len(b), v, DIM)
        if rc != DIM:
            raise SystemExit(f"encode failed rc={rc}")
        vecs.append(list(v))
    el = time.perf_counter() - t0
    rate = len(texts) / el
    print(f"records       {len(texts)}")
    print(f"elapsed       {el:.3f} s")
    print(f"rate          {rate:,.0f} records/s   ({el/len(texts)*1e6:,.0f} us/record)")
    # the control: are these actually different vectors?
    distinct = len({tuple(round(x, 6) for x in v[:32]) for v in vecs})
    print(f"distinct(32d) {distinct} of {len(vecs)}  -> {'real' if distinct > 1 else 'CONSTANT - MEASURES NOTHING'}")
    a, b2 = vecs[0], vecs[-1]
    cos = sum(x * y for x, y in zip(a, b2))
    print(f"cos(first,last) {cos:.4f}")
    http = 60.0
    print(f"vs HTTP path  {rate / (1 / (http / 1000)):.0f}x faster than 60 ms/record")
    # full-corpus projection, stated as arithmetic not a promise
    full = 2897805
    print(f"2,897,805 rows would take {full / rate / 60:.1f} min at this rate (HTTP path: {full * http / 1000 / 3600:.1f} h)")
    json.dump({"dll": os.path.basename(DLL), "records": len(texts), "seconds": round(el, 3),
               "rate_per_s": round(rate, 1), "us_per_record": round(el / len(texts) * 1e6, 1),
               "distinct_prefix": distinct, "norm": round(norm, 6)},
              open(os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                                "data", "native_encoder_bench.json"), "w"), indent=1)


if __name__ == "__main__":
    main()
