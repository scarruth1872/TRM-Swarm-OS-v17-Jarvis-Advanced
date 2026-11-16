#!/usr/bin/env python3
"""spec_decode_test.py — does a small draft model make a bigger one faster, with the same output?

WHY: the Bonsai family shares one 248,320-token vocabulary (measured from the GGUF headers), which is
exactly the condition llama.cpp's speculative decoding requires — a draft model may only propose
tokens the target can also emit. That makes "smallest model leads, largest confirms" testable today,
without training anything.

The gate is stated before the run, not after:
  1. throughput must improve (target tokens/second, same prompts, same budget),
  2. the generated text must be IDENTICAL to the baseline under greedy sampling — a speedup that
     changes the answer is not a speedup, it is a different model.

USAGE
  python tools/spec_decode_test.py --target <big.gguf> --draft <small.gguf> --exe <server.exe>
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import time
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
PROMPTS = [
    ("arith", "What is 17 * 23? Answer with the number only."),
    ("code", "Write a Python function that returns the nth Fibonacci number. Code only."),
    ("recall", "In one sentence, what does the acronym RAID stand for?"),
]
MAX_TOKENS = 192


def gpu_gb() -> float:
    r = subprocess.run(["powershell", "-NoProfile", "-Command",
        r"(Get-Counter '\GPU Adapter Memory(*)\Dedicated Usage' -ErrorAction SilentlyContinue)."
        r"CounterSamples | Where-Object {$_.CookedValue -gt 50MB} | "
        r"Measure-Object -Property CookedValue -Maximum | Select-Object -ExpandProperty Maximum"],
        capture_output=True, text=True, timeout=90)
    try:
        return round(float(r.stdout.strip()) / 1024 ** 3, 2)
    except ValueError:
        return -1.0


def post(url: str, body: dict, timeout: float = 600.0) -> dict:
    req = urllib.request.Request(url, data=json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def run(exe: Path, target: Path, port: int, tag: str, extra: list[str], outdir: Path) -> dict:
    log = outdir / f"{tag}.log"
    cmd = [str(exe), "--model", str(target), "--host", "127.0.0.1", "--port", str(port),
           "--n-gpu-layers", "99", "-fa", "on", "--ctx-size", "4096", "--threads", "8"] + extra
    t0 = time.time()
    with open(log, "wb") as lf:
        proc = subprocess.Popen(cmd, stdout=lf, stderr=subprocess.STDOUT,
                                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    up = False
    while time.time() - t0 < 420:
        if proc.poll() is not None:
            break
        try:
            urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=3).read()
            up = True
            break
        except Exception:
            if "error loading model" in log.read_text(errors="replace"):
                break
            time.sleep(1)
    res = {"tag": tag, "loaded": up, "load_s": round(time.time() - t0, 1), "vram_gb": gpu_gb(), "prompts": []}
    if not up:
        res["fatal"] = log.read_text(errors="replace").splitlines()[-6:]
        if proc.poll() is None:
            proc.terminate()
        return res

    for name, prompt in PROMPTS:
        body = {"messages": [{"role": "user", "content": prompt}], "temperature": 0,
                "max_tokens": MAX_TOKENS, "ignore_eos": True, "stream": False,
                "seed": 1234}
        t = time.time()
        r = post(f"http://127.0.0.1:{port}/v1/chat/completions", body)
        el = time.time() - t
        usage = r.get("usage", {}) or {}
        msg = (r.get("choices") or [{}])[0].get("message", {}) or {}
        txt = (msg.get("content") or msg.get("reasoning_content") or "").strip()
        ct = usage.get("completion_tokens") or 0
        res["prompts"].append({"name": name, "seconds": round(el, 2), "tokens": ct,
                               "tok_s": round(ct / el, 2) if ct else None, "text": txt})

    txt = log.read_text(errors="replace")
    # llama.cpp reports draft acceptance in the server log; keep whatever it printed
    res["draft_lines"] = [l.strip() for l in txt.splitlines()
                          if re.search(r"draft|accept", l, re.I)][-6:]
    res["server_tok_s"] = [float(m) for m in re.findall(r"([\d.]+) tokens per second", txt)][-3:]

    proc.terminate()
    try:
        proc.wait(timeout=30)
    except subprocess.TimeoutExpired:
        proc.kill()
    time.sleep(2)
    return res


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--target", required=True)
    ap.add_argument("--draft", default="")
    ap.add_argument("--exe", required=True)
    ap.add_argument("--port", type=int, default=8614)
    # MEASURED 2026-11-13: build b12360 REMOVED --draft-max; it errors "the argument has been removed,
    # use --spec-draft-n-max". The flag name is version-dependent, so it is a parameter, not a constant.
    ap.add_argument("--draft-flag", default="--spec-draft-n-max",
                    help="build-specific name; b12360 uses --spec-draft-n-max, older builds --draft-max")
    ap.add_argument("--draft-max", type=int, default=8)
    # MEASURED 2026-11-13: loading a draft with -md is NOT enough on b12360 - --spec-type defaults to
    # "none", so the draft sat idle and every result was x1.00 with zero acceptance lines in the log.
    # Values: none,draft-simple,draft-eagle3,draft-mtp,draft-dflash,draft-dspark,ngram-*.
    ap.add_argument("--spec-type", default="draft-simple")
    ap.add_argument("--tag", default="spec")
    args = ap.parse_args()

    exe, target = Path(args.exe), Path(args.target)
    for p in [exe, target]:
        if not p.exists():
            print(f"FAIL: missing {p}")
            return 2
    outdir = REPO / "scratch" / "spec"; outdir.mkdir(parents=True, exist_ok=True)
    stamp = time.strftime("%Y%m%d_%H%M%S")

    print(f"target: {target.name} ({target.stat().st_size/1024**3:.2f} GB) | gpu free {gpu_gb()} GB")
    base = run(exe, target, args.port, f"{stamp}_{args.tag}_baseline", ["-ub", "128", "-b", "512"], outdir)
    print(f"  baseline loaded={base['loaded']} load={base['load_s']}s vram={base['vram_gb']}GB")
    for p in base.get("prompts", []):
        print(f"    {p['name']:<7} {p['tok_s']} tok/s  {p['tokens']} tok  {p['text'][:40]!r}")

    if not base["loaded"]:
        print("FAIL: baseline did not load; nothing to compare")
        return 1
    if not args.draft:
        return 0

    print(f"\ndraft: {Path(args.draft).name} ({Path(args.draft).stat().st_size/1024**3:.2f} GB) "
          f"{args.draft_flag} {args.draft_max} --spec-type {args.spec_type}")
    spec = run(exe, target, args.port, f"{stamp}_{args.tag}_spec",
               ["-ub", "128", "-b", "512", "-md", args.draft, args.draft_flag, str(args.draft_max),
                "--spec-type", args.spec_type], outdir)
    print(f"  spec loaded={spec['loaded']} load={spec['load_s']}s vram={spec['vram_gb']}GB")
    for p in spec.get("prompts", []):
        print(f"    {p['name']:<7} {p['tok_s']} tok/s  {p['tokens']} tok  {p['text'][:40]!r}")
    for l in spec.get("draft_lines", []):
        print("   log:", l[:130])

    # MEASURED 2026-11-13: when the spec server failed to load (removed --draft-max flag), the prompt
    # lists were empty, zip() produced nothing, and the gate printed "mean speedup x0.00; all
    # identical=True" - a vacuous truth that reads as a pass. An empty comparison is a FAILED
    # measurement and must say so.
    if not spec["loaded"]:
        print("\n=== gate: NOT RUN ===")
        print("  the speculative server never loaded, so there is no comparison:")
        for l in spec.get("fatal", [])[-3:]:
            print("   ", l[:150])
        out = outdir / f"{stamp}_{args.tag}_compare.json"
        out.write_text(json.dumps({"baseline": base, "spec": spec,
                                   "compare": [], "gate": "NOT_RUN_SPEC_NOT_LOADED"}, indent=1), encoding="utf-8")
        print("record:", out)
        return 1

    same = []
    for a, b in zip(base.get("prompts", []), spec.get("prompts", [])):
        same.append({"prompt": a["name"], "identical": a["text"] == b["text"],
                     "base_tok_s": a["tok_s"], "spec_tok_s": b["tok_s"],
                     "speedup": round(b["tok_s"] / a["tok_s"], 2) if a["tok_s"] and b["tok_s"] else None})
    out = outdir / f"{stamp}_{args.tag}_compare.json"
    out.write_text(json.dumps({"baseline": base, "spec": spec, "compare": same}, indent=1), encoding="utf-8")

    print("\n=== gate ===")
    for s in same:
        print(f"  {s['prompt']:<7} {s['base_tok_s']} -> {s['spec_tok_s']} tok/s  "
              f"x{s['speedup']}  identical={s['identical']}")
    if not same:
        print("  gate: NOT RUN - no paired prompts were produced")
        return 1
    mean_sp = sum(s["speedup"] or 0 for s in same) / len(same)
    verdict = "PASS" if (mean_sp > 1.0 and all(s["identical"] for s in same)) else "FAIL"
    print(f"  mean speedup x{mean_sp:.2f}; all identical={all(s['identical'] for s in same)}  -> {verdict}")
    print("record:", out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
