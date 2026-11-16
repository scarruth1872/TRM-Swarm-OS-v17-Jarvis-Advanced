#!/usr/bin/env python3
"""cpu_fix.py — why is the CPU tier's prefill 52x behind, and can it be fixed by configuration?

G103 measured 40 tok/s prefill on a CPU slot against 2,132 on the card. Two candidate causes:
  (a) configuration — too few threads, or a ubatch too small to batch prefill at all
  (b) the packing — Q2_0 is a ternary format whose CPU dequant path may be the real cost

This sweeps (a) on the ternary model, then tests (b) with a plain Q4_K_M model on the same cores.
Every measurement uses /completion so prefill and decode are reported by the server itself, not inferred.
"""
from __future__ import annotations

import json
import subprocess
import time
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
EXE = r"F:\Development sites\llama-vulkan-b12360\llama-server.exe"
TERNARY = r"E:\Hugginface models\Ternary-Bonsai-2-2B-Q2_0.gguf"
PLAIN = r"F:\Development sites\TRM-Swarm-OS-v2\models\qwen3-4b-instruct-q4_k_m.gguf"

LONG = "Audit this module. One line.\n" + (
    "pub fn handler(ctx: &Ctx) -> Result<u8> { let mut acc = 0u32; for slot in ctx.slots { "
    "acc = acc.wrapping_add(slot.trit as u32); } Ok(acc as u8) }\n" * 11)


def measure(port: int, prompt: str, n: int = 32) -> dict:
    body = {"prompt": prompt, "n_predict": n, "temperature": 0, "stream": False, "cache_prompt": False}
    t0 = time.time()
    r = json.loads(urllib.request.urlopen(urllib.request.Request(
        f"http://127.0.0.1:{port}/completion", data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json"}), timeout=600).read())
    tm = r.get("timings", {}) or {}
    return {"wall": round(time.time() - t0, 2), "prompt_n": tm.get("prompt_n"),
            "prefill_tok_s": round(1000 / tm["prompt_per_token_ms"], 1) if tm.get("prompt_per_token_ms") else None,
            "decode_tok_s": round(1000 / tm["predicted_per_token_ms"], 1) if tm.get("predicted_per_token_ms") else None}


def run(tag: str, model: str, threads: int, ub: int, batch: int, port: int, outdir: Path) -> dict:
    log = open(outdir / f"{tag}.log", "wb")
    cmd = [EXE, "--model", model, "--host", "127.0.0.1", "--port", str(port),
           "--n-gpu-layers", "0", "--threads", str(threads), "--parallel", "1",
           "--ctx-size", "2048", "-ub", str(ub), "-b", str(batch), "--no-op-offload"]
    t0 = time.time()
    proc = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT,
                            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    up = False
    while time.time() - t0 < 180:
        if proc.poll() is not None:
            break
        try:
            urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=3).read()
            up = True
            break
        except Exception:
            time.sleep(0.5)
    rec = {"tag": tag, "model": Path(model).name, "threads": threads, "ub": ub, "b": batch,
           "loaded": up, "load_s": round(time.time() - t0, 1)}
    if up:
        rec.update(measure(port, LONG))
    else:
        rec["fatal"] = (outdir / f"{tag}.log").read_text(errors="replace").splitlines()[-3:]
    proc.terminate()
    try:
        proc.wait(timeout=30)
    except Exception:
        proc.kill()
    time.sleep(1)
    print(f"  {tag:<18} load {rec['load_s']:>5}s | prefill {rec.get('prefill_tok_s')} tok/s | "
          f"decode {rec.get('decode_tok_s')} tok/s | wall {rec.get('wall')}s")
    return rec


def main() -> int:
    outdir = REPO / "scratch" / "cpu_fix"; outdir.mkdir(parents=True, exist_ok=True)
    print(f"CPU tier sweep | prompt {len(LONG)} chars (~602 tokens) | plain model: {Path(PLAIN).name} "
          f"{'present' if Path(PLAIN).exists() else 'MISSING'}")
    recs = []
    for tag, threads, ub, b in (("tern_t12_ub128", 12, 128, 2048),
                                ("tern_t12_ub512", 12, 512, 2048),
                                ("tern_t24_ub512", 24, 512, 2048),
                                ("tern_t24_ub1024", 24, 1024, 2048)):
        recs.append(run(tag, TERNARY, threads, ub, b, 8750, outdir))
    if Path(PLAIN).exists():
        for tag, threads, ub in (("plain_t24_ub512", 24, 512), ("plain_t12_ub512", 12, 512)):
            recs.append(run(tag, PLAIN, threads, ub, 2048, 8750, outdir))
    out = outdir / f"{time.strftime('%Y%m%d_%H%M%S')}_cpu_fix_summary.json"
    out.write_text(json.dumps(recs, indent=1), encoding="utf-8")
    best = max([r for r in recs if r.get("prefill_tok_s")], key=lambda r: r["prefill_tok_s"], default=None)
    if best:
        print(f"\nbest prefill: {best['tag']} at {best['prefill_tok_s']} tok/s "
              f"(vs G103 baseline 40.3 tok/s = {best['prefill_tok_s']/40.3:.1f}x)")
    print("record:", out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
