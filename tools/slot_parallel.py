#!/usr/bin/env python3
"""slot_parallel.py — one server, N slots: the way concurrency is meant to work on a single GPU.

MEASURED CONTRAST (2026-11-13): five separate llama-server processes, each with the same 0.51 GB
2B model, produced 0.53 tok/s AGGREGATE (0.11-1.9 each) against 107 tok/s for one process. The card
serialises contexts and every process pays its own KV/compute buffers. The supported way to serve N
concurrent agents from one GPU is ONE process with --parallel N, where weights are shared and decode
is batched - so this measures exactly that, with the same model, prompt and token budget.
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
                                   headers={"Content-Type": "application/json"}), timeout=600).read())
        el = time.time() - t0
        ct = (r.get("usage", {}) or {}).get("completion_tokens") or 0
        out[key] = {"seconds": round(el, 2), "tokens": ct, "tok_s": round(ct / el, 2) if el else 0}
    except Exception as e:
        out[key] = {"error": type(e).__name__}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--slots", type=int, default=8)
    ap.add_argument("--tokens", type=int, default=32)
    ap.add_argument("--port", type=int, default=8730)
    args = ap.parse_args()

    outdir = REPO / "scratch" / "parallel"; outdir.mkdir(parents=True, exist_ok=True)
    stamp = time.strftime("%Y%m%d_%H%M%S")
    log = open(outdir / f"{stamp}_slots{args.slots}.log", "wb")
    cmd = [EXE, "--model", MODEL, "--host", "127.0.0.1", "--port", str(args.port),
           "--n-gpu-layers", "99", "-fa", "on", "--ctx-size", str(2048 * args.slots),
           "--parallel", str(args.slots), "--threads", "4", "-ub", "128", "-b", "512"]
    print(f"one server, {args.slots} slots | ctx {2048*args.slots}")
    t0 = time.time()
    proc = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT,
                            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    up = False
    while time.time() - t0 < 240:
        if proc.poll() is not None:
            break
        try:
            urllib.request.urlopen(f"http://127.0.0.1:{args.port}/health", timeout=3).read()
            up = True
            break
        except Exception:
            time.sleep(0.5)
    print(f"  loaded={up} in {round(time.time()-t0,1)}s")
    if not up:
        proc.terminate()
        return 1

    results: dict = {}
    ths = [threading.Thread(target=post, args=(args.port, args.tokens, results, f"slot{i}"))
           for i in range(args.slots)]
    t1 = time.time()
    for t in ths: t.start()
    for t in ths: t.join()
    wall = time.time() - t1

    ok = [v for v in results.values() if "tok_s" in v]
    agg = sum(v["tokens"] for v in ok)
    print(f"\n  wall {wall:.2f}s | {len(ok)}/{args.slots} answered | "
          f"aggregate {agg/wall:.1f} tok/s | mean per request {sum(v['tok_s'] for v in ok)/max(1,len(ok)):.1f} tok/s")
    for k, v in sorted(results.items()): print(f"    {k:<7} {v}")

    rec = {"mode": "single-process-slots", "slots": args.slots, "wall": round(wall, 2),
           "answered": len(ok), "aggregate_tok_s": round(agg / wall, 2) if wall else 0,
           "mean_tok_s": round(sum(v["tok_s"] for v in ok) / len(ok), 2) if ok else None,
           "results": results}
    path = outdir / f"{stamp}_slots{args.slots}_summary.json"
    path.write_text(json.dumps(rec, indent=1), encoding="utf-8")
    print("record:", path)

    proc.terminate()
    try: proc.wait(timeout=30)
    except Exception: proc.kill()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
