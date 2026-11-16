#!/usr/bin/env python3
"""platform_stop.py — stop the platform and PROVE it is down.

WHY THIS EXISTS (measured 2026-11-13): the batch script decided "stopped" from `netstat -ano` rows. On this host
those rows do not exist for the platform's ports at all — `netstat -ano | grep :8593` and `:3000` return nothing
while both servers answer HTTP, and Get-NetTCPConnection shows only TimeWait/FinWait1 entries owned by pid 0.
So STOP_TRM_PLATFORM.bat printed "[DOWN] ... port free" for a live server, printed "[STOPPED] Jarvis (pid 20424)"
after killing a wrapper, and signed off with "TRM platform stopped (council verified down)" while the embed
server and Jarvis were still serving. Nothing about a reboot should rest on that.

A stop is verified by asking the port again, never by a pid that vanished or a socket table that cannot see it.

Order of handles, per service:
    1. the pidfile the launcher wrote            (exact, preferred)
    2. the command line that names the port       (Win32_Process CommandLine regex)
    3. report STILL UP and exit non-zero          (never claim success)
"""
from __future__ import annotations

import argparse
import json
import subprocess
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RUN = ROOT / "run"

# name, port, probe path, expected marker (None = any 200 counts), pidfile, command-line regex
SERVICES: list[tuple[str, int, str, str | None, str | None, str]] = [
    ("Bonsai2-27B PTQ1_0", 8612, "/props", None, "model_8612.pid", r"--port 8612\b"),
    ("CPU ternary pool", 8620, "/props", None, "cpu_pool.pid", r"--port 8620\b"),
    ("GPU 8-slot pool", 8730, "/props", None, "gpu_pool.pid", r"--port 8730\b"),
    ("GPU 14-slot experiment", 8740, "/props", None, "gpu_pool14.pid", r"--port 8740\b"),
    ("Embeddings", 8593, "/health", "ok", None, r"--port 8593\b"),
    ("Inference Engine", 8587, "/health", "ok", None, r"--port 8587\b"),
    ("Deep Tier", 8591, "/health", "ok", None, r"--port 8591\b"),
    ("Qwen3-4B", 8595, "/health", "ok", None, r"--port 8595\b"),
    ("Instella-MoE", 8596, "/health", "ok", None, r"--port 8596\b"),
    ("Jarvis", 3000, "/health", None, None, r"Jarvis-Advanced-main"),
    ("Companion Bridge", 3002, "/health", None, None, r"trm_companion_bridge"),
    ("Companion Proxy", 3001, "/health", None, None, r"trm_companion_proxy"),
    ("LCARS Dashboard", 5183, "/", None, None, r"LCARS|vite"),
    ("Kernel UI", 5184, "/", None, None, r"5184"),
]

COUNCIL_PORT = 8022

# the image that owns each port; a command-line match without this is not the owner
NAMES: dict[int, tuple[str, ...]] = {
    8612: ("llama-server.exe",), 8620: ("llama-server.exe",), 8730: ("llama-server.exe",),
    8740: ("llama-server.exe",), 8593: ("llama-server.exe",), 8587: ("llama-server.exe",),
    8591: ("llama-server.exe",), 8595: ("llama-server.exe",), 8596: ("llama-server.exe",),
    3000: ("node.exe", "cmd.exe"),
    3001: ("python.exe", "python3.exe"), 3002: ("trm_companion_bridge.exe",),
    5183: ("node.exe",), 5184: ("node.exe",),
}


def alive(port: int, path: str = "/props", marker: str | None = None, timeout: float = 3.0) -> bool:
    try:
        raw = urllib.request.urlopen(f"http://127.0.0.1:{port}{path}", timeout=timeout).read().decode("utf-8", "replace")
    except Exception:
        return False
    return marker is None or marker in raw


def ps(cmd: str) -> str:
    r = subprocess.run(["powershell", "-NoProfile", "-Command", cmd], capture_output=True, text=True, timeout=180)
    return (r.stdout or "").strip()


def kill(pid: int) -> bool:
    try:
        ps(f"Stop-Process -Id {pid} -Force -ErrorAction SilentlyContinue")
        return True
    except Exception:
        return False


def pids_by_cmdline(pattern: str, names: tuple[str, ...] = ()) -> list[int]:
    """Match the command line, optionally restricted to process names.

    MEASURED DEFECT: without the name filter this matched and killed bystanders — a bash.exe whose working
    directory happened to contain "Jarvis-Advanced-main", and shells whose command line mentioned a port
    number. The port's owner is the process whose IMAGE is the server's, so the name is required.
    """
    guard = ""
    if names:
        alt = "|".join(n.replace("'", "''") for n in names)
        guard = " -and $_.Name -match '^(%s)$'" % alt
    cmd = ("Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -match '%s'%s } | "
           "Select-Object -ExpandProperty ProcessId" % (pattern.replace("'", "''"), guard))
    out = ps(cmd)
    out = out if out and "No such" not in out else ""
    return [int(x) for x in out.split() if x.strip().isdigit()]


def stop_council() -> str:
    if not alive(COUNCIL_PORT, "/health", "Native Rust Sovereign Kernel"):
        return "already down"
    try:
        urllib.request.urlopen(urllib.request.Request(
            f"http://127.0.0.1:{COUNCIL_PORT}/platform/shutdown",
            data=json.dumps({"confirm": "stop"}).encode(),
            headers={"Content-Type": "application/json"}), timeout=10).read()
    except Exception:
        pass
    for _ in range(20):
        if not alive(COUNCIL_PORT, "/health", "Native Rust Sovereign Kernel"):
            return "self-terminated"
        time.sleep(1)
    pf = RUN / "council.pid"
    if pf.exists() and pf.read_text().strip().isdigit():
        kill(int(pf.read_text().strip()))
    for _ in range(10):
        if not alive(COUNCIL_PORT, "/health", "Native Rust Sovereign Kernel"):
            return "stopped by recorded pid"
        time.sleep(1)
    return "STILL UP"


def stop_service(name: str, port: int, probe: str, marker: str | None,
                 pidfile: str | None, regex: str, names: tuple[str, ...] = ()) -> dict:
    if not alive(port, probe, marker):
        return {"name": name, "port": port, "result": "already down", "via": "-"}
    via = None
    if pidfile:
        pf = RUN / pidfile
        if pf.exists() and pf.read_text().strip().isdigit():
            kill(int(pf.read_text().strip()))
            via = f"pidfile {pidfile}"
            time.sleep(2)
    if alive(port, probe, marker):
        pids = pids_by_cmdline(regex, names)
        if pids:
            for pid in pids:
                kill(pid)
            via = (via + " + " if via else "") + f"cmdline {regex} -> {','.join(map(str, pids))}"
            time.sleep(2)
    ok = not alive(port, probe, marker)
    return {"name": name, "port": port, "result": "stopped" if ok else "STILL UP", "via": via or "no handle"}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--keep", default="", help="comma ports to leave running")
    ap.add_argument("--json", default="")
    a = ap.parse_args()
    keep = {int(x) for x in a.keep.split(",") if x.strip().isdigit()}

    print("=" * 70)
    print("  TRM SWARM OS — PLATFORM SHUTDOWN (verified over HTTP)")
    print("=" * 70)
    council = stop_council()
    print(f" [{'OK ' if council != 'STILL UP' else 'FAIL'}]  Council Server :{COUNCIL_PORT} — {council}")

    rows = []
    for name, port, probe, marker, pidfile, regex in SERVICES:
        if port in keep:
            rows.append({"name": name, "port": port, "result": "kept", "via": "--keep"})
            continue
        r = stop_service(name, port, probe, marker, pidfile, regex, NAMES.get(port, ()))
        rows.append(r)
        tag = {"already down": "DOWN", "stopped": "STOPPED", "kept": "KEPT"}.get(r["result"], "FAIL")
        print(f" [{tag:<7}] {name:<24} :{port} — {r['via']}")

    # final authority: ask every port again
    still = [(n, p) for n, p, probe, marker, _, _ in SERVICES if p not in keep and alive(p, probe, marker)]
    if alive(COUNCIL_PORT, "/health", "Native Rust Sovereign Kernel"):
        still.append(("Council Server", COUNCIL_PORT))
    print()
    if still:
        print("NOT STOPPED: " + ", ".join(f"{n}:{p}" for n, p in still))
        rc = 1
    else:
        print("TRM platform stopped — every port re-probed and silent.")
        rc = 0
    rec = {"ts": time.strftime("%Y-%m-%dT%H:%M:%S"), "council": council, "services": rows, "still_up": still}
    out = Path(a.json) if a.json else ROOT / "scratch" / "stop" / f"{time.strftime('%Y%m%d_%H%M%S')}_stop.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(rec, indent=1), encoding="utf-8")
    print("record:", out)
    return rc


if __name__ == "__main__":
    raise SystemExit(main())
