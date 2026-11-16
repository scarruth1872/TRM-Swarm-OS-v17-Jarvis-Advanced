"""audit_ingest_ledger - refuse to accept an ingestion claim you cannot re-check.

An ingestion ledger is only evidence if every entry carries a handle someone else can re-derive:
where the bytes came from, how many, their hash, and the hash/count of what was written. This tool
checks that, and re-verifies anything present locally. It is designed to FAIL - a ledger of prose
("SEALED (+1 TRUTH)", "validated on gold-standard controls") must not pass.

Usage:
    python tools/audit_ingest_ledger.py <ledger.json> [...more]
    python tools/audit_ingest_ledger.py build/chembl_real.k3.json

Exit codes: 0 every entry verifiable, 1 at least one entry unverifiable or mismatched, 2 unreadable.

Per-entry required fields (any of the accepted aliases):
    source or source_url or url or path   -> where the data came from
    records or record_count or n_records  -> how many records
    bytes or source_bytes or size         -> how many bytes of source
    sha256 or source_sha256 or seal_sha256-> a hash of the source (or of the written payload)
Optional, verified when present and the file exists locally:
    shard or output or artifact           -> written artifact; header/payload/seal are re-derived
"""
from __future__ import annotations

import hashlib
import json
import os
import struct
import sys

SOURCE_KEYS = ("source", "source_url", "url", "path", "file", "download")
COUNT_KEYS = ("records", "record_count", "n_records", "rows", "entries", "molecules", "variants")
BYTES_KEYS = ("bytes", "source_bytes", "size", "size_bytes", "file_bytes")
HASH_KEYS = ("sha256", "source_sha256", "seal_sha256", "sha256_seal", "checksum")
SHARD_KEYS = ("shard", "output", "artifact", "output_path", "shard_path")


def first(d: dict, keys) -> object | None:
    for k in keys:
        if k in d and d[k] not in (None, "", [], {}):
            return d[k]
    return None


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def verify_shard(path: str) -> list[str]:
    """A .k3 shard must declare its own size arithmetic - trits = records*dim, payload = ceil(trits/4)."""
    problems: list[str] = []
    side = path + ".json"
    if not os.path.exists(side):
        return [f"no sidecar {os.path.basename(side)} - header arithmetic cannot be checked"]
    meta = json.load(open(side, encoding="utf-8"))
    recs, dim = meta.get("records"), meta.get("dim")
    if not recs or not dim:
        return ["sidecar lacks records/dim"]
    expect_trits = recs * dim
    if meta.get("trits") is not None and meta["trits"] != expect_trits:
        problems.append(f"trits {meta['trits']} != records*dim {expect_trits}")
    expect_payload = (expect_trits + 3) // 4
    if meta.get("payload_bytes") is not None and meta["payload_bytes"] != expect_payload:
        problems.append(f"payload_bytes {meta['payload_bytes']} != ceil(trits/4) {expect_payload}")
    on_disk = os.path.getsize(path)
    declared = meta.get("file_bytes", on_disk)
    if on_disk != declared:
        problems.append(f"file on disk {on_disk} B != declared {declared} B")
    # never assume a header size: derive it, then require it to be a plausible small header
    payload_declared = meta.get("payload_bytes")
    header = on_disk - payload_declared if isinstance(payload_declared, int) else None
    if header is None:
        problems.append("sidecar lacks payload_bytes - payload cannot be bounded")
    elif not (0 < header <= 4096):
        problems.append(f"derived header {header} B is not a plausible shard header")
    elif meta.get("padding_bytes", 0) == 0 and payload_declared != expect_payload:
        problems.append(f"declared payload {payload_declared} B != ceil(trits/4) {expect_payload} B (padding declared 0)")
    if meta.get("padding_bytes", 0) not in (0, None):
        problems.append(f"padding_bytes={meta['padding_bytes']} - shard is padded, not faithful")
    return problems


def audit_entry(i: int, e: dict) -> tuple[bool, list[str]]:
    notes: list[str] = []
    src = first(e, SOURCE_KEYS)
    cnt = first(e, COUNT_KEYS)
    nbytes = first(e, BYTES_KEYS)
    hsh = first(e, HASH_KEYS)
    ok = True
    for label, val in (("source", src), ("records", cnt), ("bytes", nbytes), ("sha256", hsh)):
        if val is None:
            notes.append(f"missing {label}")
            ok = False
    if hsh is not None and (not isinstance(hsh, str) or len(hsh) != 64):
        notes.append(f"sha256 is not 64 hex chars: {hsh!r}")
        ok = False
    # re-derive anything local
    if isinstance(src, str) and os.path.exists(src):
        real = sha256_file(src)
        size = os.path.getsize(src)
        notes.append(f"source present: {size} B, sha256 {real[:16]}...")
        if isinstance(hsh, str) and len(hsh) == 64 and hsh != real:
            notes.append(f"sha256 MISMATCH: ledger {hsh[:16]}... vs file {real[:16]}...")
            ok = False
        if isinstance(nbytes, int) and nbytes not in (size,):
            notes.append(f"bytes MISMATCH: ledger {nbytes} vs file {size}")
            ok = False
    elif isinstance(src, str) and src.startswith(("http://", "https://")):
        notes.append("remote source not fetched by this audit (no network use)")
    shard = first(e, SHARD_KEYS)
    if shard is None:
        # `path` is also a source alias, so first() may have returned the corpus name instead.
        for key in ("path", "shard", "output", "file"):
            v = e.get(key)
            if isinstance(v, str) and v.lower().endswith(".k3"):
                shard = v  # a shard is its own source: header arithmetic must be re-derived either way
                break
    if isinstance(shard, str):
        p = shard if os.path.isabs(shard) else shard
        if os.path.exists(p):
            problems = verify_shard(p)
            notes.extend(problems)
            if problems:
                ok = False
            else:
                notes.append(f"shard verified: {os.path.basename(p)}")
        else:
            notes.append(f"shard declared but absent: {shard}")
    return ok, notes


def entries_of(doc) -> list[dict]:
    if isinstance(doc, list):
        return [x for x in doc if isinstance(x, dict)]
    if isinstance(doc, dict):
        for key in ("datasets", "entries", "records", "shards", "ingestions", "waves"):
            v = doc.get(key)
            if isinstance(v, list) and v and isinstance(v[0], dict):
                return v
            if isinstance(v, dict):
                flat = []
                for k, sub in v.items():
                    if isinstance(sub, list):
                        flat.extend([x for x in sub if isinstance(x, dict)])
                    elif isinstance(sub, dict):
                        flat.append(sub)
                if flat:
                    return flat
        # a single shard meta file
        if any(k in doc for k in COUNT_KEYS):
            return [doc]
    return []


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 2
    failed = 0
    for path in argv[1:]:
        try:
            doc = json.load(open(path, encoding="utf-8"))
        except Exception as exc:  # noqa: BLE001
            print(f"UNREADABLE {path}: {exc}")
            return 2
        ents = entries_of(doc)
        print(f"\n=== {path}  ({len(ents)} entries)")
        if not ents:
            print("  FAIL: no verifiable entries in this document - it is prose, not a ledger")
            failed += 1
            continue
        for i, e in enumerate(ents):
            # a k3_ingest_real sidecar does not repeat its own path - the shard is its sibling
            if "shard" not in e and path.lower().endswith(".k3.json"):
                sibling = path[:-5]
                if os.path.exists(sibling):
                    e["shard"] = sibling
            ok, notes = audit_entry(i, e)
            label = str(first(e, SOURCE_KEYS))[:60] or f"entry {i}"
            print(f"  [{'PASS' if ok else 'FAIL'}] {label}")
            for n in notes:
                print(f"         - {n}")
            if not ok:
                failed += 1
    print(f"\nRESULT: {'PASS' if failed == 0 else f'FAIL ({failed} entries unverifiable)'}")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
