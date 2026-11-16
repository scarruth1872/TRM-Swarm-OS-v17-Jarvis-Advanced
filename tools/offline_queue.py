#!/usr/bin/env python3
"""offline_queue.py — let the cores work without ever gating a latency batch.

WHY (G104): four CPU items turned a 7.5 s batch into 33.7 s because the batch waited on them. Pinning did not
fix it, a better ubatch did not fix it, and a cost model that excludes them entirely threw the capacity away.
The only arrangement that keeps both is a QUEUE: latency-critical work goes to the GPU slots and returns now;
everything the cores can do is written down and drained afterwards, at the cores' own pace.

Files (append-only, one JSON object per line):
    <queue>                 jobs: {id, ts, name, payload, status:"pending"}
    <queue>.results.jsonl   results: {id, ts, name, verdict, tokens, seconds, port}

USAGE
  python tools/offline_queue.py submit --queue q.jsonl --name mod.rs --text "..." [--text-file path]
  python tools/offline_queue.py drain  --queue q.jsonl --port 8620 --workers 4 [--budget-seconds 600]
  python tools/offline_queue.py status --queue q.jsonl
"""
from __future__ import annotations

import argparse
import json
import threading
import time
import urllib.request
import uuid
from pathlib import Path

PROMPT = ("Audit this source file of a systems platform. Answer in ONE line: what the module does, then at most "
          "one risk (silent failure / fake telemetry / unbounded work). File: {name}\n---\n{body}")


def append(path: Path, row: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "a", encoding="utf-8") as fh:
        fh.write(json.dumps(row, ensure_ascii=False) + "\n")


def load(path: Path) -> list[dict]:
    if not path.exists():
        return []
    out = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.strip()
        if line:
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                pass
    return out


def submit(queue: Path, name: str, text: str) -> str:
    jid = uuid.uuid4().hex[:12]
    append(queue, {"id": jid, "ts": round(time.time(), 3), "name": name,
                   "payload": text[:1200], "status": "pending"})
    return jid


def drain(queue: Path, port: int, workers: int, budget_seconds: float) -> dict:
    jobs = [j for j in load(queue) if j.get("status") == "pending"]
    results_path = Path(str(queue) + ".results.jsonl")
    done = {r["id"] for r in load(results_path)}
    jobs = [j for j in jobs if j["id"] not in done]
    if not jobs:
        print("queue empty")
        return {"drained": 0}
    print(f"draining {len(jobs)} jobs on :{port} with {workers} workers")
    t0 = time.time()
    lock = threading.Lock()
    stats = {"ok": 0, "err": 0, "tokens": 0, "seconds": 0.0}

    def work(job: dict) -> None:
        body = {"messages": [{"role": "user", "content": PROMPT.format(name=job["name"], body=job["payload"])}],
                "temperature": 0.1, "max_tokens": 120, "stream": False,
                "chat_template_kwargs": {"enable_thinking": False}}
        ts = time.time()
        try:
            r = json.loads(urllib.request.urlopen(urllib.request.Request(
                f"http://127.0.0.1:{port}/v1/chat/completions", data=json.dumps(body).encode(),
                headers={"Content-Type": "application/json"}), timeout=900).read())
            msg = (r.get("choices") or [{}])[0].get("message", {}) or {}
            txt = (msg.get("content") or msg.get("reasoning_content") or "").strip()
            tok = (r.get("usage", {}) or {}).get("completion_tokens") or 0
            append(results_path, {"id": job["id"], "ts": round(time.time(), 3), "name": job["name"],
                                  "verdict": txt[:300], "tokens": tok, "seconds": round(time.time() - ts, 2),
                                  "port": port, "status": "done"})
            with lock:
                stats["ok"] += 1
                stats["tokens"] += tok
                stats["seconds"] += time.time() - ts
        except Exception as e:
            append(results_path, {"id": job["id"], "ts": round(time.time(), 3), "name": job["name"],
                                  "error": type(e).__name__, "port": port, "status": "error"})
            with lock:
                stats["err"] += 1

    from concurrent.futures import ThreadPoolExecutor
    with ThreadPoolExecutor(max_workers=workers) as ex:
        list(ex.map(work, jobs))
    wall = time.time() - t0
    rec = {"drained": len(jobs), "wall_seconds": round(wall, 2), "jobs_per_s": round(len(jobs) / wall, 2) if wall else 0,
           "ok": stats["ok"], "errors": stats["err"], "tokens": stats["tokens"],
           "tokens_per_s": round(stats["tokens"] / stats["seconds"], 2) if stats["seconds"] else None}
    print(json.dumps(rec, indent=1))
    return rec


def status(queue: Path) -> dict:
    jobs = load(queue)
    results = load(Path(str(queue) + ".results.jsonl"))
    pending = [j for j in jobs if j.get("status") == "pending"]
    done_ids = {r["id"] for r in results if r.get("status") == "done"}
    rec = {"submitted": len(jobs), "results": len(results), "pending": len(pending),
           "done": len(done_ids), "errors": len([r for r in results if r.get("status") == "error"]),
           "queue": str(queue)}
    print(json.dumps(rec, indent=1))
    return rec


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["submit", "drain", "status"])
    ap.add_argument("--queue", required=True)
    ap.add_argument("--name", default="")
    ap.add_argument("--text", default="")
    ap.add_argument("--text-file", default="")
    ap.add_argument("--port", type=int, default=8620)
    ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--budget-seconds", type=float, default=900)
    a = ap.parse_args()
    q = Path(a.queue)
    if a.cmd == "submit":
        text = Path(a.text_file).read_text(encoding="utf-8", errors="replace") if a.text_file else a.text
        print("queued", submit(q, a.name or a.text_file, text))
    elif a.cmd == "drain":
        drain(q, a.port, a.workers, a.budget_seconds)
    else:
        status(q)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
