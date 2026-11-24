#!/usr/bin/env python3
"""Daily digest — the one scheduled message that reaches Telegram.

WHY (measured 2026-11-24)
------------------------
All 11 cron jobs delivered `local` or `origin`, so nothing the platform measures ever reached the channel the
user actually reads. The bot only ever spoke when spoken to. This is the digest that closes that gap: one
message a day, computed from the live endpoints, with the numbers a reader can act on and nothing that is
carried forward from a previous report.

Every figure comes from a response or a command run in this process. Anything that cannot be read is printed
as unmeasured rather than omitted, so a missing line is visible too.

Usage: telegram_digest.py            (prints the digest; cron delivers it)
       telegram_digest.py --check    (prints and exits without asserting anything)
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
from datetime import datetime
from pathlib import Path

KERNEL = os.environ.get("TRM_KERNEL", "http://127.0.0.1:8022")
REPO = Path(os.environ.get("TRM_REPO", r"F:\Development sites\TRM-Swarm-OS-v2"))


def get(path: str, timeout: float = 10.0):
    with urllib.request.urlopen(f"{KERNEL}{path}", timeout=timeout) as r:  # noqa: S310 - localhost
        return json.loads(r.read().decode("utf-8"))


def safe(path: str):
    try:
        return get(path), None
    except (urllib.error.URLError, urllib.error.HTTPError, OSError, ValueError) as exc:
        return None, f"{path} unreadable ({type(exc).__name__})"


def git_count_since_hours(hours: int = 24) -> str:
    try:
        out = subprocess.run(
            ["git", "-C", str(REPO), "log", f"--since={hours} hours ago", "--oneline"],
            capture_output=True, text=True, timeout=20,
        )
        n = len([l for l in out.stdout.splitlines() if l.strip()])
        return str(n)
    except (OSError, subprocess.SubprocessError):
        return "unmeasured"


def main() -> int:
    problems: list[str] = []
    lines: list[str] = []
    lines.append(f"TRM daily digest — {datetime.now().strftime('%Y-%m-%d %H:%M')}")

    health, err = safe("/health")
    if health:
        lines.append(f"platform: {health.get('status')} | build {health.get('build_commit')} | up {int(health.get('uptime_seconds') or 0)//60} min")
    else:
        problems.append(err)

    matrix, err = safe("/system/health_matrix")
    if matrix:
        lines.append(f"health matrix: {matrix.get('status')} ({matrix.get('anomaly_count')} anomalies)")
    else:
        problems.append(err)

    board, err = safe("/kanban/stats")
    if board:
        lines.append(
            f"board: {board.get('total_cards')} cards — {board.get('todo_count')} todo / "
            f"{board.get('in_progress_count')} doing / {board.get('review_count')} review / "
            f"{board.get('done_count')} done / {board.get('retired_count', 0)} retired"
        )
    else:
        problems.append(err)

    ddr, err = safe("/ddr/stats")
    if ddr:
        lines.append(f"ddr: {ddr.get('scans_performed')} scans (persistent) | mitigation {ddr.get('rowhammer_mitigation')}")
    else:
        problems.append(err)

    fed, err = safe("/federation/stats")
    if fed:
        lines.append(f"federation: {fed.get('active_nodes')}/{fed.get('configured_nodes')} peers active")
    else:
        problems.append(err)

    amr, err = safe("/amr/deployment")
    if amr:
        lines.append(f"amr node: {amr.get('verdict')} ({amr.get('steps_refuted')} refuted / {amr.get('steps_undetermined')} undetermined)")
    else:
        problems.append(err)

    lines.append(f"commits (24h): {git_count_since_hours(24)}")

    # open work that is actually open, not the whole board
    try:
        full = get("/kanban/board")
        todo = full.get("TODO") or []
        if todo:
            lines.append(f"open work ({len(todo)}):")
            for c in todo[:5]:
                lines.append(f"  · {c.get('card_id')} {(c.get('title') or '')[:66]}")
            if len(todo) > 5:
                lines.append(f"  +{len(todo) - 5} more")
    except (urllib.error.URLError, urllib.error.HTTPError, OSError, ValueError) as exc:
        problems.append(f"board unreadable ({type(exc).__name__})")

    jobs_path = Path(os.environ.get("TRM_CRON_JOBS", str(Path.home() / "AppData/Local/hermes/cron/jobs.json")))
    try:
        raw = json.loads(jobs_path.read_text(encoding="utf-8"))
        jobs = raw.get("jobs", raw) if isinstance(raw, dict) else raw
        if isinstance(jobs, dict):
            jobs = list(jobs.values())
        failing = [j for j in jobs if isinstance(j, dict) and j.get("last_status") not in (None, "ok")]
        if failing:
            lines.append(f"cron: {len(failing)} job(s) with a non-ok last status")
            for j in failing[:3]:
                lines.append(f"  · {j.get('name')}: {str(j.get('last_status'))}")
    except (OSError, ValueError):
        problems.append("cron job list unreadable")

    if problems:
        lines.append("unmeasured: " + "; ".join(problems))

    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main())
