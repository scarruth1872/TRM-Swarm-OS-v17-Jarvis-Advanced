#!/usr/bin/env python3
"""bench_gguf.py — sweep llama-server flags on one GGUF and report tokens/second.

WHY: 4.9 tok/s on a 12 GB card for a 10.9 GB IQ2 model is not obviously a flag problem —
it can be a full offload that still syncs to the CPU every token (hybrid SSM blocks), a
partial offload nobody reported, or simply KV traffic that never touches the card. Guessing
which one wastes an afternoon. So each candidate configuration is started, measured with the
same forced-length generation, and unloaded; the table says which one is fastest and the
server log is kept for the loader's own offload report.

Each config is a child process we start and terminate ourselves — no taskkill.

USAGE
  python tools/bench_gguf.py --model "E:/.../X.gguf" --tag qwen38
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
DEFAULT_EXE = r"F:\Development sites\ub3\llama-vulkan-win\llama-server.exe"

# one prompt, forced to a fixed length, so every config generates the same work
PROMPT = "Write a Python function that returns the nth Fibonacci number, with a docstring."
MAX_TOKENS = 192

CONFIGS = [
    # name,                       extra flags
    ("cpu_only",                  ["--n-gpu-layers", "0"]),
    ("gpu99_base",                ["--n-gpu-layers", "99"]),
    ("gpu99_fa",                  ["--n-gpu-layers", "99", "-fa", "on"]),
    ("gpu99_fa_kvq8",             ["--n-gpu-layers", "99", "-fa", "on", "-ctk", "q8_0", "-ctv", "q8_0"]),
    ("gpu99_fa_batch",            ["--n-gpu-layers", "99", "-fa", "on", "-b", "2048", "-ub", "512"]),
    ("gpu99_fa_nommap",           ["--n-gpu-layers", "99", "-fa", "on", "--no-mmap"]),
    # Second round: the model needs ~11.8 GB and the card has 11.47 GB free, so llama.cpp's
    # auto-fit leaves ~2.9 GB host-mapped while still reporting "65/65 layers on GPU". These
    # configs attack THAT split rather than the flags that had no effect at all.
    ("fa_ub_small",               ["--n-gpu-layers", "99", "-fa", "on", "-ub", "128", "-b", "512"]),
    ("fa_ctx512",                 ["--n-gpu-layers", "99", "-fa", "on", "--ctx-size", "512", "-ub", "128"]),
    ("fa_nokvoffload",            ["--n-gpu-layers", "99", "-fa", "on", "--no-kv-offload"]),
    ("gpu60_fa",                  ["--n-gpu-layers", "60", "-fa", "on"]),
    # Round 3: ubatch 128 beat 512 by 15%, so the axis is per-step overhead, not raw throughput.
    # n_slots defaults to 4 here (measured: "initializing, n_slots = 4"), which multiplies KV and
    # recurrent state; a single-slot server has no reason to pay for four.
    ("fa_ub64",                   ["--n-gpu-layers", "99", "-fa", "on", "-ub", "64", "-b", "256"]),
    ("fa_ub128_par1",             ["--n-gpu-layers", "99", "-fa", "on", "-ub", "128", "-b", "512", "--parallel", "1"]),
    ("fa_ub128_par1_t4",          ["--n-gpu-layers", "99", "-fa", "on", "-ub", "128", "-b", "512", "--parallel", "1", "-t", "4"]),
    ("fa_ub256",                  ["--n-gpu-layers", "99", "-fa", "on", "-ub", "256", "-b", "512"]),
    # Round 4: can the 2.9 GB host-mapped tail be pulled onto the card at all?
    ("best",                      ["--n-gpu-layers", "99", "-fa", "on", "-ub", "128", "-b", "512"]),
    ("best_ot_embd_gpu",          ["--n-gpu-layers", "99", "-fa", "on", "-ub", "128", "-b", "512",
                                   "-ot", "token_embd\\.weight=Vulkan0"]),
    ("best_ot_out_gpu",           ["--n-gpu-layers", "99", "-fa", "on", "-ub", "128", "-b", "512",
                                   "-ot", "output\\.weight=Vulkan0"]),
    ("fa_ub128_par1_kvq8",        ["--n-gpu-layers", "99", "-fa", "on", "-ub", "128", "-b", "512", "--parallel", "1", "-ctk", "q8_0", "-ctv", "q8_0"]),
    ("gpu48_fa",                  ["--n-gpu-layers", "48", "-fa", "on"]),
]


def ps(script: str) -> str:
    r = subprocess.run(["powershell", "-NoProfile", "-Command", script],
                       capture_output=True, text=True, timeout=120)
    return r.stdout.strip()


def gpu_gb() -> float:
    out = ps(r"(Get-Counter '\GPU Adapter Memory(*)\Dedicated Usage' -ErrorAction SilentlyContinue)."
             r"CounterSamples | Where-Object {$_.CookedValue -gt 100MB} | "
             r"Measure-Object -Property CookedValue -Maximum | Select-Object -ExpandProperty Maximum")
    try:
        return round(float(out) / 1024 ** 3, 2)
    except (TypeError, ValueError):
        return -1.0


def post(url: str, body: dict, timeout: float = 300.0) -> dict:
    req = urllib.request.Request(url, data=json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def get(url: str, timeout: float = 6.0) -> dict:
    with urllib.request.urlopen(url, timeout=timeout) as r:
        return json.loads(r.read())


def run_config(exe: Path, model: Path, port: int, name: str, flags: list[str],
               ctx: int, outdir: Path, tag: str) -> dict:
    log = outdir / f"{tag}_{name}.log"
    # -lv 4 is what makes the loader print where the weights actually went
    # ("CPU_Mapped model buffer size" vs "Vulkan0 model buffer size"); without it every config
    # looks identical and the 2.9 GB host-mapped tail is invisible.
    cmd = [str(exe), "--model", str(model), "--host", "127.0.0.1", "--port", str(port),
           "--ctx-size", str(ctx), "--threads", "8", "-lv", "4"] + flags
    before = gpu_gb()
    t0 = time.time()
    with open(log, "wb") as lf:
        proc = subprocess.Popen(cmd, stdout=lf, stderr=subprocess.STDOUT,
                                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    up = False
    FATAL = ("error loading model", "failed to load model", "GGML_ASSERT", "unknown model architecture")
    while time.time() - t0 < 600:
        if proc.poll() is not None:
            break
        try:
            get(f"http://127.0.0.1:{port}/health", 3)
            up = True
            break
        except Exception:
            try:
                if any(f in log.read_text(errors="replace") for f in FATAL):
                    break
            except OSError:
                pass
            time.sleep(1.0)
    load_s = round(time.time() - t0, 1)
    rec: dict = {"config": name, "flags": flags, "ctx": ctx, "loaded": up, "load_seconds": load_s,
                 "gpu_before_gb": before}
    if not up:
        txt = log.read_text(errors="replace")
        rec["fatal"] = [l for l in txt.splitlines() if any(f in l for f in FATAL)][:3]
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=20)
            except subprocess.TimeoutExpired:
                proc.kill()
        rec["gpu_during_gb"] = gpu_gb()
        print(f"  {name:<20} FAILED   {rec['fatal'][:1]}")
        return rec

    vram = gpu_gb()
    rec["gpu_during_gb"] = vram
    # the loader's own account of what it put where (build-dependent, kept verbatim if present)
    txt = log.read_text(errors="replace")
    rec["offload_lines"] = [l.strip() for l in txt.splitlines()
                            if "offload" in l.lower() or "buffer size" in l.lower()][:8]
    # the numbers that explain the speed, pulled out of the log instead of eyeballed
    import re as _re
    def _mb(pat):
        m = _re.search(pat, txt)
        return float(m.group(1)) if m else None
    rec["vram_weights_mb"] = _mb(r"Vulkan0 model buffer size =\s*([\d.]+) MiB")
    rec["host_weights_mb"] = _mb(r"CPU_Mapped model buffer size =\s*([\d.]+) MiB")
    _lay = _re.search(r"offloaded (\d+/\d+) layers", txt)
    rec["layers_offloaded"] = _lay.group(1) if _lay else None

    body = {"messages": [{"role": "user", "content": PROMPT}], "temperature": 0,
            "max_tokens": MAX_TOKENS, "ignore_eos": True, "stream": False}
    t = time.time()
    try:
        r = post(f"http://127.0.0.1:{port}/v1/chat/completions", body)
        el = time.time() - t
        usage = r.get("usage", {}) or {}
        ct = usage.get("completion_tokens") or 0
        pt = usage.get("prompt_tokens") or 0
        msg = (r.get("choices") or [{}])[0].get("message", {}) or {}
        rec.update({"seconds": round(el, 2), "completion_tokens": ct, "prompt_tokens": pt,
                    "tok_per_s": round(ct / el, 2) if ct else None,
                    "sample": (msg.get("content") or msg.get("reasoning_content") or "")[:120]})
        print(f"  {name:<20} {el:6.2f}s  {ct:>4} tok  {rec['tok_per_s']:>6} tok/s  "
              f"gpu {vram:>5} GB  load {load_s:>5}s")
    except Exception as e:
        rec["error"] = f"{type(e).__name__}: {e}"
        print(f"  {name:<20} ERROR {type(e).__name__}")

    proc.terminate()
    try:
        proc.wait(timeout=30)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=30)
    time.sleep(2)
    return rec


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", required=True)
    ap.add_argument("--exe", default=DEFAULT_EXE)
    ap.add_argument("--port", type=int, default=8603)
    ap.add_argument("--ctx", type=int, default=2048)
    ap.add_argument("--tag", default="bench")
    ap.add_argument("--only", default="", help="comma separated config names")
    args = ap.parse_args()

    model, exe = Path(args.model), Path(args.exe)
    for p in (model, exe):
        if not p.exists():
            print(f"FAIL: missing {p}")
            return 2
    outdir = REPO / "scratch" / "bench"
    outdir.mkdir(parents=True, exist_ok=True)
    stamp = time.strftime("%Y%m%d_%H%M%S")

    configs = CONFIGS
    if args.only:
        want = {s.strip() for s in args.only.split(",") if s.strip()}
        configs = [c for c in CONFIGS if c[0] in want]

    print(f"model: {model.name} ({model.stat().st_size/1024**3:.2f} GB) | ctx {args.ctx} | "
          f"{MAX_TOKENS} forced tokens per config | gpu free {gpu_gb()} GB")
    recs = []
    for name, flags in configs:
        recs.append(run_config(exe, model, args.port, name, flags, args.ctx, outdir, f"{stamp}_{args.tag}"))

    recs.sort(key=lambda r: -(r.get("tok_per_s") or 0))
    out = outdir / f"{stamp}_{args.tag}_summary.json"
    out.write_text(json.dumps(recs, indent=2), encoding="utf-8")
    print("\n=== ranked ===")
    for r in recs:
        print(f"  {r['config']:<20} {str(r.get('tok_per_s')):>7} tok/s   "
              f"gpu {str(r.get('gpu_during_gb')):>5} GB   {'ok' if r['loaded'] else 'FAILED'}")
    print(f"\nsummary: {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
