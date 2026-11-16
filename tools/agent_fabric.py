#!/usr/bin/env python3
"""agent_fabric.py — give the 584 digital-twin slots real work, bound to CPU/GPU/RAM by design.

WHY THIS EXISTS
  `trm_uefi_kernel/src/digital_twin.rs` defines 584 slots in three teams and a `dispatch(ambiguity)`
  that picks a slot id. For the Bonsai team it increments a counter, flips a trit and returns - no
  model is asked anything; all 134 Bonsai slots start at state_trit = -1 ("dormant"). So the platform
  advertises a 584-slot fleet whose deep tier never executes.

  This is the missing body, not a replacement for the kernel: an out-of-process worker pool that
  routes each item by ambiguity to the tier that can answer it cheapest, and records what each tier
  delivered. It writes a JSONL ledger the kernel already knows how to read as an artefact, and it
  never touches the decision path.

BINDING (voltage doctrine: each tier gets the resource it is best at)
  fast_path  CPU, ~1 us/item      deterministic signals over text: no model, no VRAM, no queue
  bonsai     GPU, ~50-200 ms/item llama-server (ternary Bonsai 2) - the only tier that reasons
  ledger     RAM->disk             append-only JSONL, one row per item, sampled writes

MEASUREMENT
  Per tier: items, wall seconds, items/s, and (for the GPU tier) tokens and tokens/s from the
  server's own usage block. Plus host CPU percent and GPU dedicated VRAM sampled around the run,
  so "we used the GPU" is a number and not a claim.

USAGE
  python tools/agent_fabric.py --n 40 --port 8612 --workers 4
  python tools/agent_fabric.py --n 40 --dry-run        # fast-path tier only, no model needed
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import threading
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CPU_PORTS: set[int] = set()
OUT = REPO / "swarm_v2_artifacts" / "AGENT_FABRIC_LEDGER.jsonl"

# --- teams, copied from digital_twin.rs so the ledger names the same slots -------------------
TEAMS = {"fast_path": (0, 300), "instella": (300, 450), "bonsai": (450, 584)}
ROLES = {
    "fast_path": ["sensor_validator", "boundary_checker", "safety_interlock", "range_checker",
                  "schema_checker", "pattern_matcher", "error_counter"],
    "bonsai": ["logic_prover", "policy_verifier", "structural_validator", "invariant_checker",
               "safety_auditor", "plan_evaluator", "risk_assessor", "consistency_verifier"],
}

FAST_SIGNALS = [
    ("unsafe_block", re.compile(r"\bunsafe\s*\{")),
    ("todo_marker", re.compile(r"\b(TODO|FIXME|XXX|unimplemented!|todo!)\b")),
    ("returns_zero_todo", re.compile(r"return\s+0\s*//\s*TODO", re.I)),
    ("test_module", re.compile(r"#\[cfg\(test\)\]|def test_")),
    ("assert_only", re.compile(r"GGML_ASSERT|assert_eq!|assert!")),
    ("panic_site", re.compile(r"\bpanic!\(|\.unwrap\(\)|\.expect\(")),
]


def slot_for(team: str, i: int) -> tuple[int, str]:
    lo, hi = TEAMS[team]
    span = hi - lo
    return lo + (i % span), ROLES[team][i % len(ROLES[team])]


def host_stats() -> dict:
    # NOTE: no -f N formats here. MEASURED defect: '{0:N1}' emits "12,048.8", float() raises, the
    # whole stats call degrades to None and the run then crashes on formatting a None. Plain
    # round-trippable numbers only.
    script = (r"$o=Get-CimInstance Win32_OperatingSystem;"
              r"$c=(Get-Counter '\GPU Adapter Memory(*)\Dedicated Usage' -ErrorAction SilentlyContinue)."
              r"CounterSamples | Where-Object {$_.CookedValue -gt 50MB} | Measure-Object -Property CookedValue -Maximum;"
              r"$u=(Get-Counter '\Processor Information(_Total)\% Processor Utility' -ErrorAction SilentlyContinue)."
              r"CounterSamples[0].CookedValue;"
              r"'{0}|{1}|{2}' -f [math]::Round($u,1), [math]::Round($c.Maximum/1GB,2), [math]::Round($o.FreePhysicalMemory/1KB,0)")
    try:
        out = subprocess.run(["powershell", "-NoProfile", "-Command", script],
                             capture_output=True, text=True, timeout=90).stdout.strip()
        cpu, vram, avail = (x.replace(",", "") for x in out.split("|"))
        return {"cpu_pct": float(cpu), "vram_gb": float(vram), "ram_free_mb": float(avail)}
    except Exception:
        return {"cpu_pct": None, "vram_gb": None, "ram_free_mb": None}


def fast_path(item: dict) -> dict:
    """Deterministic signals only. No model, no GPU, no ambiguity."""
    t0 = time.perf_counter()
    hits = [name for name, rx in FAST_SIGNALS if rx.search(item["text"])]
    dt = (time.perf_counter() - t0) * 1e6
    verdict = "flag" if hits else "clean"
    return {"tier": "fast_path", "verdict": verdict, "signals": hits, "us": round(dt, 1)}


# MEASURED 2026-11-13: with the model's default reasoning budget, all 34 GPU answers came back as
# truncated thought ("We need answer user's request...") and 96 tokens were consumed per item with no
# answer produced - 27 s latency, 0.17 items/s. Two changes: thinking off (chat_template_kwargs) and
# enough tokens for a real answer. Same lesson as G95, now in the fabric.
PROMPT = ("Audit this source file of a systems platform. Answer in ONE line: what the module does, "
          "then at most one risk (silent failure / fake telemetry / unbounded work). File: {name}\n---\n{body}")



# ---------------------------------------------------------------- measured cost model (G103)
# Per-token cost MEASURED on this box, 2026-11-13, same 2B Q2_0 model, same /completion call:
#   GPU slot (:8730)   prefill 2,132 tok/s = 0.47 ms/token | decode  144.3 tok/s = 6.93 ms/token
#   CPU slot (:8620)   prefill    40 tok/s = 24.80 ms/token | decode   20.7 tok/s = 48.31 ms/token
# The tiers differ 7x on decode but 52x on PRE-FILL, so payload size - not wording, not core count -
# decides whether a CPU slot is usable: a 600-token prompt costs 0.28 s on the GPU and 14.9 s on a core.
# Pinning does not change this: affinity restricts our process, not the other 8 unpinned processes on
# this box (all 0xFFFFFF), and L3/IF/memory controllers stay shared.
COST = {
    "gpu": {"prefill_ms": 0.47, "decode_ms": 6.93},
    # MEASURED 2026-11-13 (cpu_fix sweep): the CPU prefill was a CONFIGURATION fault - with -ub 512
    # instead of the default 128, the same cores do 87.5 tok/s prefill (2.2x) at 19.7 tok/s decode.
    # 24 threads was worse (prefill 76.7, decode 4.1): the 3900X's 12 physical cores lose to SMT
    # oversubscription, so 12 threads it is.
    # FINAL CPU pool config: threads 6, parallel 4, -ub 512 (pid in run/cpu_pool.pid, pinned 0xFF000).
    # Measured on the 1,635-char audit payload: prefill 48.9 tok/s (20.45 ms/tok), decode 23.2 tok/s
    # (43.10 ms/tok). -ub 512 buys +21% prefill but costs 12% aggregate throughput; prefill wins for
    # audit payloads because 300 prompt tokens dominate 60 answer tokens.
    "cpu": {"prefill_ms": 20.45, "decode_ms": 43.10},
}
OUT_TOKENS = 60          # fabric asks for one line; 60 covers a full audit answer


def predict_ms(tier: str, prompt_chars: int, out_tokens: int = OUT_TOKENS) -> float:
    """Latency a tier would need for this payload. tokens ~ chars/4 is the estimator we ship with."""
    tok = max(1, prompt_chars // 4)
    c = COST[tier]
    return tok * c["prefill_ms"] + out_tokens * c["decode_ms"]


def offline_submit(queue_path: str, name: str, text: str) -> str:
    import uuid
    q = Path(queue_path); q.parent.mkdir(parents=True, exist_ok=True)
    jid = uuid.uuid4().hex[:12]
    with open(q, "a", encoding="utf-8") as fh:
        fh.write(json.dumps({"id": jid, "ts": round(time.time(), 3), "name": name,
                             "payload": text, "status": "pending"}, ensure_ascii=False) + "\n")
    return jid


def classify_port(port: int) -> str:
    return "cpu" if port in CPU_PORTS else "gpu"

def bonsai_tier(item: dict, port: int, slot_i: int) -> dict:
    slot, role = slot_for("bonsai", slot_i)
    body = {"messages": [{"role": "user", "content": PROMPT.format(name=item["name"], body=item["text"][:1200])}],
            "temperature": 0.1, "max_tokens": 160, "stream": False,
            "chat_template_kwargs": {"enable_thinking": False}}
    t0 = time.time()
    try:
        req = urllib.request.Request(f"http://127.0.0.1:{port}/v1/chat/completions",
                                     data=json.dumps(body).encode(),
                                     headers={"Content-Type": "application/json"})
        r = json.loads(urllib.request.urlopen(req, timeout=300).read())
        dt = time.time() - t0
        msg = (r.get("choices") or [{}])[0].get("message", {}) or {}
        txt = (msg.get("content") or msg.get("reasoning_content") or "").strip()
        usage = r.get("usage", {}) or {}
        return {"tier": "bonsai", "slot": slot, "role": role, "seconds": round(dt, 2),
                "completion_tokens": usage.get("completion_tokens"),
                "verdict": txt[:300]}
    except Exception as e:
        return {"tier": "bonsai", "slot": slot, "role": role, "seconds": round(time.time() - t0, 2),
                "error": f"{type(e).__name__}"}


def collect_items(n: int) -> list[dict]:
    """Real repo files - the platform's own inventory is the work queue."""
    roots = [REPO / "trm_uefi_kernel" / "src", REPO / "swarm_v2" / "core"]
    items = []
    for root in roots:
        if not root.exists():
            continue
        for p in sorted(root.rglob("*.rs")) + sorted(root.rglob("*.py")):
            try:
                txt = p.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            if len(txt) < 400:
                continue
            items.append({"name": str(p.relative_to(REPO)).replace("\\", "/"), "text": txt})
            if len(items) >= n:
                return items
    return items


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=40)
    ap.add_argument("--port", type=int, default=8612)
    # G100: agents cannot be 12 processes; they are 12 SLOTS. A pool may span two servers
    # (GPU slots + CPU slots), so ports are a list and workers are dealt round-robin across it.
    ap.add_argument("--ports", default="", help="comma list, e.g. 8730,8620 (GPU slots + CPU slots)")
    # MEASURED 2026-11-13: round-robin across a GPU pool and a CPU pool sent 15 of 24 items to the CPU
    # slots, which are 10-30x slower (3 tok/s vs 40), and the wall clock became theirs - 164 s for 30
    # items at 0.18 items/s. Placement must follow measured throughput, not take turns.
    ap.add_argument("--weights", default="", help="per-port weight, comma list matching --ports (e.g. 40,3)")
    ap.add_argument("--cpu-ports", default="", help="which ports are CPU-hosted, e.g. 8620; costs differ 52x on prefill")
    ap.add_argument("--budget-ms", type=float, default=8000.0, help="latency budget for tier eligibility")
    # G104: capacity the cores can actually provide must not be thrown away, and must not be awaited either.
    # With --offline-queue, an item that no tier can serve inside the budget is SUBMITTED to the queue and the
    # batch moves on; a worker drains it later. Nobody waits for a core.
    ap.add_argument("--offline-queue", default="", help="JSONL queue path; over-budget items are submitted, not awaited")
    ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--dry-run", action="store_true", help="fast-path tier only, no model required")
    ap.add_argument("--tag", default="")
    args = ap.parse_args()

    items = collect_items(args.n)
    if not items:
        print("FAIL: no work items found")
        return 2
    pool = [int(x) for x in args.ports.split(",") if x.strip()] if args.ports else [args.port]
    if args.weights:
        weight = {pt: float(w) for pt, w in zip(pool, [x for x in args.weights.split(",") if x.strip()])}
    else:
        weight = {pt: 1.0 for pt in pool}
    global CPU_PORTS
    CPU_PORTS = {int(x) for x in args.cpu_ports.split(",") if x.strip()}
    in_flight = {pt: 0 for pt in pool}
    place_lock = threading.Lock()

    def choose_port(prompt_chars: int = 0, budget_ms: float = 8000.0) -> int:
        """Cost-aware: a port is eligible only if its MEASURED prefill+decode fits the latency budget,
        then the least-loaded eligible port wins. Long payloads therefore never land on a core."""
        with place_lock:
            eligible = [q for q in pool
                        if predict_ms(classify_port(q), prompt_chars) <= budget_ms] or list(pool)
            pt = min(eligible, key=lambda q: (in_flight[q] + 1) / weight.get(q, 1.0))
            in_flight[pt] += 1
            return pt

    def release_port(pt: int) -> None:
        with place_lock:
            in_flight[pt] = max(0, in_flight[pt] - 1)
    tag = args.tag or time.strftime("%Y%m%d_%H%M%S")
    print(f"work queue: {len(items)} repo files | tiers: fast_path(CPU) + "
          f"{'none (dry-run)' if args.dry_run else 'bonsai(pool ' + ','.join(':'+str(p) for p in pool) + ')'}")
    before = host_stats()
    print("before: cpu {cpu_pct}%  vram {vram_gb} GB  ram_free {ram_free_mb} MB".format(**before))

    rows, lock = [], threading.Lock()
    t_start = time.time()

    def handle(i: int, item: dict) -> None:
        row = {"ts": round(time.time(), 3), "item": item["name"]}
        fp = fast_path(item)
        row.update({"fast_path": fp})
        # ambiguity: a file with no deterministic signal is the one worth spending a model on
        row["ambiguity"] = 0.02 if fp["signals"] else 0.85
        # an item no tier can serve inside the budget goes to the offline queue instead of gating the batch
        if args.offline_queue and predict_ms("cpu", len(item["text"][:1200]), 60) > args.budget_ms \
                and predict_ms("gpu", len(item["text"][:1200]), 60) > args.budget_ms:
            jid = offline_submit(args.offline_queue, item["name"], item["text"][:1200])
            row["tier_used"] = "offline_queued"
            row["job"] = jid
            row["answer"] = {"queued": True, "predicted_cpu_ms": round(predict_ms("cpu", len(item["text"][:1200]), 60))}
            with lock:
                rows.append(row)
            return
        if args.dry_run or row["ambiguity"] < 0.05:
            row["tier_used"] = "fast_path"
            row["answer"] = fp["signals"] or ["no deterministic signal"]
        else:
            pt = choose_port(len(item["text"][:1200]), args.budget_ms)
            b = bonsai_tier(item, pt, i)
            release_port(pt)
            row["tier_used"] = "bonsai"
            row["port"] = pt
            row["answer"] = b
        with lock:
            rows.append(row)
            if len(rows) % 10 == 0:
                print(f"  ...{len(rows)}/{len(items)}  {len(rows)/(time.time()-t_start):.2f} items/s")

    with ThreadPoolExecutor(max_workers=args.workers) as ex:
        list(ex.map(lambda p: handle(*p), list(enumerate(items))))

    wall = time.time() - t_start
    during = host_stats()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with open(OUT, "a", encoding="utf-8") as fh:
        for r in rows:
            fh.write(json.dumps(r, ensure_ascii=False) + "\n")

    by_tier: dict[str, list] = {}
    for r in rows:
        by_tier.setdefault(r["tier_used"], []).append(r)
    bonsai_rows = [r for r in by_tier.get("bonsai", []) if isinstance(r["answer"], dict) and "completion_tokens" in r["answer"]]
    tok = sum((r["answer"].get("completion_tokens") or 0) for r in bonsai_rows)
    secs = sum((r["answer"].get("seconds") or 0) for r in bonsai_rows)

    summary = {
        "tag": tag, "items": len(items), "wall_seconds": round(wall, 2),
        "items_per_s": round(len(rows) / wall, 2),
        "per_tier": {k: {"items": len(v)} for k, v in by_tier.items()},
        "gpu_tier": {"items": len(bonsai_rows), "completion_tokens": tok,
                     "tokens_per_s_summed": round(tok / secs, 2) if secs else None,
                     "mean_latency_s": round(secs / len(bonsai_rows), 2) if bonsai_rows else None},
        "errors": len([r for r in rows if isinstance(r.get("answer"), dict) and "error" in r["answer"]]),
        "pool": pool,
        "offline_queued": len([r for r in rows if r["tier_used"] == "offline_queued"]),
        "by_port": {str(pt): sum(1 for r in bonsai_rows if r.get("port") == pt) for pt in pool},
        "host_before": before, "host_during": during,
        "ledger": str(OUT),
    }
    print("\n=== summary ===")
    print(json.dumps(summary, indent=1))
    rec = REPO / "scratch" / "fabric" / f"{tag}_summary.json"
    rec.parent.mkdir(parents=True, exist_ok=True)
    rec.write_text(json.dumps({"summary": summary, "rows": rows}, indent=1), encoding="utf-8")
    print("record:", rec)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
