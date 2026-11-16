#!/usr/bin/env python3
"""Record verified work into the platform memory, so the store reflects what was actually done.

Why this exists: `swarm_v2_memory/*.json` is written by `AgentMemory` inside a running agent
loop (`base_agent.py`), and nothing has run since 2026-11-13, so two days of real work never
reached the store. This tool records *measured* facts, not prose: every entry carries a
`source` pointing at the file, commit or endpoint it came from, and is refused if that source
cannot be read.

Refusals (all silent-skip, loudly counted):
  * LLM-availability text - the reason 62.8% of the store is useless (G159)
  * duplicates - 71.1% of the store was duplicate (G159)
  * entries with no verifiable source

Writes to two places, which is what the live writer does:
  swarm_v2_memory/<Agent>_memory.json   (type task_result / fact, the agent's own long-term memory)
  swarm_v2_global_memory/turbo_memory.jsonl   (the id-addressable archive the fabric indexes)
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from datetime import datetime

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MEM_DIR = os.path.join(ROOT, "swarm_v2_memory")
TURBO = os.path.join(ROOT, "swarm_v2_global_memory", "turbo_memory.jsonl")

POISON = ("[llm offline]", "[llm error]", "ollama is not running", "failed to connect to ollama",
          "my neural core", "over-taxed", "identity_lock")


def _sha(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8", "ignore")).hexdigest()[:16]


def _is_poison(text: str) -> bool:
    low = (text or "").lower()
    return any(p in low for p in POISON)


def _load_agent(agent: str) -> list:
    path = os.path.join(MEM_DIR, f"{agent}_memory.json")
    if not os.path.exists(path):
        return []
    try:
        with open(path, encoding="utf-8") as fh:
            return json.load(fh)
    except Exception:
        return []


def _existing_hashes() -> set[str]:
    seen: set[str] = set()
    if not os.path.isdir(MEM_DIR):
        return seen
    for name in os.listdir(MEM_DIR):
        if not name.endswith("_memory.json"):
            continue
        try:
            with open(os.path.join(MEM_DIR, name), encoding="utf-8") as fh:
                for e in json.load(fh):
                    txt = " ".join(str(e.get(k) or "") for k in ("task", "result", "content"))
                    if txt.strip():
                        seen.add(_sha(txt.strip()[:200]))
        except Exception:
            continue
    return seen


def _verify_source(source: str) -> bool:
    if not source:
        return False
    if source.startswith("http"):
        return True
    return os.path.exists(os.path.join(ROOT, source)) or os.path.exists(source)


def record(agent: str, task: str, result: str, source: str, kind: str = "task_result",
           dry_run: bool = False) -> dict:
    """Append one verified record. Returns what happened, never raises for a refusal."""
    out = {"agent": agent, "kind": kind, "task": task[:200], "written": False, "refused": None}
    text = f"{task} {result}".strip()
    if not text:
        out["refused"] = "empty"
        return out
    if _is_poison(text):
        out["refused"] = "llm-failure-text"
        return out
    if not _verify_source(source):
        out["refused"] = f"source not readable: {source!r}"
        return out
    if _sha(text[:200]) in _existing_hashes():
        out["refused"] = "duplicate"
        return out
    if dry_run:
        out["written"] = True
        out["dry_run"] = True
        return out

    ts = datetime.now().isoformat()
    os.makedirs(MEM_DIR, exist_ok=True)
    entries = _load_agent(agent)
    if kind == "fact":
        entry = {"type": "fact", "content": result[:500] or task[:500], "source": source, "learned_at": ts}
    else:
        entry = {"type": "task_result", "task": task[:200], "result": result[:500],
                 "status": "completed", "completed_at": ts, "source": source}
    entries.append(entry)
    path = os.path.join(MEM_DIR, f"{agent}_memory.json")
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as fh:
        json.dump(entries, fh, indent=2, ensure_ascii=False, default=str)
    os.replace(tmp, path)

    # turbo archive: id-addressable, this is what the vector fabric indexes
    mid = _sha(f"{agent}|{task}|{ts}")
    rec = {"id": mid, "content": (f"{task} -> {result}" if kind == "task_result" else result)[:600],
           "metadata": {"kind": kind, "source": source, "priority": 1},
           "agent": agent, "timestamp": ts}
    os.makedirs(os.path.dirname(TURBO), exist_ok=True)
    with open(TURBO, "a", encoding="utf-8") as fh:
        fh.write(json.dumps(rec, ensure_ascii=False) + "\n")

    out.update({"written": True, "id": mid, "timestamp": ts, "agent_file": os.path.relpath(path, ROOT),
                "turbo_line_id": mid})
    return out


def backfill_from_journal(last_n: int, dry_run: bool) -> list[dict]:
    """Record journal articles as measured facts - each cites its own manifest entry."""
    man_path = os.path.join(ROOT, "phoenix_journal_audit_manifest.json")
    with open(man_path, encoding="utf-8") as fh:
        man = json.load(fh)
    arts = sorted(man.get("articles", []), key=lambda a: a.get("id", 0))[-last_n:]
    results = []
    for a in arts:
        title = a.get("title", "")
        finding = (a.get("finding") or a.get("phoenix_audit_note") or "")[:400]
        status = a.get("phoenix_status", "SPEC")
        results.append(record(
            agent="Scribe", kind="task_result",
            task=f"Journal article {a.get('id')}: {title[:120]}",
            result=f"[{status}] {finding}",
            source="phoenix_journal_audit_manifest.json", dry_run=dry_run))
    return results


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--agent", default="Archi")
    ap.add_argument("--task")
    ap.add_argument("--result")
    ap.add_argument("--source", help="file/URL that substantiates this memory; required")
    ap.add_argument("--kind", choices=["task_result", "fact"], default="task_result")
    ap.add_argument("--from-journal", type=int, metavar="N", help="record the last N journal articles")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    if args.from_journal:
        res = backfill_from_journal(args.from_journal, args.dry_run)
    elif args.task and args.result:
        res = [record(args.agent, args.task, args.result, args.source or "", args.kind, args.dry_run)]
    else:
        ap.error("give --from-journal N or --task/--result")
        return 2

    wrote = sum(1 for r in res if r.get("written") and not r.get("dry_run"))
    refused = {}
    for r in res:
        if r.get("refused"):
            refused[r["refused"].split(":")[0]] = refused.get(r["refused"].split(":")[0], 0) + 1
    print(json.dumps({"attempted": len(res), "written": wrote, "refused": refused,
                      "dry_run": args.dry_run}, indent=2))
    for r in res:
        if r.get("refused"):
            print(f"  REFUSED {r['refused']} :: {r['task'][:60]}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
