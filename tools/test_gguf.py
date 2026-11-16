#!/usr/bin/env python3
"""test_gguf.py — load one GGUF, ask it three fixed questions, measure, unload.

WHY: "test the models" has to produce numbers, not impressions. This starts exactly one
llama-server (its own child process, terminated by us, never taskkill), waits for /health,
sends three fixed prompts at temperature 0, records latency and tokens/second per prompt,
samples GPU dedicated memory and process commit while it runs, then unloads and verifies the
port is dead. Each run writes a JSON record so two models can be compared later without
re-asking them anything.

USAGE
  python tools/test_gguf.py --model "E:/Hugginface models/X.gguf" --port 8601 --ngl 99 --ctx 2048
  python tools/test_gguf.py --model ... --exe "D:/llama-instella/.../llama-server.exe"

Exit codes: 0 measured and unloaded · 1 the server never became healthy · 2 the harness failed.
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
DEFAULT_EXE = r"F:\Development sites\ub3\llama-vulkan-win\llama-server.exe"
INSTELLA_EXE = r"D:\llama-instella\llama.cpp\build\bin\llama-server.exe"

PROMPTS = [
    ("arith", "What is 17 * 23? Answer with the number only."),
    ("code", "Write a Python function that returns the nth Fibonacci number. Code only."),
    ("recall", "In one sentence, what does the acronym RAID stand for?"),
]


def ps(script: str) -> str:
    r = subprocess.run(["powershell", "-NoProfile", "-Command", script],
                       capture_output=True, text=True, timeout=120)
    return r.stdout.strip()


def gpu_dedicated_gb() -> float:
    out = ps(r"(Get-Counter '\GPU Adapter Memory(*)\Dedicated Usage' -ErrorAction SilentlyContinue)."
             r"CounterSamples | Where-Object {$_.CookedValue -gt 100MB} | "
             r"Measure-Object -Property CookedValue -Maximum | Select-Object -ExpandProperty Maximum")
    try:
        return round(float(out) / 1024 ** 3, 2)
    except (TypeError, ValueError):
        return -1.0


def proc_mem_mb(pid: int) -> tuple[int, int]:
    """(working set MB, private commit MB) for a pid — measured, not estimated."""
    out = ps(f"$p=Get-Process -Id {pid} -ErrorAction SilentlyContinue; "
             f"if($p){{'{{0}} {{1}}' -f [math]::Round($p.WorkingSet64/1MB),[math]::Round($p.PrivateMemorySize64/1MB)}}")
    try:
        ws, priv = out.split()
        return int(ws), int(priv)
    except ValueError:
        return -1, -1


def http_json(url: str, payload: dict | None = None, timeout: float = 180.0) -> dict:
    data = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(url, data=data,
                                 headers={"Content-Type": "application/json"} if data else {})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", required=True)
    ap.add_argument("--exe", default=DEFAULT_EXE)
    ap.add_argument("--port", type=int, default=8601)
    ap.add_argument("--ngl", type=int, default=99)
    ap.add_argument("--ctx", type=int, default=2048)
    ap.add_argument("--max-tokens", type=int, default=128)
    ap.add_argument("--ready-timeout", type=int, default=420)
    ap.add_argument("--tag", default="")
    ap.add_argument("--extra", default="", help="extra server flags, e.g. --override-kv k=int:2048")
    args = ap.parse_args()

    model = Path(args.model)
    if not model.exists():
        print(f"FAIL: model not found: {model}")
        return 2
    exe = Path(args.exe)
    if not exe.exists():
        print(f"FAIL: server binary not found: {exe}")
        return 2

    stamp = time.strftime("%Y%m%d_%H%M%S")
    tag = args.tag or model.stem[:40].replace(" ", "_")
    outdir = REPO / "scratch" / "model_tests"
    outdir.mkdir(parents=True, exist_ok=True)
    log = outdir / f"{stamp}_{tag}.log"
    record_path = outdir / f"{stamp}_{tag}.json"

    cmd = [str(exe), "--model", str(model), "--host", "127.0.0.1", "--port", str(args.port),
           "--n-gpu-layers", str(args.ngl), "--ctx-size", str(args.ctx), "--threads", "8"]
    if args.extra:
        cmd += args.extra.split()
    print(f"model : {model.name}  ({model.stat().st_size/1024**3:.2f} GB)")
    print(f"exe   : {exe}")
    print(f"flags : --n-gpu-layers {args.ngl} --ctx-size {args.ctx} --threads 8  (port {args.port})")
    print(f"gpu   : {gpu_dedicated_gb()} GB dedicated in use before load")

    t0 = time.time()
    with open(log, "wb") as lf:
        proc = subprocess.Popen(cmd, stdout=lf, stderr=subprocess.STDOUT,
                                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    up = False
    load_s = None
    FATAL = ("error loading model", "failed to load model", "unknown model architecture",
             "GGML_ASSERT", "exiting due to model loading error")
    while time.time() - t0 < args.ready_timeout:
        if proc.poll() is not None:
            break
        # A loader that rejects the file prints a fatal line and may then sit there. Waiting the
        # full timeout on an already-decided failure is 7 minutes of nothing; measured twice.
        try:
            seen = log.read_text(errors="replace")
            if any(f in seen for f in FATAL):
                break
        except OSError:
            pass
        try:
            http_json(f"http://127.0.0.1:{args.port}/health", timeout=3)
            up = True
            load_s = round(time.time() - t0, 1)
            break
        except Exception:
            time.sleep(1.0)

    if not up:
        full = log.read_text(errors="replace")
        tail = "\n".join(full.splitlines()[-12:])
        print(f"FAIL: never answered /health in {args.ready_timeout}s (exit {proc.poll()})")
        print("--- server log tail ---\n" + tail)
        if proc.poll() is None:
            proc.terminate()
        # A refusal is a result. Write it down with the loader's own words, otherwise the
        # only evidence that a model cannot run is the absence of a file.
        rec = {"model": model.name, "model_gb": round(model.stat().st_size / 1024 ** 3, 2),
               "exe": str(exe), "ngl": args.ngl, "ctx": args.ctx, "port": args.port,
               "loaded": False, "exit_code": proc.poll(),
               "fatal_lines": [l for l in full.splitlines()
                               if any(k in l for k in ("error loading", "failed to load", "missing tensor",
                                                       "GGML_ASSERT", "unknown model"))][:6],
               "log": str(log)}
        record_path.write_text(json.dumps(rec, indent=2), encoding="utf-8")
        print(f"record: {record_path.name}")
        return 1

    props = http_json(f"http://127.0.0.1:{args.port}/props", timeout=15)
    ws_mb, priv_mb = proc_mem_mb(proc.pid)
    gpu_after = gpu_dedicated_gb()
    print(f"loaded: {load_s}s | commit {priv_mb:,} MB | WS {ws_mb:,} MB | gpu {gpu_after} GB "
          f"(+{round(gpu_after - (gpu_after - 0),2)})")

    results = []
    for name, prompt in PROMPTS:
        body = {"messages": [{"role": "user", "content": prompt}], "temperature": 0,
                "max_tokens": args.max_tokens, "stream": False}
        t = time.time()
        try:
            r = http_json(f"http://127.0.0.1:{args.port}/v1/chat/completions", body)
            el = time.time() - t
            usage = r.get("usage", {}) or {}
            msg = (r.get("choices") or [{}])[0].get("message", {}) or {}
            txt = msg.get("content") or ""
            # Reasoning models put the visible answer in `content` and the trace elsewhere; several
            # templates leave `content` empty and everything in reasoning_content. Measured on
            # Qwen3.8-27B: three prompts returned 128 tokens each and an empty string.
            reasoning = msg.get("reasoning_content") or msg.get("reasoning") or ""
            if not txt.strip() and reasoning:
                txt = "[REASONING ONLY] " + reasoning
            ct = usage.get("completion_tokens")
            tps = round(ct / el, 2) if ct else None
            results.append({"prompt": name, "seconds": round(el, 2), "completion_tokens": ct,
                            "tok_per_s": tps, "answer": txt.strip()[:400],
                            "reasoning_present": bool(reasoning)})
            print(f"  {name:<7} {el:6.2f}s  {str(ct):>4} tok  {tps if tps else '-'} tok/s  | {txt.strip()[:60]!r}")
        except Exception as e:
            results.append({"prompt": name, "error": f"{type(e).__name__}: {e}"})
            print(f"  {name:<7} ERROR {type(e).__name__}")

    ws_mb2, priv_mb2 = proc_mem_mb(proc.pid)
    gpu_peak = gpu_dedicated_gb()

    proc.terminate()
    try:
        proc.wait(timeout=30)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=30)
    time.sleep(2)
    dead = True
    try:
        http_json(f"http://127.0.0.1:{args.port}/health", timeout=3)
        dead = False
    except Exception:
        pass

    rec = {
        "model": model.name, "model_gb": round(model.stat().st_size / 1024 ** 3, 2),
        "exe": str(exe), "ngl": args.ngl, "ctx": args.ctx, "port": args.port,
        "load_seconds": load_s, "commit_mb_loaded": priv_mb, "ws_mb_loaded": ws_mb,
        "commit_mb_after_prompts": priv_mb2, "gpu_dedicated_gb_peak": gpu_peak,
        "props_model_path": props.get("model_path"), "n_ctx": (props.get("default_generation_settings") or {}).get("n_ctx"),
        "results": results, "port_dead_after_unload": dead, "log": str(log),
    }
    record_path.write_text(json.dumps(rec, indent=2), encoding="utf-8")
    print(f"unloaded: port {args.port} {'dead' if dead else 'STILL ANSWERING'} | record {record_path.name}")
    return 0 if dead else 2


if __name__ == "__main__":
    raise SystemExit(main())
