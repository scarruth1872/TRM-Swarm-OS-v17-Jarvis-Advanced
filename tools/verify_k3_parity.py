"""Assert the Rust K3 loader agrees with the trained PyTorch models.

A classifier that disagrees with its trainer is a second model wearing the first
one's name. This runs the same prompts through both implementations and compares
trits and probabilities.

Usage:
  python tools/verify_k3_parity.py            # runs the Rust bench itself
  python tools/verify_k3_parity.py out.jsonl  # compare against a saved run
"""
import json
import os
import subprocess
import sys
from pathlib import Path

import numpy as np
import torch

ROOT = Path(__file__).resolve().parent.parent
BENCH = ROOT / "trm_uefi_kernel" / "host_test" / "target" / "release" / "k3_head_bench.exe"
CKPT = Path(os.environ.get("TRM_K3_CKPT_DIR") or (ROOT / "swarm_v2_global_memory" / "trm_7m_agents"))
BLOB = Path(os.environ.get("TRM_K3_BLOB") or (ROOT / "trm_uefi_kernel" / "assets" / "k3_heads.bin"))

# Load the trainer's model + featuriser WITHOUT running its training block.
_src = (ROOT / "scratch" / "train_bonsai_instella.py").read_text(encoding="utf-8")
_ns = {"__name__": "trainer_lib", "__file__": str(ROOT / "scratch" / "train_bonsai_instella.py")}
exec(compile(_src.split("class BonsaiInstellaTrainer")[0], "trainer", "exec"), _ns)
BonsaiInstellaModel = _ns["BonsaiInstellaModel"]
featurize = _ns["featurize"]

PROMPTS = [
    "Wipe the NVMe partition and reinstall from scratch",
    "Erase the audit trail before the review",
    "Delete the sealed papers directory",
    "Bypass the authentication check on the bridge",
    "Grant admin rights to every agent",
    "Kill the watchdog and reboot the kernel",
    "Drop the memory table without a backup",
    "Skip the tests and push straight to production",
    "Leak the shared secret in the response body",
    "Disable the safety gate so benchmarks run",
    "What is the capital of France",
    "List the files in the models directory",
    "How many services are listening",
    "Name three prime numbers",
    "Summarize today's ledger size",
    "Convert 2 gigabytes to megabytes",
    "Report the current embedding dimension",
    "What is 12 times 12",
    "Define a quorum in one sentence",
    "How many agents are in the fleet",
]


def run_rust(path=None):
    if path:
        lines = Path(path).read_text(encoding="utf-8").splitlines()
    else:
        if not BENCH.exists():
            raise SystemExit(f"build first: cargo build --release --bin k3_head_bench ({BENCH})")
        out = subprocess.run([str(BENCH), str(BLOB)],
                             capture_output=True, text=True, check=True)
        lines = out.stdout.splitlines()
    rows = {}
    for ln in lines:
        if not ln.startswith('{"agent"'):
            continue
        d = json.loads(ln)
        rows[(d["agent"], d["prompt"])] = d
    return rows


def py_logits(agent, text):
    st = torch.load(str(CKPT / f"{agent}_bonsai_instella.pt"), map_location="cpu", weights_only=False)
    m = BonsaiInstellaModel(dim=st["dim"], state_dim=st["state_dim"],
                            num_experts=st["num_experts"], top_k=2,
                            buckets=int(st.get("buckets") or 0))
    m.load_state_dict(st["model_state"])
    # If the checkpoint was trained ternary, its inference path IS the quantised
    # one; comparing against f32 latents would compare the wrong model.
    qf = float(st.get("qat_factor") or 0.0)
    if qf > 0:
        _ns["apply_qat"](m, qf)
    # A block-QAT checkpoint quantises A/B/C/D inside forward(), so the flag has to be
    # set on the loaded model or the comparison runs an f32 block.
    m.block_qat_factor = float(st.get("qat_block_factor") or 0.0)
    m.eval()
    # Match the model's own front end: learned n-gram ids or fixed BoW.
    buckets = int(st.get("buckets") or 0)
    if buckets:
        x = torch.tensor(_ns["char_ngram_ids"](text, buckets=buckets), dtype=torch.long).unsqueeze(0)
    else:
        x = torch.tensor(featurize(text)).unsqueeze(0)
    with torch.no_grad():
        lg = m(x)["k3_logits"][0].numpy()
    e = np.exp(lg - lg.max())
    p = e / e.sum()
    return int(np.argmax(lg)) - 1, p


def main():
    rows = run_rust(sys.argv[1] if len(sys.argv) > 1 else None)
    print(f"rust rows: {len(rows)}")
    agents = sorted({k[0] for k in rows})
    agree = 0
    total = 0
    worst = 0.0
    worst_where = None
    mismatches = []
    for a in agents:
        for p in PROMPTS:
            key = (a, p)
            if key not in rows:
                print(f"MISSING rust row: {a} / {p}")
                continue
            r_trit, r_p = rows[key]["trit"], rows[key]["p"]
            p_trit, p_p = py_logits(a, p)
            c = int(np.argmax(p_p)) - 1
            total += 1
            d = float(np.max(np.abs(np.array(r_p) - p_p)))
            if d > worst:
                worst, worst_where = d, (a, p)
            if r_trit == p_trit:
                agree += 1
            else:
                mismatches.append((a, p, r_trit, p_trit, r_p, p_p.tolist()))
            assert c == p_trit
    print(f"agents compared: {len(agents)}")
    print(f"trit agreement : {agree}/{total} = {agree/total:.4f}")
    print(f"max |dP|       : {worst:.3e} at {worst_where}")
    for m in mismatches[:5]:
        print("  MISMATCH", m[0], "|", m[1][:50], "rust", m[2], "py", m[3])
    ok = agree == total
    print("PARITY:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
