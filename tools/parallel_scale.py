#!/usr/bin/env python3
"""parallel_scale.py — how many small-model servers can this box actually run, and does the
throughput add up when CPU and GPU instances are PINNED?

WHY: the 584-slot design implies many concurrent "agents". The honest question is not whether we
can start twelve servers (we can) but whether twelve produce more work per second than one, and
where each one should live. So this measures:

  1. one instance alone on the GPU  -> the reference rate
  2. N instances split GPU/CPU, each pinned to distinct logical cores,
     one request each, fired together -> aggregate tokens/second and the per-instance spread

PINNING is real: CPU instances are started with --n-gpu-layers 0 and given an explicit
SetProcessAffinityMask (six logical cores each) so the scheduler cannot migrate them; GPU instances
take --n-gpu-layers 99 and share one device by construction.

USAGE
  python tools/parallel_scale.py --n-gpu 8 --n-cpu 4 --tokens 96
"""
from __future__ import annotations

import argparse
import ctypes
import json
import subprocess
import sys
import threading
import time
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
EXE = r"F:\Development sites\llama-vulkan-b12360\llama-server.exe"
MODEL = r"E:\Hugginface models\Ternary-Bonsai-2-2B-Q2_0.gguf"
PROMPT = "Name the three laws of thermodynamics in one short list."
LOGICAL = 24

def cpu_masks(n: int, cores: int = LOGICAL) -> list[int]:
    """Carve the logical CPUs into n disjoint masks, so no two CPU instances can share a core."""
    width = cores // n
    return [((1 << width) - 1) << (i * width) for i in range(n)]


CPU_AFFINITY = cpu_masks(4)   # default split before --n-cpu is known


def gpu_gb() -> float:
    r = subprocess.run(["powershell", "-NoProfile", "-Command",
        r"(Get-Counter '\GPU Adapter Memory(*)\Dedicated Usage' -ErrorAction SilentlyContinue)."
        r"CounterSamples | Where-Object {$_.CookedValue -gt 50MB} | "
        r"Measure-Object -Property CookedValue -Maximum | Select-Object -ExpandProperty Maximum"],
        capture_output=True, text=True, timeout=90)
    try: return round(float(r.stdout.strip()) / 1024 ** 3, 2)
    except ValueError: return -1.0


def cpu_pct() -> float:
    r = subprocess.run(["powershell", "-NoProfile", "-Command",
        "(Get-Counter '\\Processor Information(_Total)\\% Processor Utility').CounterSamples[0].CookedValue"],
        capture_output=True, text=True, timeout=60)
    try: return round(float(r.stdout.strip()), 1)
    except ValueError: return -1.0


def pin(proc: subprocess.Popen, mask: int) -> bool:
    """SetProcessAffinityMask on the live child. Real pinning, not a request to the OS."""
    try:
        h = ctypes.windll.kernel32.OpenProcess(0x0200 | 0x0400, False, proc.pid)  # SET_INFORMATION|QUERY
        ok = ctypes.windll.kernel32.SetProcessAffinityMask(h, ctypes.c_size_t(mask))
        ctypes.windll.kernel32.CloseHandle(h)
        return bool(ok)
    except Exception:
        return False


def start(port: int, gpu_layers: int, threads: int, outdir: Path, tag: str) -> subprocess.Popen:
    log = open(outdir / f"{tag}_{port}.log", "wb")
    cmd = [EXE, "--model", MODEL, "--host", "127.0.0.1", "--port", str(port),
           "--n-gpu-layers", str(gpu_layers), "--ctx-size", "1024", "--threads", str(threads),
           "--parallel", "1", "-fa", "on"]
    if gpu_layers:
        cmd += ["-ub", "128", "-b", "256"]
    return subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT,
                            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))


def wait_health(port: int, timeout: float = 180.0) -> bool:
    t0 = time.time()
    while time.time() - t0 < timeout:
        try:
            urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=3).read()
            return True
        except Exception:
            time.sleep(0.5)
    return False


def one_request(port: int, tokens: int, results: dict, key: str) -> None:
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
        results[key] = {"port": port, "seconds": round(el, 2), "tokens": ct,
                        "tok_s": round(ct / el, 2) if el else 0}
    except Exception as e:
        results[key] = {"port": port, "error": f"{type(e).__name__}"}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--n-gpu", type=int, default=8)
    ap.add_argument("--n-cpu", type=int, default=4)
    ap.add_argument("--tokens", type=int, default=96)
    ap.add_argument("--base-port", type=int, default=8700)
    ap.add_argument("--tag", default="p12")
    args = ap.parse_args()

    outdir = REPO / "scratch" / "parallel"; outdir.mkdir(parents=True, exist_ok=True)
    stamp = time.strftime("%Y%m%d_%H%M%S")
    procs: list[subprocess.Popen] = []
    report = {"tag": f"{stamp}_{args.tag}", "single": None, "parallel": None}

    # ---- 1. reference: ONE instance alone on the GPU -------------------------------------
    print(f"reference: one instance, GPU, {args.tokens} tokens | gpu {gpu_gb()} GB")
    p = start(args.base_port, 99, 4, outdir, f"{stamp}_single")
    procs.append(p)
    if not wait_health(args.base_port):
        print("FAIL: reference server did not come up")
        for q in procs: q.terminate()
        return 1
    r: dict = {}
    one_request(args.base_port, args.tokens, r, "single")
    report["single"] = r.get("single")
    print(f"  single: {report['single']}")
    p.terminate(); p.wait(timeout=30); procs.clear()
    time.sleep(2)

    # ---- 2. N instances, pinned -----------------------------------------------------------------
    global CPU_AFFINITY
    CPU_AFFINITY = cpu_masks(max(1, args.n_cpu))
    total = args.n_gpu + args.n_cpu
    print(f"\nstarting {total} instances: {args.n_gpu} GPU + {args.n_cpu} CPU pinned "
          f"{[hex(m) for m in CPU_AFFINITY[:args.n_cpu]]}")
    ports, affinities = [], []
    t_load = time.time()
    for i in range(args.n_gpu):
        port = args.base_port + 10 + i
        procs.append(start(port, 99, 2, outdir, f"{stamp}_gpu"))
        ports.append((port, "gpu")); affinities.append(None)
    for i in range(args.n_cpu):
        port = args.base_port + 10 + args.n_gpu + i
        p = start(port, 0, max(1, LOGICAL // max(1, args.n_cpu)), outdir, f"{stamp}_cpu")
        procs.append(p)
        ports.append((port, "cpu")); affinities.append(CPU_AFFINITY[i % len(CPU_AFFINITY)])

    pinned = 0
    # CPU instances are the last n_cpu entries of procs, in the same order as CPU_AFFINITY
    for j in range(args.n_cpu):
        idx = args.n_gpu + j
        ok = pin(procs[idx], CPU_AFFINITY[j % len(CPU_AFFINITY)])
        pinned += 1 if ok else 0
    print(f"  affinity applied to {pinned}/{args.n_cpu} CPU instances")

    # MEASURED DEFECT: a sequential wait_health() per port cost 12 x up to 180 s and got the whole run
    # killed before it measured anything. Wait for ALL of them concurrently against one deadline.
    health: dict[int, bool] = {}
    ths = [threading.Thread(target=lambda q: health.__setitem__(q, wait_health(q, 120.0)), args=(port,))
           for port, _ in ports]
    t_h = time.time()
    for th in ths: th.start()
    for th in ths: th.join()
    up = [health.get(port, False) for port, _ in ports]
    print(f"  health wait wall {round(time.time()-t_h,1)}s")
    print(f"  healthy: {sum(up)}/{total}  (load wall {round(time.time()-t_load,1)}s)  gpu {gpu_gb()} GB")

    before_cpu, before_gpu = cpu_pct(), gpu_gb()
    results: dict = {}
    threads = [threading.Thread(target=one_request, args=(port, args.tokens, results, f"{kind}{i}"))
               for i, (port, kind) in enumerate(ports) if up[i]]
    t0 = time.time()
    for th in threads: th.start()
    for th in threads: th.join()
    wall = time.time() - t0
    after_cpu, after_gpu = cpu_pct(), gpu_gb()

    ok = [v for v in results.values() if "tok_s" in v]
    agg_tok = sum(v["tokens"] for v in ok)
    report["parallel"] = {
        "instances_requested": total, "instances_healthy": sum(up),
        "cpu_pinned": pinned, "wall_seconds": round(wall, 2),
        "aggregate_tokens": agg_tok,
        "aggregate_tok_s": round(agg_tok / wall, 2) if wall else 0,
        "per_instance_mean_tok_s": round(sum(v["tok_s"] for v in ok) / len(ok), 2) if ok else None,
        "per_instance": ok, "errors": [v for v in results.values() if "error" in v],
        "cpu_pct_before": before_cpu, "cpu_pct_after": after_cpu,
        "gpu_gb_before": before_gpu, "gpu_gb_after": after_gpu,
    }

    print(f"\n=== {total} in parallel ===")
    for k, v in sorted(report["parallel"]["per_instance"].items() if isinstance(report["parallel"]["per_instance"], dict) else []):
        print(f"  {k:<8} port {v['port']} {v['tok_s']:>7} tok/s  {v['tokens']} tok in {v['seconds']}s")
    print(f"  aggregate {report['parallel']['aggregate_tok_s']} tok/s over {report['parallel']['wall_seconds']}s")
    print(f"  CPU {before_cpu}% -> {after_cpu}% | GPU {before_gpu} -> {after_gpu} GB")
    if report["single"]:
        sp = report["parallel"]["aggregate_tok_s"] / report["single"]["tok_s"] if report["single"]["tok_s"] else 0
        # MEASURED DEFECT: x0.0049 printed as "x0.00", which reads as missing data rather than as a
        # 200x collapse. Keep enough digits for a result this small to stay legible, and label it.
        report["scale_factor"] = round(sp, 4)
        verdict = "WORSE THAN SERIAL" if sp < 1 else "BETTER THAN SERIAL"
        print(f"  scale vs single instance: x{sp:.4f}  -> {verdict}")

    out = outdir / f"{report['tag']}_summary.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print("record:", out)

    for p in procs:
        try:
            p.terminate()
        except Exception:
            pass
    for p in procs:
        try:
            p.wait(timeout=20)
        except Exception:
            p.kill()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
