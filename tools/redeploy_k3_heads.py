"""Re-export the K3 heads and bounce the council so the new blob is served.

Because the blob is loaded from disk (not embedded), a retrain needs no rebuild -
but the running process caches the heads in a OnceLock, so it does need a restart.
This does the whole pass and fails loudly at any step.

Usage: python tools/redeploy_k3_heads.py [--no-restart]
"""
import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

import psutil

ROOT = Path(__file__).resolve().parent.parent
PY = sys.executable
BLOB = ROOT / "trm_uefi_kernel" / "assets" / "k3_heads.bin"
MAN = ROOT / "trm_uefi_kernel" / "assets" / "k3_heads.json"
EXE = ROOT / "trm_uefi_kernel" / "host_council_server" / "target" / "release" / "host_council_server.exe"
CWD = ROOT / "trm_uefi_kernel" / "host_council_server"
LOG = ROOT / "logs" / "council8022_k3.log"


def stop_8022():
    killed = []
    for c in psutil.net_connections(kind="tcp"):
        if c.laddr and c.laddr.port == 8022 and c.status == "LISTEN":
            p = psutil.Process(c.pid)
            name = p.name()
            p.terminate()  # graceful; /F would pop an interactive dialog
            killed.append((c.pid, name))
    deadline = time.time() + 15
    while time.time() < deadline:
        still = [c.pid for c in psutil.net_connections(kind="tcp")
                 if c.laddr and c.laddr.port == 8022 and c.status == "LISTEN"]
        if not still:
            break
        time.sleep(0.5)
    return killed


def start_8022():
    LOG.parent.mkdir(parents=True, exist_ok=True)
    log = open(LOG, "ab")
    p = subprocess.Popen([str(EXE)], cwd=str(CWD), stdout=log, stderr=log,
                         creationflags=subprocess.DETACHED_PROCESS | subprocess.CREATE_NEW_PROCESS_GROUP)
    for i in range(60):
        time.sleep(1)
        try:
            import urllib.request
            urllib.request.urlopen("http://127.0.0.1:8022/health", timeout=2).read()
            return p.pid, i + 1
        except Exception:
            continue
    raise SystemExit(f"council did not come up on :8022 (pid {p.pid}, see {LOG})")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--no-restart", action="store_true")
    args = ap.parse_args()

    print("[1/3] exporting heads")
    r = subprocess.run([PY, str(ROOT / "tools" / "export_k3_heads.py")],
                       cwd=str(ROOT), capture_output=True, text=True)
    print(r.stdout.strip()[-600:])
    if r.returncode != 0:
        print(r.stderr[-1500:])
        raise SystemExit("export failed")

    man = json.loads(MAN.read_text(encoding="utf-8"))
    emb = [a["agent"] for a in man["agents"] if a.get("buckets")]
    print(f"[2/3] manifest: v{man['version']} {man['n_agents']} agents "
          f"{man['bytes'] / 1e6:.1f} MB, embedding tables: {len(emb)}")
    print(f"       sha256 {man['sha256'][:16]}...")

    if args.no_restart:
        print("[3/3] skipped restart (--no-restart)")
        return 0

    print("[3/3] restarting council")
    for pid, name in stop_8022():
        print(f"       stopped pid {pid} ({name})")
    pid, secs = start_8022()
    print(f"       started pid {pid}, healthy after {secs}s")

    import urllib.request
    body = json.loads(urllib.request.urlopen("http://127.0.0.1:8022/k3/heads", timeout=20).read())
    print(f"       /k3/heads count={body['count']} loaded={body['loaded']} error={body.get('error')}")
    if not body.get("loaded") or body["count"] != man["n_agents"]:
        raise SystemExit("served heads disagree with the manifest")
    print("OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
