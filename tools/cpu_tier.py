#!/usr/bin/env python3
"""cpu_tier.py — the CPU-ternary tier, measured instead of assumed.

CONTEXT: in G100 eight CPU-hosted ternary instances were created and pinned (affinity verified 8/8) but
never reached /health inside a 120 s window, so the tier was recorded as UNMEASURED. The likely causes
were 3 threads per instance and a deadline that assumed GPU-like load times. This measures the tier
properly: one instance with real thread counts, then a multi-slot CPU server.

WHAT A CPU-TERNARY TIER IS GOOD FOR: if the 2B answers at even a fraction of GPU speed, it buys
capacity - N agents answering while the GPU serves the reasoning tier - without touching VRAM.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import threading
import time
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
EXE = r"F:\Development sites\llama-vulkan-b12360\llama-server.exe"
MODEL = r"E:\Hugginface models\Ternary-Bonsai-2-2B-Q2_0.gguf"
PROMPT = "Name the three laws of thermodynamics in one short list."


def post(port: int, tokens: int, out: dict, key: str) -> None:
    body = {"messages": [{"role": "user", "content": PROMPT}], "temperature": 0,
            "max_tokens": tokens, "ignore_eos": True, "stream": False,
            "chat_template_kwargs": {"enable_thinking": False}}
    t0 = time.time()
    try:
        r = json.loads(urllib.request.urlopen(
            urllib.request.Request(f"http://127.0.0.1:{port}/v1/chat/completions",
                                   data=json.dumps(body).encode(),
                                   headers={"Content-Type": "application/json"}), timeout=900).read())
        el = time.time() - t0
        ct = (r.get("usage", {}) or {}).get("completion_tokens") or 0
        txt = ((r.get("choices") or [{}])[0].get("message", {}) or {}).get("content", "")
        out[key] = {"seconds": round(el, 2), "tokens": ct, "tok_s": round(ct / el, 2) if el else 0,
                    "text": txt[:80]}
    except Exception as e:
        out[key] = {"error": type(e).__name__}


def run(tag: str, threads: int, parallel: int, tokens: int, port: int, outdir: Path) -> dict:
    log = open(outdir / f"{tag}.log", "wb")
    cmd = [EXE, "--model", MODEL, "--host", "127.0.0.1", "--port", str(port),
           "--n-gpu-layers", "0", "--threads", str(threads), "--parallel", str(parallel),
           "--ctx-size", str(2048 * parallel), "--no-op-offload"]
    t0 = time.time()
    proc = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT,
                            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    up = False
    while time.time() - t0 < 300:
        if proc.poll() is not None:
            break
        try:
            urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=3).read()
            up = True
            break
        except Exception:
            time.sleep(1)
    load_s = round(time.time() - t0, 1)
    print(f"  {tag}: loaded={up} in {load_s}s (threads {threads}, parallel {parallel})")
    rec = {"tag": tag, "threads": threads, "parallel": parallel, "loaded": up, "load_s": load_s}
    if not up:
        t = (outdir / f"{tag}.log").read_text(errors="replace").splitlines()[-4:]
        rec["fatal"] = t
        print("   ", t)
        proc.terminate()
        return rec

    results: dict = {}
    ths = [threading.Thread(target=post, args=(port, tokens, results, f"r{i}")) for i in range(parallel)]
    t1 = time.time()
    for t in ths: t.start()
    for t in ths: t.join()
    wall = time.time() - t1
    ok = [v for v in results.values() if "tok_s" in v]
    agg = sum(v["tokens"] for v in ok)
    rec.update({"wall": round(wall, 2), "answered": len(ok),
                "aggregate_tok_s": round(agg / wall, 2) if wall else 0,
                "mean_tok_s": round(sum(v["tok_s"] for v in ok) / len(ok), 2) if ok else None,
                "results": results})
    print(f"    {len(ok)}/{parallel} answered | aggregate {rec['aggregate_tok_s']} tok/s | "
          f"mean per request {rec['mean_tok_s']} tok/s")
    for k, v in sorted(results.items()):
        print(f"     {k}: {v.get('tok_s')} tok/s  {v.get('text','')[:50]!r}")
    proc.terminate()
    try: proc.wait(timeout=30)
    except Exception: proc.kill()
    time.sleep(1)
    return rec


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--tokens", type=int, default=32)
    ap.add_argument("--port", type=int, default=8740)
    args = ap.parse_args()
    outdir = REPO / "scratch" / "cpu_tier"; outdir.mkdir(parents=True, exist_ok=True)
    stamp = time.strftime("%Y%m%d_%H%M%S")
    recs = []
    print(f"CPU-ternary tier: {Path(MODEL).name}")
    recs.append(run(f"{stamp}_cpu1_t12", 12, 1, args.tokens, args.port, outdir))
    recs.append(run(f"{stamp}_cpu4_t6", 6, 4, args.tokens, args.port, outdir))
    recs.append(run(f"{stamp}_cpu6_t4", 4, 6, args.tokens, args.port, outdir))
    out = outdir / f"{stamp}_cpu_tier_summary.json"
    out.write_text(json.dumps(recs, indent=1), encoding="utf-8")
    print("\n=== CPU-ternary ===")
    for r in recs:
        print(f"  threads {r['threads']:>2} parallel {r['parallel']}: "
              f"{r.get('aggregate_tok_s','-')} tok/s aggregate, {r.get('mean_tok_s','-')} each, "
              f"load {r.get('load_s')}s")
    print("record:", out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
