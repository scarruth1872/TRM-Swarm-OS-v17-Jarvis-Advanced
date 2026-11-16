"""K3 parity against the DEPLOYED binary: v20's /k3/classify vs the PyTorch trainer.

tools/verify_k3_parity.py compares the trainer to trm_uefi_kernel/host_test's k3_head_bench.exe,
which exercises the 19.x kernel library. That says nothing about the v20 process now serving
:8022, whose head tier was ported into the council crate. This script asks the live endpoint the
same questions and compares trit + probabilities against the checkpoints that trained the heads.

Usage: python scratch/verify_k3_parity_http.py [max_prompts_per_agent]
"""
import importlib.util
import json
import sys
import urllib.request
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent
BASE = "http://127.0.0.1:8022"

spec = importlib.util.spec_from_file_location("vk3", ROOT / "tools" / "verify_k3_parity.py")
vk3 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(vk3)          # trainer model + featuriser + PROMPTS

N = int(sys.argv[1]) if len(sys.argv) > 1 else 3
PROMPTS = vk3.PROMPTS[:N]
LABELS = {-1: "block", 0: "analyze", 1: "proceed"}


def http_heads():
    with urllib.request.urlopen(f"{BASE}/k3/heads", timeout=20) as r:
        d = json.loads(r.read())
    if not d.get("loaded"):
        raise SystemExit(f"heads not loaded: {d.get('error')}")
    return [h["agent"] for h in d["heads"]]


def http_classify(agent, text):
    body = json.dumps({"agent": agent, "text": text}).encode()
    req = urllib.request.Request(f"{BASE}/k3/classify", data=body,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.loads(r.read())


def main():
    agents = http_heads()
    print(f"agents: {len(agents)} | prompts/agent: {len(PROMPTS)} | cases: {len(agents)*len(PROMPTS)}")
    total = agree = 0
    max_dp = 0.0
    worst = None
    for agent in agents:
        for text in PROMPTS:
            rust = http_classify(agent, text)
            if "trit" not in rust:
                print(f"  !! {agent}: {rust}")
                continue
            r_trit = int(rust["trit"])
            r_p = rust["probs"]
            p_trit, p_vec = vk3.py_logits(agent, text)
            p = {LABELS[i - 1]: float(p_vec[i]) for i in range(3)}
            dp = max(abs(r_p[k] - p[k]) for k in ("block", "analyze", "proceed"))
            total += 1
            same = (r_trit == p_trit)
            agree += int(same)
            if dp > max_dp:
                max_dp, worst = dp, (agent, text[:40], r_trit, p_trit, dp)
            if not same:
                print(f"  MISMATCH {agent:10s} rust {r_trit:+d} torch {p_trit:+d} | {text[:44]}")
    print(f"\nparity: {agree}/{total} trits agree | max |dP| {max_dp:.3e}")
    if worst:
        print(f"worst: {worst[0]} '{worst[1]}' rust {worst[2]:+d} torch {worst[3]:+d} dP {worst[4]:.3e}")
    return 0 if total and agree == total and max_dp < 1e-3 else 1


if __name__ == "__main__":
    sys.exit(main())
