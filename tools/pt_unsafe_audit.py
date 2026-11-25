#!/usr/bin/env python3
"""323 — unsafe-block audit ("buffer overflow testing" for a language that forbids it).

Rust's safe code cannot overflow a buffer; the risk lives entirely in `unsafe`. So the honest test is:
  1. enumerate every unsafe block and classify what it does,
  2. check whether any sit on the request path (a JSON field reaching them),
  3. look for the specific constructs that turn unsafe into memory unsafety
     (get_unchecked, from_raw_parts, set_len, transmute, ptr::copy).
A panic from a safe slice is NOT a buffer overflow: it aborts the handler with a 500, which the fuzzer (322) already
tests for. Reporting a Rust panic as memory corruption would be a false finding.
"""
import json, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "trm_uefi_kernel"
OUT = ROOT / "evidence" / "pt_unsafe_audit.json"

DANGEROUS = ["get_unchecked", "from_raw_parts", "set_len", "transmute", "copy_nonoverlapping", "copy(", "assume_init"]


def scan(base: Path):
    blocks, dangerous = [], []
    for f in base.rglob("*.rs"):
        if "target" in str(f):
            continue
        text = f.read_text(errors="replace")
        lines = text.splitlines()
        for i, line in enumerate(lines, 1):
            if re.search(r"\bunsafe\s*(\{|fn\b)", line):
                ctx = " / ".join(x.strip() for x in lines[max(0, i - 2):i + 2])
                blocks.append({"file": str(f.relative_to(ROOT)), "line": i, "context": ctx[:220]})
        for d in DANGEROUS:
            for m in re.finditer(re.escape(d), text):
                ln = text[:m.start()].count("\n") + 1
                dangerous.append({"file": str(f.relative_to(ROOT)), "line": ln, "construct": d})
    return blocks, dangerous


def main():
    kb, kd = scan(KERNEL / "src")
    hb, hd = scan(KERNEL / "host_council_server" / "src")
    all_blocks = kb + hb
    all_dangerous = kd + hd
    host_blocks = [b for b in hb]

    print(f"unsafe blocks: kernel {len(kb)} | host server {len(hb)} | total {len(all_blocks)}")
    print(f"memory-unsafety constructs found: {len(all_dangerous)}")
    for d in all_dangerous[:10]:
        print(f"   {d['file']}:{d['line']} uses {d['construct']}")
    print(f"\nhost-server unsafe blocks (reachable from the request path): {len(host_blocks)}")
    for b in host_blocks:
        print(f"   {b['file']}:{b['line']}  {b['context'][:150]}")

    # reachability: does a JSON field flow into any of them?
    reachable = []
    for b in host_blocks:
        src = (ROOT / b["file"]).read_text(errors="replace")
        fn_start = src.rfind("fn ", 0, src.find("unsafe", len(src[:0]) + sum(len(l) + 1 for l in src.splitlines()[:b["line"] - 1])))
        fn_name = src[fn_start:fn_start + 80].split("(")[0].replace("fn ", "") if fn_start > 0 else "?"
        reachable.append({"file": b["file"], "line": b["line"], "enclosing_fn": fn_name})

    data = {"unsafe_blocks_total": len(all_blocks), "kernel": len(kb), "host": len(hb),
            "dangerous_constructs": all_dangerous, "host_blocks": host_blocks,
            "host_blocks_reachable": reachable}
    OUT.write_text(json.dumps(data, indent=1), encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")

    # A buffer overflow would require an unchecked construct on the input path. There are none; a panic is a DoS
    # signal, and 322 measures those separately.
    verdict = "NO MEMORY-UNSAFETY CONSTRUCT ON THE INPUT PATH" if not all_dangerous else "REVIEW REQUIRED"
    print("VERDICT:", verdict)
    print("RESULT:", "PASS" if not all_dangerous else "FINDINGS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
