"""Seal journal blocks into the monad ledger, re-deriving what they cite.

A seal that only copies a receipt proves the receipt was written, not that it was true. So a block spec
does not carry its evidence as text: it carries *claims about files*, and this sealer recomputes them at
seal time and REFUSES TO WRITE if any one of them does not re-measure. Specs look like:

    {
      "kind": "bare_metal_supplicant",
      "fact": "...",
      "not_claimed": "...",
      "commit": "8cf8c39",
      "rederive": {
         "files": [{"path": "trm_uefi_kernel/trm_wifi/src/eapol.rs", "sha256": "<expected>"}],
         "oracles": [{"script": "trm_chainload/scripts/verify_wifi_crypto.py",
                      "binary": "<path to trm_wifi>", "expect_exit": 0}]
      }
    }

The digest is the canonical recipe (`sort_keys=True`, `separators=(",", ":")`) over the body with `hash`
absent -- the same recipe `swarm_v2/core/verify_ledger_chain.py` calls `canonical`. The chain is checked
BEFORE anything is appended, including the block about to be written, so a broken chain is a refusal
rather than a block that has to be repaired afterwards.

    python swarm_v2/core/seal_monad_journal.py --spec <spec.json>            # seal it
    python swarm_v2/core/seal_monad_journal.py --spec <spec.json> --dry-run  # re-derive and show, write nothing
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys
from datetime import datetime, timezone

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LEDGER = os.path.join(ROOT, "swarm_v2_artifacts", "monad_ledger.jsonl")
CANON = (",", ":")


def digest(body):
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=CANON).encode()).hexdigest()


def load_ledger():
    with open(LEDGER, encoding="utf-8") as fh:
        return [json.loads(ln) for ln in fh if ln.strip()]


def verdict(block):
    body = {k: v for k, v in block.items() if k != "hash"}
    if block.get("sealed") is False:
        return "unsealed" if block.get("unsealed_reason") else "UNVERIFIED"
    return "canonical" if digest(body) == block.get("hash") else (
        "alt_separators" if hashlib.sha256(
            json.dumps(body, sort_keys=True, separators=(", ", ": ")).encode()).hexdigest() == block.get("hash")
        else "UNVERIFIED")


def chain_problems(blocks):
    """Every problem a chain can have, named. Declared-unsealed blocks are their own state, not failures."""
    problems, prev = [], None
    for b in blocks:
        v = verdict(b)
        if v == "UNVERIFIED":
            problems.append(f"block {b.get('index')} is UNVERIFIED (its hash is not reproducible from its body)")
        if prev is not None and b.get("prev_hash") != prev:
            problems.append(f"block {b.get('index')} links to {str(b.get('prev_hash'))[:12]} but {prev[:12]} came before it")
        prev = b.get("hash")
    return problems


def wsl_path(path):
    """A Windows path as the WSL side sees it. The oracle has to run where the Linux ELF can execute, and
    a C:/... path is not a path there -- it is a filename with a colon in it."""
    if len(path) > 1 and path[1] == ":":
        return "/mnt/" + path[0].lower() + path[2:]
    return path


def rederive(spec):
    """Re-measure every claim the block makes about the world. Returns the measured evidence dict."""
    red = spec.get("rederive", {})
    measured = {}
    for entry in red.get("files", []):
        path = os.path.join(ROOT, entry["path"])
        if not os.path.exists(path):
            raise SystemExit(f"refusing to seal: {entry['path']} does not exist to be measured")
        with open(path, "rb") as fh:
            got = hashlib.sha256(fh.read()).hexdigest()
        want = entry["sha256"]
        if got != want:
            raise SystemExit(f"refusing to seal: {entry['path']} is {got[:16]}, the block claims {want[:16]}")
        measured.setdefault("files", {})[entry["path"]] = {
            "sha256": got, "bytes": os.path.getsize(path)}
    for entry in red.get("oracles", []):
        script = os.path.join(ROOT, entry["script"]).replace("\\", "/")
        if entry.get("shell") == "wsl":
            # The artifact under test is a Linux ELF, so the oracle has to run where it can execute --
            # through WSL. Its stdout carries NUL bytes and a distro warning, so both are cleaned here
            # rather than being left to look like part of the answer.
            inner = " ".join([f'"{wsl_path(script)}"'] +
                             [f'"{wsl_path(a)}"' for a in entry.get("args", [])])
            proc = subprocess.run(["wsl.exe", "-e", "bash", "-c",
                                   f"python3 {inner}"], capture_output=True, text=True)
            out = "\n".join(l for l in proc.stdout.replace("\x00", "").splitlines()
                            if "cargo/env" not in l and not l.startswith("wsl:"))
        else:
            argv = [sys.executable, script] + entry.get("args", [])
            proc = subprocess.run(argv, capture_output=True, text=True)
            out = proc.stdout
        if proc.returncode != entry.get("expect_exit", 0):
            raise SystemExit(
                f"refusing to seal: {entry['script']} exited {proc.returncode}, not {entry.get('expect_exit', 0)} -- "
                f"the thing this block vouches for does not currently hold")
        for want in entry.get("expect_stdout_contains", []):
            if want not in out:
                raise SystemExit(
                    f"refusing to seal: {entry['script']} did not print {want!r} -- the claim this block makes "
                    f"about it is not what the oracle currently says")
        measured.setdefault("oracles", {})[entry["script"]] = {
            "exit": proc.returncode,
            "last_line": out.strip().splitlines()[-1].strip() if out.strip() else "",
            "expect_stdout_contains": entry.get("expect_stdout_contains", []),
        }
    return measured


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--spec", required=True, help="JSON: one block body, or a list of them, in order")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    specs = json.load(open(args.spec, encoding="utf-8"))
    if isinstance(specs, dict):
        specs = [specs]

    blocks = load_ledger()
    problems = chain_problems(blocks)
    if problems:
        print("the ledger is not intact before this seal -- refusing to append to it:")
        for p in problems:
            print("   " + p)
        return 1
    tip = blocks[-1]
    print(f"  tip: index {tip.get('index')}, {str(tip.get('hash'))[:16]}...  ({len(blocks)} blocks)")

    new = []
    for spec in specs:
        measured = rederive(spec)
        body = {k: v for k, v in spec.items() if k != "rederive"}
        body["index"] = (new[-1]["index"] if new else tip["index"]) + 1
        body["anchor"] = tip.get("anchor")
        body["prev_hash"] = new[-1]["hash"] if new else tip["hash"]
        body["sealed_at"] = datetime.now(timezone.utc).replace(microsecond=0).isoformat()
        body["evidence"] = measured
        body["hash"] = digest(body)
        new.append(body)
        print(f"  re-derived ok -> index {body['index']}  {body['kind']}  {body['hash'][:16]}...")

    problems = chain_problems(blocks + new)
    if problems:
        print("refusing to write: the chain would not verify afterwards:")
        for p in problems:
            print("   " + p)
        return 1
    if args.dry_run:
        print("  dry run: every claim re-measured, the chain would hold, nothing written")
        return 0

    with open(LEDGER, "a", encoding="utf-8") as fh:
        for b in new:
            fh.write(json.dumps(b, sort_keys=True) + "\n")
        fh.flush()
        os.fsync(fh.fileno())
    print(f"  sealed {len(new)} block(s); the ledger now has {len(blocks) + len(new)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
