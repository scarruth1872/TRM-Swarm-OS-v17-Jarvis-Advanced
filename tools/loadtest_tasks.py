#!/usr/bin/env python3
"""1,000-task load test with schema *and* grounding validation.

Why this exists: throughput alone is a meaningless number. A model that answers 8 items/s with unparseable prose,
or that cites symbols that are not in the file, has not done the work. Every task here is grounded on a real source
file and validated three ways:

  1. STRUCTURE - does the reply parse as JSON (code fences stripped) with the required keys?
  2. SCHEMA    - is `risk` inside its enum, are the lists non-empty and the right type?
  3. GROUNDING - does every symbol in `evidence_symbols` literally occur in the excerpt it was shown?

A task that fails 1 or 2 is a FAILURE, not a slow success. Grounding failures are reported separately because a
hallucinated citation is worse than an unparseable one: it looks right.

Usage:
  python tools/loadtest_tasks.py --n 1000 --ports 8587 --workers 4 --out scratch/loadtest/run1
"""
from __future__ import annotations

import argparse
import json
import os
import random
import re
import sys
import threading
import time
import urllib.error
import urllib.request
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXCERPT_CHARS = 2400

# Five task shapes. Variety is the point: a load test that sends the same prompt 1,000 times measures caching,
# not capability. Each shape has its own required keys so a model cannot reuse one answer layout.
ARRAY_KEYS = {"responsibilities", "evidence_symbols", "depends_on", "exposes", "failure_modes", "symbols", "tests", "risks"}


IDENT_RE = re.compile(r"(?<![A-Za-z0-9_])[A-Za-z_][A-Za-z0-9_]{2,}")


def excerpt_symbols(excerpt: str, cap: int = 160) -> list[str]:
    """Identifiers that literally occur in the excerpt, for enum-constrained citation.

    MEASURED 2026-11-14: the 1.5B's strict grounding is 59.0 % because it cites identifiers that are not in the file.
    A grammar whose evidence_symbols is an enum of the excerpt's own identifiers makes fabrication impossible by
    construction - the model must SELECT instead of INVENT. This does not prove it selects the *right* symbol, so a
    second objective check (is the cited symbol a declaration?) is reported alongside.
    """
    seen, out = set(), []
    for m in IDENT_RE.findall(excerpt):
        if m not in seen and not m.isdigit():
            seen.add(m)
            out.append(m)
            if len(out) >= cap:
                break
    return out


def declaration_symbols(excerpt: str) -> set[str]:
    """Names the excerpt actually declares (def/class/fn/impl/type/struct/const). Objective, no labels needed."""
    syms = set()
    for line in excerpt.splitlines():
        for rx in (r"\bdef\s+([A-Za-z_][A-Za-z0-9_]*)", r"\bclass\s+([A-Za-z_][A-Za-z0-9_]*)",
                   r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)", r"\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)",
                   r"\benum\s+([A-Za-z_][A-Za-z0-9_]*)", r"\bimpl\s+(?:\w+\s+for\s+)?([A-Za-z_][A-Za-z0-9_]*)",
                   r"\bconst\s+([A-Za-z_][A-Za-z0-9_]*)", r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*"):
            m = re.search(rx, line)
            if m:
                syms.add(m.group(1))
    return syms


def schema_for(task: str, symbols: list[str] | None = None) -> dict:
    """JSON schema for a task shape, built from the same `required`/`enums` the validator uses.

    MEASURED 2026-11-14: the Instella fork ignores every reasoning-off switch, so its think block competes with the
    answer for tokens (16 % schema at a 500-token budget, 21 % at 2,000). A grammar cannot emit " thinking" if the
    output must be JSON from the first token, which is the only remaining lever on that model.
    """
    spec = TASKS[task]
    props = {}
    for key in spec["required"]:
        if key in spec.get("enums", {}):
            props[key] = {"type": "string", "enum": spec["enums"][key]}
        elif key == "evidence_symbols" and symbols:
            # enumeration forces selection from the excerpt's own vocabulary
            props[key] = {"type": "array", "items": {"type": "string", "enum": symbols},
                          "minItems": 2, "maxItems": 2, "uniqueItems": True}
        elif key in ARRAY_KEYS:
            props[key] = {"type": "array", "items": {"type": "string"}, "minItems": 1, "maxItems": 6}
        else:
            props[key] = {"type": "string"}
    return {"type": "object", "properties": props, "required": list(spec["required"]),
            "additionalProperties": False}


TASKS = {
    "risk": {
        "ask": ('Report what this code is responsible for and how dangerous it is to change. Reply ONLY with a JSON object whose keys are: responsibilities (array of 2-4 short phrases), risk (exactly one of none, low, medium, high), evidence_symbols (array of 2 identifiers that appear verbatim in the excerpt), fix (one sentence, or an empty string).'),
        "required": ["responsibilities", "risk", "evidence_symbols", "fix"],
        "enums": {"risk": ["none", "low", "medium", "high"]},
    },
    "coupling": {
        "ask": ('Identify what this code depends on and what depends on it. Reply ONLY with a JSON object whose keys are: depends_on (array of module or type names it calls, taken from the excerpt), exposes (array of public names other code would call), coupling (exactly one of low, medium, high), evidence_symbols (array of 2 identifiers that appear verbatim in the excerpt).'),
        "required": ["depends_on", "exposes", "coupling", "evidence_symbols"],
        "enums": {"coupling": ["low", "medium", "high"]},
    },
    "failure": {
        "ask": ('List the ways this code can fail at runtime, most likely first. Reply ONLY with a JSON object whose keys are: failure_modes (array of 2-4 concrete failure descriptions), most_likely (one phrase), evidence_symbols (array of 2 identifiers that appear verbatim in the excerpt), guard (one sentence, or an empty string).'),
        "required": ["failure_modes", "most_likely", "evidence_symbols", "guard"],
        "enums": {},
    },
    "testability": {
        "ask": ('Judge how testable this code is and what would block a test. Reply ONLY with a JSON object whose keys are: testability (exactly one of easy, possible, hard), blockers (array of 0-3 concrete obstacles), entry_point (one function or method name taken from the excerpt), evidence_symbols (array of 2 identifiers that appear verbatim in the excerpt).'),
        "required": ["testability", "blockers", "entry_point", "evidence_symbols"],
        "enums": {"testability": ["easy", "possible", "hard"]},
    },
    "docstring": {
        "ask": ('Write what the documentation for this code should say. Reply ONLY with a JSON object whose keys are: summary (one sentence describing the contract), inputs (array of 0-3 parameters with their meaning), returns (what it yields, or an empty string), evidence_symbols (array of 2 identifiers that appear verbatim in the excerpt).'),
        "required": ["summary", "inputs", "returns", "evidence_symbols"],
        "enums": {},
    },
}
TASK_ORDER = list(TASKS)
SYS = ("You are a precise code analyst. You reply with one JSON object and nothing else. "
       "Every identifier you cite must appear character for character in the excerpt you were given. "
       "Never repeat the wording of these instructions inside a field.")


def collect_sources(limit: int) -> list[Path]:
    """Real files from the repo, ranked by size so excerpts are substantive."""
    exts = {".py", ".rs", ".ts"}
    skip = {"node_modules", "target", ".git", "dist", "build", "__pycache__", "models", "scratch", "logs"}
    files: list[Path] = []
    for base in ("swarm_v2", "host_council_server/src", "tools", "tests", "dashboard/src", "trmboot"):
        d = ROOT / base
        if not d.exists():
            continue
        for p in d.rglob("*"):
            if p.suffix in exts and p.is_file() and not (set(p.parts) & skip):
                try:
                    if 1500 <= p.stat().st_size <= 400_000:
                        files.append(p)
                except OSError:
                    pass
    files.sort(key=lambda p: (-p.stat().st_size, str(p)))
    random.Random(7).shuffle(files)
    return files[:limit]


def excerpt_of(path: Path, chars: int = EXCERPT_CHARS) -> str:
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""
    return text[:chars]


def strip_json(content: str) -> str:
    """Models wrap JSON in fences or lead with a think block. Take the outermost object.

    MEASURED 2026-11-13: the Instella fork ignores `enable_thinking` and streams its reasoning inline in `content`,
    so the reply is " thinking ... </think> {json}". Extracting the first `{` of that text finds a brace inside the
    reasoning, not the answer. Everything up to the LAST closing think tag is dropped first.
    """
    c = content.strip()
    if "</think>" in c:
        c = c.rsplit("</think>", 1)[1].strip()
    c = re.sub(r"^```(?:json)?", "", c).strip()
    c = re.sub(r"```$", "", c).strip()
    i, j = c.find("{"), c.rfind("}")
    return c[i:j + 1] if i != -1 and j > i else c


def validate(shape: dict, raw: str, excerpt: str) -> dict:
    out = {"parse_ok": False, "schema_ok": False, "grounded": None, "bad_enum": None, "missing": None}
    try:
        obj = json.loads(strip_json(raw))
    except (json.JSONDecodeError, ValueError):
        return out
    out["parse_ok"] = True
    if not isinstance(obj, dict):
        return out
    missing = [k for k in shape["required"] if k not in obj]
    bad = []
    for k, allowed in shape["enums"].items():
        v = obj.get(k)
        if not isinstance(v, str) or v.strip().lower() not in allowed:
            bad.append(f"{k}={v!r}")
    out["missing"] = missing or None
    out["bad_enum"] = bad or None
    out["schema_ok"] = not missing and not bad
    syms = obj.get("evidence_symbols")
    if isinstance(syms, list) and syms:
        checked = [s for s in syms if isinstance(s, str) and s.strip()]
        out["grounded"] = bool(checked) and all(s in excerpt for s in checked)
        out["symbols"] = checked[:4]
    else:
        out["grounded"] = False
    return out


def slot_ctx(port: int) -> int | None:
    """Per-slot context. llama-server divides --ctx-size by --parallel, so :8587 was serving 1024 tokens per slot
    while dense Rust excerpts cost 1,121 - that produced 28 HTTP 400s that looked like model failures."""
    try:
        d = json.loads(urllib.request.urlopen(f"http://127.0.0.1:{port}/props", timeout=10).read())
        # this build exposes per-slot context only under default_generation_settings; top-level "n_ctx" is absent,
        # and a None there silently fell back to a guessed budget - the same class of defect as a defaulted nominal.
        n = d.get("n_ctx") or (d.get("default_generation_settings") or {}).get("n_ctx")
        return int(n) if n else None
    except Exception:
        return None


def count_tokens(port: int, text: str) -> int | None:
    try:
        req = urllib.request.Request(f"http://127.0.0.1:{port}/tokenize",
                                     data=json.dumps({"content": text}).encode(),
                                     headers={"Content-Type": "application/json"})
        return len(json.loads(urllib.request.urlopen(req, timeout=30).read()).get("tokens", []))
    except Exception:
        return None


def call(port: int, prompt: str, max_tokens: int, timeout: float, no_think: bool = False,
         schema: dict | None = None) -> dict:
    payload = {"messages": [{"role": "system", "content": SYS}, {"role": "user", "content": prompt}],
               "max_tokens": max_tokens, "temperature": 0.1, "stream": False}
    if no_think:
        # MEASURED 2026-11-13: the Bonsai 2 family is a thinking model. At 200 tokens it returned content="" with
        # 738 characters of reasoning - a blank answer that scores as total failure while the model is working fine.
        # With enable_thinking=false the same task answered valid JSON in 0.2 s. Structured extraction does not want
        # the think block, so this is opt-in and recorded in the summary config.
        payload["chat_template_kwargs"] = {"enable_thinking": False}
    if schema is not None:
        payload["response_format"] = {"type": "json_schema",
                                      "json_schema": {"name": "answer", "strict": True, "schema": schema}}
    body = json.dumps(payload).encode()
    req = urllib.request.Request(f"http://127.0.0.1:{port}/v1/chat/completions", data=body,
                                 headers={"Content-Type": "application/json"})
    t0 = time.time()
    try:
        try:
            d = json.loads(urllib.request.urlopen(req, timeout=timeout).read())
        except Exception as e0:  # noqa: BLE001
            # MEASURED 2026-11-14: restarting a model server mid-run produced 89 x HTTP 503 at ~0 ms, and the harness
            # still printed a "SUMMARY" over them - statistics about a dead port, which reads as a model result. A 503
            # during model load is not a model failure, so retry once; if it persists the run is marked INVALID below.
            if "503" in str(e0) or "10061" in str(e0) or "10054" in str(e0):
                time.sleep(8)
                d = json.loads(urllib.request.urlopen(req, timeout=timeout).read())
            else:
                raise
        el = time.time() - t0
        ch = d["choices"][0]["message"]
        content = ch.get("content") or ""
        u = d.get("usage", {})
        return {"ok": True, "content": content, "latency_s": el,
                "prompt_tokens": u.get("prompt_tokens"), "completion_tokens": u.get("completion_tokens"),
                "reasoning_chars": len(ch.get("reasoning_content") or "")}
    except Exception as e:  # noqa: BLE001 - the error text is the measurement
        return {"ok": False, "error": f"{type(e).__name__}: {str(e)[:120]}", "latency_s": time.time() - t0}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=1000)
    ap.add_argument("--ports", default="8587", help="comma list of chat ports")
    ap.add_argument("--slots", type=int, default=4, help="concurrent slots per port")
    ap.add_argument("--workers", type=int, default=0, help="override total concurrency")
    ap.add_argument("--max-tokens", type=int, default=420)
    ap.add_argument("--timeout", type=float, default=150.0)
    ap.add_argument("--out", default="scratch/loadtest/run")
    ap.add_argument("--seed", type=int, default=11)
    ap.add_argument("--no-think", action="store_true", help="send enable_thinking=false (thinking models)")
    ap.add_argument("--symbol-enum", nargs="?", const="all", default=None, choices=["all", "decls"],
                    help="constrain evidence_symbols: 'all' = any identifier in the excerpt, "
                         "'decls' = only names the excerpt declares (def/class/fn/struct/impl/const/assignment)")
    ap.add_argument("--json-schema", action="store_true",
                    help="constrain sampling to the task's JSON schema (grammar decoding)")
    args = ap.parse_args()

    ports = [int(p) for p in args.ports.split(",") if p.strip()]
    workers = args.workers or args.slots * len(ports)
    outdir = ROOT / args.out
    outdir.mkdir(parents=True, exist_ok=True)
    results_path = outdir / "results.jsonl"
    results_path.write_text("", encoding="utf-8")

    sources = collect_sources(args.n)
    if not sources:
        print("no source files found", file=sys.stderr)
        return 2
    ctx = {p: slot_ctx(p) for p in ports}
    print(f"[ctx] per-slot context: {ctx}", flush=True)
    rnd = random.Random(args.seed)
    jobs = []
    for i in range(args.n):
        src = sources[i % len(sources)]
        ex = excerpt_of(src)
        if not ex:
            continue
        tname = TASK_ORDER[i % len(TASK_ORDER)]
        port = ports[i % len(ports)]
        ask = TASKS[tname]["ask"]
        budget = (ctx.get(port) or 4096) - args.max_tokens - 64
        prompt = f"{ask}\n\nFILE: {src.relative_to(ROOT)}\nEXCERPT:\n{ex}"
        ntok = count_tokens(port, prompt)
        while ntok and ntok > budget and len(ex) > 400:
            ex = ex[: int(len(ex) * 0.8)]
            prompt = f"{ask}\n\nFILE: {src.relative_to(ROOT)}\nEXCERPT:\n{ex}"
            ntok = count_tokens(port, prompt)
        jobs.append({"id": i, "port": port, "file": str(src.relative_to(ROOT)),
                     "task": tname, "excerpt": ex, "prompt_tokens": ntok, "prompt": prompt})
    print(f"[plan] {len(jobs)} tasks | {len(ports)} port(s) {ports} | {workers} workers | {args.max_tokens} max_tokens",
          flush=True)

    lock = threading.Lock()
    fh = results_path.open("a", encoding="utf-8")
    done = {"n": 0}
    t_start = time.time()

    def run(job: dict) -> None:
        # enum source: every identifier in the excerpt, or only the names it declares
        enum_syms = None
        if args.json_schema and args.symbol_enum:
            if args.symbol_enum == "decls":
                enum_syms = sorted(declaration_symbols(job.get("excerpt", "")))[:160]
            else:
                enum_syms = excerpt_symbols(job.get("excerpt", ""))
        r = call(job["port"], job["prompt"], args.max_tokens, args.timeout, args.no_think,
                 schema_for(job["task"], enum_syms) if args.json_schema else None)
        rec = {"id": job["id"], "port": job["port"], "file": job["file"], "task": job["task"],
               "latency_s": round(r["latency_s"], 2), "ok": r["ok"]}
        if r["ok"]:
            v = validate(TASKS[job["task"]], r["content"], job["excerpt"])
            rec.update({"prompt_tokens": r["prompt_tokens"], "completion_tokens": r["completion_tokens"],
                        "reasoning_chars": r["reasoning_chars"], "content_chars": len(r["content"]),
                        **v, "content": r["content"][:400]})
        else:
            rec.update({"error": r["error"], "parse_ok": False, "schema_ok": False, "grounded": None})
        with lock:
            fh.write(json.dumps(rec) + "\n")
            fh.flush()
            done["n"] += 1
            if done["n"] % 50 == 0 or done["n"] == len(jobs):
                el = time.time() - t_start
                print(f"[{done['n']}/{len(jobs)}] {el:7.1f}s  {done['n']/el:6.2f} items/s", flush=True)

    import queue
    q: queue.Queue = queue.Queue()
    for j in jobs:
        q.put(j)

    def worker() -> None:
        while True:
            try:
                j = q.get_nowait()
            except queue.Empty:
                return
            run(j)

    threads = [threading.Thread(target=worker, daemon=True) for _ in range(workers)]
    [t.start() for t in threads]
    [t.join() for t in threads]
    fh.close()

    wall = time.time() - t_start
    recs = [json.loads(l) for l in results_path.read_text(encoding="utf-8").splitlines() if l.strip()]
    n = len(recs)
    lat = sorted(r["latency_s"] for r in recs)
    def pct(p: float) -> float:
        return round(lat[min(int(p / 100 * len(lat)), len(lat) - 1)], 2) if lat else 0.0
    by_task: dict[str, dict] = defaultdict(lambda: {"n": 0, "schema_ok": 0, "grounded": 0, "parse_ok": 0})
    by_port: dict[int, dict] = defaultdict(lambda: {"n": 0, "schema_ok": 0, "grounded": 0, "errors": 0, "lat": []})
    for r in recs:
        t = by_task[r["task"]]
        t["n"] += 1
        t["parse_ok"] += bool(r.get("parse_ok"))
        t["schema_ok"] += bool(r.get("schema_ok"))
        t["grounded"] += bool(r.get("grounded"))
        p = by_port[r["port"]]
        p["n"] += 1
        p["schema_ok"] += bool(r.get("schema_ok"))
        p["grounded"] += bool(r.get("grounded"))
        p["errors"] += 0 if r["ok"] else 1
        p["lat"].append(r["latency_s"])
    summary = {
        "jobs": len(jobs), "completed": n, "wall_s": round(wall, 1),
        "items_per_s": round(n / wall, 3) if wall else 0,
        "calls_ok": sum(1 for r in recs if r["ok"]),
        "calls_failed": sum(1 for r in recs if not r["ok"]),
        "parse_ok": sum(1 for r in recs if r.get("parse_ok")),
        "schema_ok": sum(1 for r in recs if r.get("schema_ok")),
        "grounded": sum(1 for r in recs if r.get("grounded")),
        "schema_ok_pct": round(100 * sum(1 for r in recs if r.get("schema_ok")) / n, 1) if n else 0,
        "grounded_pct": round(100 * sum(1 for r in recs if r.get("grounded")) / n, 1) if n else 0,
        "latency_p50": pct(50), "latency_p95": pct(95), "latency_max": lat[-1] if lat else 0,
        "completion_tokens": sum(r.get("completion_tokens") or 0 for r in recs),
        "prompt_tokens": sum(r.get("prompt_tokens") or 0 for r in recs),
        "by_task": {k: v for k, v in by_task.items()},
        "by_port": {str(k): {kk: (round(sum(vv) / len(vv), 2) if kk == "lat" and vv else vv)
                             for kk, vv in v.items()} for k, v in by_port.items()},
        "config": vars(args),
    }
    (outdir / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print("\n=== SUMMARY ===")
    for k in ("jobs", "completed", "calls_failed", "items_per_s", "schema_ok_pct", "grounded_pct",
              "latency_p50", "latency_p95", "latency_max"):
        print(f"  {k:<16} {summary[k]}")
    print("  by_task:", {k: f"{v['schema_ok']}/{v['n']}" for k, v in by_task.items()})
    print(f"  results: {results_path}")

    # A run is only a measurement if the port stayed up. >20 % failed calls means the harness measured an outage
    # (server restarted, port closed, model reloading) - that is never a model result.
    failed_pct = 100 * summary["calls_failed"] / max(summary["jobs"], 1)
    summary["VALID"] = failed_pct <= 20.0
    if not summary["VALID"]:
        reason = (f"{summary['calls_failed']} of {summary['jobs']} calls failed ({failed_pct:.1f} %); "
                  f"the port was restarted or closed during the run, so these numbers describe the outage, not the model")
        summary["INVALID_REASON"] = reason
        (outdir / "INVALID.md").write_text(
            f"# INVALID RUN - do not cite\n\n{reason}\n\n"
            f"First failure kinds: {[r.get('error', '')[:80] for r in recs if not r['ok']][:3]}\n", encoding="utf-8")
        print("\n" + "!" * 78)
        print("RUN INVALID - DO NOT CITE: " + reason)
        print("!" * 78)
    (outdir / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
    return 0 if summary["VALID"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
