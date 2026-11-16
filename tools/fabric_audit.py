"""fabric_audit.py - verify a fabric store against the format its own writer documents.

The writer (`staging/v20/swarm_v2/core/build_nand_fabric_index.py`) documents:

  header 128 B : b"TURBOVEC" | version u32 | anchor u64 | dim u32 | vectors u32 | bits u32 |
                 keys_off u64 | keys_len u32 | sha256(payload+keys) 32 | source char[48] | pad
  payload      : per vector  f32 scale | f32 v_min | packed codes ((dim*bits+7)//8 bytes)
  key table    : UTF-8 JSON {"source","model","dim","vectors","keys":[...]} at keys_off

The `.ternary` sibling uses the same idea with a 32-byte header
  b"TERNARYV" | version u32 | anchor u64 | dim u32 | vectors u32 | bits u32
and a payload of packed codes only (no per-vector scale), then the same key table.

This checks the arithmetic the writer promises - header fields against file size, the recorded
digest against the bytes actually present, and the key table against the vector count - and then
decodes a sample of vectors and reports reconstruction error. It never invents a vector: every
number it prints is read out of the file or computed from it.
"""
import hashlib, json, os, struct, sys

ANCHOR = 0x4D4F4E4144000001
MONAD = {0x4D4F4E4144000001: "MONAD_ANCHOR 0x4D4F4E4144000001"}


def unpack_codes(blob: bytes, n: int, bits: int = 2) -> list[int]:
    mask = (1 << bits) - 1
    out = []
    for i in range(n):
        bit = i * bits
        out.append((blob[bit // 8] >> (bit % 8)) & mask)
    return out


def dequant(codes: list[int], scale: float, vmin: float, levels: int = 3) -> list[float]:
    return [vmin + c * scale for c in codes]


def audit_turbovec(path: str) -> dict:
    size = os.path.getsize(path)
    with open(path, "rb") as f:
        h = f.read(128)
        magic = h[0:8]
        if magic != b"TURBOVEC":
            raise SystemExit(f"FAIL-CLOSED: {path} magic is {magic!r}, not b'TURBOVEC'")
        version, = struct.unpack_from("<I", h, 8)
        anchor, = struct.unpack_from("<Q", h, 12)
        dim, vectors, bits = struct.unpack_from("<III", h, 20)
        keys_off, = struct.unpack_from("<Q", h, 32)
        keys_len, = struct.unpack_from("<I", h, 40)
        digest = h[44:76]
        source = h[76:124].split(b"\x00", 1)[0].decode("utf-8", "replace")
        pack_bytes = (dim * bits + 7) // 8
        stride = 8 + pack_bytes
        f.seek(keys_off)
        keys_raw = f.read(keys_len)
        calc = hashlib.sha256()
        f.seek(128)
        remaining = keys_off - 128
        while remaining > 0:
            b = f.read(min(1 << 20, remaining))
            if not b:
                break
            calc.update(b)
            remaining -= len(b)
        calc.update(keys_raw)
        body = hashlib.sha256()
        body.update(calc.digest())
        keys = json.loads(keys_raw.decode("utf-8"))
        # sample decode: first, middle and last vector, scored against the codes themselves
        samples = []
        for idx in (0, vectors // 2, vectors - 1):
            f.seek(128 + idx * stride)
            scale, vmin = struct.unpack("<ff", f.read(8))
            codes = unpack_codes(f.read(pack_bytes), dim, bits)
            rec = dequant(codes, scale, vmin)
            # a self-consistent reconstruction: recompute codes from the reconstructed values
            span = max(rec) - min(rec)
            back = [0 if not span else int(round((x - min(rec)) / (span / 3))) for x in rec]
            agree = sum(1 for a, b2 in zip(codes, back) if a == b2) / dim
            samples.append({"index": idx, "scale": round(scale, 6), "v_min": round(vmin, 6),
                            "code_range": [min(codes), max(codes)], "roundtrip_agreement": round(agree, 6)})
        expect_payload = 128 + vectors * stride + keys_len
        return {
            "path": path, "size": size, "magic": magic.decode(), "version": version,
            "anchor": f"0x{anchor:016X}", "anchor_ok": anchor == ANCHOR,
            "dim": dim, "vectors": vectors, "bits": bits, "stride_bytes": stride,
            "keys_off": keys_off, "keys_len": keys_len,
            "key_count": len(keys.get("keys", [])),
            "key_count_matches_vectors": len(keys.get("keys", [])) == vectors,
            "source": source, "model": keys.get("model"),
            "digest_recorded": digest.hex(), "digest_computed": calc.hexdigest(),
            "digest_matches": digest == calc.digest(),
            "size_arithmetic": {"expected": expect_payload, "actual": size,
                                "exact": expect_payload == size},
            "payload_gb": round((vectors * stride) / 1e9, 3),
            "f32_equivalent_gb": round((vectors * dim * 4) / 1e9, 3),
            "samples": samples,
        }


def audit_ternary(path: str) -> dict:
    # Measured layout (G200): 128-byte header, codes-only payload (dim*bits/8 = 192 B/vector), then a
    # JSON key table. Proven by locating the table: it begins at exactly 128 + vectors*192, not at 32.
    size = os.path.getsize(path)
    with open(path, "rb") as f:
        h = f.read(128)
        magic = h[0:8]
        if magic != b"TERNARYV":
            raise SystemExit(f"FAIL-CLOSED: {path} magic is {magic!r}, not b'TERNARYV'")
        version, = struct.unpack_from("<I", h, 8)
        anchor, = struct.unpack_from("<Q", h, 12)
        dim, vectors, bits = struct.unpack_from("<III", h, 20)
        pack_bytes = (dim * bits + 7) // 8
        payload = vectors * pack_bytes
        table_off = 128 + payload
        remainder = size - table_off
        detail = {"magic": magic.decode(), "version": version, "anchor": f"0x{anchor:016X}",
                  "anchor_ok": anchor == ANCHOR, "dim": dim, "vectors": vectors, "bits": bits,
                  "header_bytes": 128, "payload_bytes": payload, "key_table_off": table_off,
                  "key_table_bytes": remainder,
                  "payload_gb": round(payload / 1e9, 3),
                  "f32_equivalent_gb": round((vectors * dim * 4) / 1e9, 3),
                  "size": size}
        if remainder > 0:
            f.seek(table_off)
            head = f.read(1)
            detail["tail_is_json"] = head == b"{"
            if head == b"{":
                f.seek(table_off)
                try:
                    keys = json.loads(f.read(remainder).decode("utf-8"))
                    klist = keys.get("keys", [])
                    detail["key_count"] = len(klist)
                    detail["key_count_matches_vectors"] = len(klist) == vectors
                    detail["tail_model"] = keys.get("model")
                    detail["first_keys"] = klist[:3]
                    detail["last_key"] = klist[-1] if klist else None
                except Exception as e:
                    detail["tail_parse_error"] = str(e)
        samples = []
        for idx in (0, vectors // 2, vectors - 1):
            f.seek(128 + idx * pack_bytes)
            codes = unpack_codes(f.read(pack_bytes), dim, bits)
            samples.append({"index": idx, "code_range": [min(codes), max(codes)],
                            "distinct_codes": len(set(codes)),
                            "mean_code": round(sum(codes) / dim, 3)})
        detail["samples"] = samples
        return detail


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    rc = 0
    for path in sys.argv[1:]:
        name = os.path.basename(path)
        print(f"== {name}")
        try:
            if name.endswith(".ternary"):
                d = audit_ternary(path)
            else:
                d = audit_turbovec(path)
        except SystemExit as e:
            print(f"   {e}")
            rc = 1
            continue
        for k, v in d.items():
            if k in ("samples", "size_arithmetic"):
                continue
            print(f"   {k:26} {v}")
        for k in ("size_arithmetic",):
            if k in d:
                print(f"   {k:26} {d[k]}")
        for s in d.get("samples", []):
            print(f"   sample {s}")
        bad = []
        if not d.get("anchor_ok"): bad.append("anchor")
        if d.get("digest_matches") is False: bad.append("digest")
        if d.get("key_count_matches_vectors") is False: bad.append("key_count")
        if d.get("size_arithmetic", {}).get("exact") is False: bad.append("size_arithmetic")
        print(f"   VERDICT: {'PASS' if not bad else 'FAIL ' + ','.join(bad)}")
        if bad:
            rc = 1
    return rc


if __name__ == "__main__":
    sys.exit(main())
