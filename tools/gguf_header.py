#!/usr/bin/env python3
"""Minimal GGUF header reader (stdlib only) - reports version, counts, tensor types.

Why hand-rolled: the `gguf` pip lib was unavailable for the interpreter with a working numpy on this host, and the
Bonsai 2 files may use ternary tensor types a stale lib cannot parse at all. The header layout is stable and small:
  magic 'GGUF' | u32 version | u64 n_tensors | u64 n_kv | KV pairs | tensor infos | padding | data
Only the tensor table is read here - no dequantisation, so nothing can be invented about block layouts.
"""
from __future__ import annotations
import struct, sys
from collections import Counter
from pathlib import Path

# ggml type ids (llama.cpp ggml.h). Anything not listed prints as its raw id rather than a guess.
TYPES = {0: "F32", 1: "F16", 2: "Q4_0", 3: "Q4_1", 6: "Q5_0", 7: "Q5_1", 8: "Q8_0", 9: "Q8_1",
         10: "Q2_K", 11: "Q3_K", 12: "Q4_K", 13: "Q5_K", 14: "Q6_K", 15: "Q8_K", 16: "IQ2_XXS",
         17: "IQ2_XS", 18: "IQ3_XXS", 19: "IQ1_S", 20: "IQ4_NL", 21: "IQ3_S", 22: "IQ2_S", 23: "IQ4_XS",
         24: "I8", 25: "I16", 26: "I32", 27: "I64", 28: "F64", 29: "IQ1_M", 30: "BF16",
         34: "TQ1_0", 35: "TQ2_0", 36: "IQ4_NL_dup", 37: "MXFP4"}
SIZES = {0: 1, 1: 1, 2: 1, 3: 1, 6: 1, 7: 1, 8: 1, 9: 1, 10: 1, 11: 1, 12: 1, 13: 1, 14: 1, 15: 1,
         16: 1, 17: 1, 18: 1, 19: 1, 20: 1, 21: 1, 22: 1, 23: 1, 24: 1, 25: 2, 26: 4, 27: 8, 28: 8,
         29: 1, 30: 2, 34: 1, 35: 1, 37: 1}
# block size (elements per block) and bytes per block, from ggml.h type traits
BLOCK = {0: (1, 4), 1: (1, 2), 2: (32, 18), 3: (32, 20), 6: (32, 22), 7: (32, 24), 8: (32, 34), 9: (32, 36),
         10: (256, 84), 11: (256, 110), 12: (256, 144), 13: (256, 176), 14: (256, 210), 15: (256, 292),
         24: (1, 1), 25: (1, 2), 26: (1, 4), 27: (1, 8), 28: (1, 8), 30: (1, 2), 34: (256, 54), 35: (256, 66)}

SCALAR = {0: "<B", 1: "<b", 2: "<H", 3: "<h", 4: "<I", 5: "<i", 6: "<f", 7: "<?", 10: "<Q", 11: "<q", 12: "<d"}


def read_str(f) -> str:
    n = struct.unpack("<Q", f.read(8))[0]
    return f.read(n).decode("utf-8", "replace")


def skip_value(f, vtype: int):
    if vtype == 8:                      # string
        read_str(f)
    elif vtype == 9:                    # array
        itype, count = struct.unpack("<IQ", f.read(12))
        if itype == 8:
            for _ in range(count):
                read_str(f)
        else:
            f.seek(struct.calcsize(SCALAR[itype]) * count, 1)
    else:
        f.seek(struct.calcsize(SCALAR[vtype]), 1)


def dump(path: Path) -> dict:
    with path.open("rb") as f:
        magic = f.read(4)
        if magic != b"GGUF":
            raise SystemExit(f"not a GGUF file: {magic!r}")
        version, n_tensors, n_kv = struct.unpack("<IQQ", f.read(20))
        meta = {}
        for _ in range(n_kv):
            key = read_str(f)
            vtype = struct.unpack("<I", f.read(4))[0]
            if vtype in (0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 12) and key in (
                    "general.architecture", "general.name", "general.file_type",
                    "general.quantization_version", "llama.context_length", "general.alignment"):
                val = struct.unpack(SCALAR[vtype], f.read(struct.calcsize(SCALAR[vtype])))[0]
                meta[key] = val
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
            bs, bb = BLOCK.get(ttype, (0, 0))
            nbytes = (n_elem // bs) * bb if bs else 0
            tensors.append({"name": name, "dims": dims, "type": ttype,
                            "type_name": TYPES.get(ttype, f"UNKNOWN({ttype})"),
                            "offset": off, "elements": n_elem, "bytes": nbytes})
    return {"file": str(path), "size_bytes": path.stat().st_size, "version": version,
            "n_tensors": n_tensors, "meta": meta, "tensors": tensors}


if __name__ == "__main__":
    for arg in sys.argv[1:]:
        d = dump(Path(arg))
        print(f"\n=== {Path(arg).name} ===")
        print(f"  gguf v{d['version']}  tensors={d['n_tensors']}  file={d['size_bytes']/1e9:.2f} GB")
        print(f"  meta: {d['meta']}")
        types = Counter(t["type_name"] for t in d["tensors"])
        print(f"  tensor types: {dict(types)}")
        by_type = Counter()
        for t in d["tensors"]:
            by_type[t["type_name"]] += t["bytes"]
        tot = sum(by_type.values()) or 1
        for k, v in by_type.most_common():
            print(f"    {k:<12} {v/1e9:6.2f} GB  {100*v/tot:5.1f} %")
        for t in d["tensors"][:4]:
            print(f"  e.g. {t['name']:<46} {t['type_name']:<7} dims={t['dims']} {t['bytes']/1e6:.1f} MB")
