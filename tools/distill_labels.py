#!/usr/bin/env python3
"""Harvest teacher labels for distillation — targeting the student's actual failures.

Why the selection matters: the 1.5B scores 100 % structure and 59.0 % STRICT grounding on these tasks
(G124). Labelling 2,000 random tasks spends most of the teacher budget re-teaching something the student already
does. So this harvests in two pools:

  * FAILURES  — every task where the student's own recorded run failed strict grounding (the deficit)
  * PASSES    — an equal or smaller random sample, so the adapter does not learn "always cite" behaviour

Every teacher answer is filtered by the SAME strict checker the evaluation uses (schema valid + identifiers >= 3 chars
+ those identifiers literally present in the excerpt). Only survivors are written. A discarded answer is counted with
its reason, so the harvest's yield is itself a measurement.

Budget is expressed in labels, and the tool prints a measured ETA from live throughput after every batch, so a run
can be stopped at any point without wasting work: output is append-only JSONL and `--resume` skips finished tasks.

Usage:
  python tools/distill_labels.py --budget 400 --out scratch/distill/labels_v1.jsonl
  python tools/distill_labels.py --budget 400 --out ... --resume      # continue where it stopped
"""
from __future__ import annotations

import argparse
import json
import random
import re
from concurrent.futures import ThreadPoolExecutor
import sys
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import loadtest_tasks as lt  # noqa: E402  (same prompts, schemas and validators as the evaluation)

ID_OK = re.compile(r"^[A-Za-z_][A-Za-z0-9_\.]{2,}$")
SYS_TEACH = ("You are a precise code analyst. Cite only identifiers that appear verbatim in the excerpt. "
             "Reply with one JSON object only.")


def build_jobs(seed: int) -> list[dict]:
    """Same task construction as tools/loadtest_tasks.py, so labels match the evaluation distribution exactly."""
    sources = lt.collect_sources(1000)
    jobs = []
    # Mirrors tools/loadtest_tasks.py exactly (same source rotation, same TASK_ORDER, same excerpt) so a label's
    # (file, task) key matches the evaluation record it is meant to fix.
    for i in range(1000):
        src = sources[i % len(sources)]
        ex = lt.excerpt_of(src)
        if not ex:
            continue
        task = lt.TASK_ORDER[i % len(lt.TASK_ORDER)]
        fname = str(src.relative_to(ROOT))
        jobs.append({"id": i, "file": fname, "task": task, "excerpt": ex,
                     "prompt": f"{lt.TASKS[task]['ask']}\n\nFILE: {fname}\nEXCERPT:\n{ex}",
                     "decls": lt.declaration_symbols(ex)})
    return jobs


def student_failures(labels_path: Path) -> set[tuple[str, str]]:
    """(file, task) pairs the student failed on, from an existing harness run."""
    failed = set()
    if not labels_path.exists():
        return failed
    for line in labels_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        r = json.loads(line)
        ok = r.get("schema_ok") and all(ID_OK.match(s) for s in (r.get("symbols") or []))
        if not ok:
            failed.add((r["file"], r["task"]))
    return failed


def ask_teacher(port: int, prompt: str, schema: dict, max_tokens: int, timeout: float) -> dict:
    body = {"messages": [{"role": "system", "content": SYS_TEACH}, {"role": "user", "content": prompt}],
            "max_tokens": max_tokens, "temperature": 0.1, "stream": False,
            "chat_template_kwargs": {"enable_thinking": False},
            "response_format": {"type": "json_schema", "json_schema": {"name": "answer", "strict": True, "schema": schema}}}
    req = urllib.request.Request(f"http://127.0.0.1:{port}/v1/chat/completions",
                                 data=json.dumps(body).encode(), headers={"Content-Type": "application/json"})
    t0 = time.time()
    try:
        d = json.loads(urllib.request.urlopen(req, timeout=timeout).read())
        m = d["choices"][0]["message"]
        return {"ok": True, "content": m.get("content") or "", "latency_s": time.time() - t0,
                "completion_tokens": (d.get("usage") or {}).get("completion_tokens")}
    except Exception as e:  # noqa: BLE001
        return {"ok": False, "error": str(e)[:140], "latency_s": time.time() - t0}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--budget", type=int, default=400, help="labels to write")
    ap.add_argument("--out", required=True)
    ap.add_argument("--port", type=int, default=8612)
    ap.add_argument("--student-run", default="scratch/loadtest/run1000_clean/results.jsonl")
    ap.add_argument("--max-tokens", type=int, default=600)
    ap.add_argument("--timeout", type=float, default=300)
    ap.add_argument("--seed", type=int, default=23)
    ap.add_argument("--resume", action="store_true")
    ap.add_argument("--workers", type=int, default=8, help="concurrent teacher requests (match the server's --parallel)")
    args = ap.parse_args()

    out = ROOT / args.out
    out.parent.mkdir(parents=True, exist_ok=True)
    done: set[tuple[str, str]] = set()
    if args.resume and out.exists():
        for line in out.read_text(encoding="utf-8").splitlines():
            if line.strip():
                r = json.loads(line)
                done.add((r["file"], r["task"]))

    failed = student_failures(ROOT / args.student_run)
    jobs = build_jobs(args.seed)
    fail_jobs = [j for j in jobs if (j["file"], j["task"]) in failed and (j["file"], j["task"]) not in done]
    pass_jobs = [j for j in jobs if (j["file"], j["task"]) not in failed and (j["file"], j["task"]) not in done]
    random.Random(args.seed).shuffle(pass_jobs)
    # 3:1 failures:passes - the deficit is what needs teaching, but pure failures teaches "always cite"
    queue = fail_jobs + pass_jobs[: max(1, len(fail_jobs) // 3)]
    print(f"[queue] {len(fail_jobs)} failure tasks + {min(len(pass_jobs), len(fail_jobs)//3)} pass tasks "
          f"| budget {args.budget} | student failures known: {len(failed)}")

    written = kept_fail = 0
    rejected = {"schema": 0, "not_grounded": 0, "short_ident": 0, "not_a_declaration": 0, "call_error": 0}
    t_start, latencies = time.time(), []
    # MEASURED 2026-11-14: one request at a time against a server configured with --parallel 8 leaves 7 slots idle,
    # which is most of the "4 hour" estimate. Submitting with the same width as the server is the difference between
    # ~7 labels/min and ~55.
    with ThreadPoolExecutor(max_workers=max(1, args.workers)) as pool, out.open("a", encoding="utf-8") as fh:
        it = iter(queue)
        pending = {}
        def submit_next():
            for j in it:
                pending[pool.submit(ask_teacher, args.port, j["prompt"], lt.schema_for(j["task"]),
                                    args.max_tokens, args.timeout)] = j
                return True
            return False
        for _ in range(args.workers):
            if not submit_next():
                break
        while pending and written < args.budget:
            fut = next(f for f in pending if f.done()) if any(f.done() for f in pending) else None
            if fut is None:
                time.sleep(0.05)
                continue
            j = pending.pop(fut)
            submit_next()
            res = fut.result()
            if not res["ok"]:
                rejected["call_error"] += 1
                continue
            latencies.append(res["latency_s"])
            try:
                parsed = json.loads(lt.strip_json(res["content"]))
            except Exception:
                rejected["schema"] += 1
                continue
            if not isinstance(parsed, dict):
                rejected["schema"] += 1
                continue
            syms = parsed.get("evidence_symbols") or []
            if not all(ID_OK.match(str(s)) for s in syms) or len(syms) < 2:
                rejected["short_ident"] += 1
                continue
            if not all(s in j["excerpt"] for s in syms):
                rejected["not_grounded"] += 1
                continue
            # MEASURED 2026-11-14: "appears in the excerpt" is not enough - the enum-constrained student hit 100 %
            # on that metric while only 18.9 % of its citations were actual declarations (ungrammared: 20.6 %).
            # A distillation label is only worth teaching if its evidence names something the file DECLARES.
            if not any(s in j["decls"] for s in syms):
                rejected["not_a_declaration"] += 1
                continue
            fh.write(json.dumps({"file": j["file"], "task": j["task"], "prompt": j["prompt"],
                                 "completion": res["content"], "teacher": "Bonsai2-27B-PTQ1_0",
                                 "latency_s": res["latency_s"], "student_had_failed": (j["file"], j["task"]) in failed},
                                ensure_ascii=False) + "\n")
            fh.flush()
            written += 1
            if (j["file"], j["task"]) in failed:
                kept_fail += 1
            if written % 25 == 0:
                el = time.time() - t_start
                rate = written / el
                eta = (args.budget - written) / rate / 60 if rate else 0
                print(f"[{written}/{args.budget}] {rate*60:.1f} labels/min | elapsed {el/60:.1f} min | "
                      f"ETA {eta:.0f} min | p50 {sorted(latencies)[len(latencies)//2]:.1f}s | reject {rejected}", flush=True)

    el = time.time() - t_start
    print(f"\n=== HARVEST ===\n  written {written} (from-student-failures {kept_fail}) in {el/60:.1f} min")
    print(f"  throughput {written/max(el,1)*60:.1f} labels/min | rejected {rejected}")
    print(f"  out {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
