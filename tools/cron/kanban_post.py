#!/usr/bin/env python3
"""Post REAL work to the kanban board from measured platform state.

WHY THIS EXISTS
---------------
The board held 13 cards seeded on 2026-08-27 that never moved; 9 of them describe work that does not exist
(docs/KANBAN-AUDIT.md). The fix is not a prettier board, it is a board fed by measurement: this reads live
endpoints and posts one card per CURRENT defect.

WHAT IT POSTS
-------------
    /system/health_matrix  -> one card per reported anomaly (critical)
    /ddr/stats             -> card when the scanner has performed 0 scans, or rowhammer mitigation is
                              NOT_IMPLEMENTED (high)
    /federation/stats      -> card when configured peers are offline (high)
    Hermes cron jobs       -> card when a job's last run failed (medium)

WHAT IT REFUSES TO DO
---------------------
- It never posts progress, only defects. A card appears because something is wrong *now*; closing it is
  a `POST /kanban/cards/{id}/move`, i.e. a human or a later verified job, not this script deciding it is
  fixed.
- It never re-posts: every title is compared against the existing board, so running it twice adds nothing
  (the kernel's create endpoint numbers cards from `store.len()+1`, so without this check every run would
  duplicate the whole board).
- It never invents a figure: every number in a title comes from a field in the response, and a source that
  cannot be read is reported as unmeasured instead of being silently skipped.

Usage: kanban_post.py [--dry-run]
Exit: 0 = board read (posted or not), 3 = kernel unreachable.
"""
from __future__ import annotations

import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

KERNEL = os.environ.get("TRM_KERNEL", "http://127.0.0.1:8022")
JOBS = Path(os.environ.get(
    "TRM_CRON_JOBS",
    str(Path.home() / "AppData/Local/hermes/cron/jobs.json"),
))
MAX_TITLE = 160  # the kernel stores a card title and drops `description`; keep it meaningful and short


def get(path: str, timeout: float = 10.0):
    with urllib.request.urlopen(f"{KERNEL}{path}", timeout=timeout) as r:  # noqa: S310 - localhost only
        return json.loads(r.read().decode("utf-8"))


def post(path: str, payload: dict, timeout: float = 10.0):
    body = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{KERNEL}{path}", data=body, headers={"Content-Type": "application/json"}, method="POST"
    )
    with urllib.request.urlopen(req, timeout=timeout) as r:  # noqa: S310 - localhost only
        return json.loads(r.read().decode("utf-8"))


def trim(text: str, limit: int = MAX_TITLE) -> str:
    text = " ".join(text.split())
    return text if len(text) <= limit else text[: limit - 1].rstrip() + "…"


def current_titles(board) -> set[str]:
    """Collect active (non-DONE) card titles so closed defects can re-open if they recur."""
    titles: set[str] = set()
    if isinstance(board, dict):
        for column, items in board.items():
            if str(column).upper() == "DONE":
                continue
            if isinstance(items, list):
                for c in items:
                    if isinstance(c, dict) and c.get("title"):
                        if str(c.get("status", "")).upper() == "DONE":
                            continue
                        titles.add(str(c["title"]))
    return titles


def discoveries() -> tuple[list[dict], list[str], list[str]]:
    """Return (cards to post, unmeasured sources, skipped-as-history notes)."""
    cards: list[dict] = []
    unmeasured: list[str] = []
    history: list[str] = []

    # 1. health matrix anomalies
    try:
        hm = get("/system/health_matrix")
        for a in hm.get("anomalies") or []:
            svc = a.get("service") or a.get("service_name") or "unknown"
            msg = a.get("message") or "no message"
            cards.append({
                "title": trim(f"Anomaly: {svc} — {msg}"),
                "assignee": "Shield",
                "priority": "critical",
            })
        if not hm.get("anomalies"):
            pass  # a clean matrix posts nothing, which is the point
    except (urllib.error.URLError, urllib.error.HTTPError, OSError, ValueError) as exc:
        unmeasured.append(f"health_matrix unreadable ({type(exc).__name__})")

    # 2. DDR scanner: armed but idle, or mitigation absent
    try:
        ddr = get("/ddr/stats")
        scans = ddr.get("scans_performed")
        mit = ddr.get("rowhammer_mitigation")
        if scans == 0:
            cards.append({
                "title": trim(
                    f"DDR scanner armed but has performed {scans} scans "
                    f"(last_scan={ddr.get('last_scan')}, status={ddr.get('status')})"
                ),
                "assignee": "Shield",
                "priority": "high",
            })
        if isinstance(mit, str) and mit.upper().startswith("NOT_IMPLEMENTED"):
            cards.append({
                "title": trim(f"DDR rowhammer mitigation is {mit} — {ddr.get('distinct_rules')} rules loaded, none enforcing"),
                "assignee": "Shield",
                "priority": "high",
            })
    except (urllib.error.URLError, urllib.error.HTTPError, OSError, ValueError) as exc:
        unmeasured.append(f"ddr/stats unreadable ({type(exc).__name__})")

    # 3. federation peers offline
    try:
        fed = get("/federation/stats")
        offline = fed.get("offline_nodes")
        configured = fed.get("configured_nodes")
        active = fed.get("active_nodes")
        if isinstance(offline, int) and offline > 0:
            cards.append({
                "title": trim(
                    f"Federation: {offline} of {configured} configured peers offline "
                    f"({active} active) — reconcile or retire the entries"
                ),
                "assignee": "Bridge",
                "priority": "high",
            })
    except (urllib.error.URLError, urllib.error.HTTPError, OSError, ValueError) as exc:
        unmeasured.append(f"federation/stats unreadable ({type(exc).__name__})")

    # 4. Hermes cron jobs whose last run failed
    try:
        raw = json.loads(JOBS.read_text(encoding="utf-8"))
        jobs = raw.get("jobs", raw) if isinstance(raw, dict) else raw
        if isinstance(jobs, dict):
            jobs = list(jobs.values())
        for job in jobs if isinstance(jobs, list) else []:
            if not isinstance(job, dict):
                continue
            status = job.get("last_status")
            if not status or status == "ok":
                continue
            err = (job.get("last_error") or "no error text").splitlines()[0]

            # A last_error can be HISTORY. Four jobs still carry `HTTP 402: Insufficient Balance` from
            # their final LLM run on 2026-11-23 05:00-07:00, before G226 repointed them to no_agent
            # scripts. A provider-balance failure cannot recur when no provider is in the path, so
            # posting that card would re-open a defect that is already fixed — the board would fill with
            # archaeology. Skip it and say so, rather than silently dropping it.
            stale_provider_error = bool(job.get("no_agent")) and any(
                k in err for k in ("402", "Insufficient Balance", "provider", "model")
            )
            if stale_provider_error:
                history.append(
                    f"cron job {job.get('name') or job.get('job_id')}: last_error is pre-no_agent "
                    f"history ({err[:60]}), not a current defect — not posted"
                )
                continue

            cards.append({
                "title": trim(
                    f"Cron job failing: {job.get('name') or job.get('job_id')} — {err[:70]}",
                    140,
                ),
                "assignee": "Orchestra",
                "priority": "medium",
            })
    except (OSError, ValueError):
        unmeasured.append(f"cron job list unreadable at {JOBS}")

    return cards, unmeasured, history


def main(argv: list[str]) -> int:
    dry = "--dry-run" in argv

    try:
        board = get("/kanban/board")
        stats_before = get("/kanban/stats")
    except (urllib.error.URLError, urllib.error.HTTPError, OSError, ValueError) as exc:
        print(f"KANBAN POST: kernel unreachable at {KERNEL} ({type(exc).__name__}: {exc})")
        print("KANBAN POST: nothing posted and nothing measured.")
        return 3

    existing = current_titles(board)
    cards, unmeasured, history = discoveries()

    print(f"KANBAN POST — {KERNEL} — {'DRY RUN' if dry else 'live'}")
    print(f"  board before: {stats_before.get('total_cards')} cards")

    created = skipped = 0
    for card in cards:
        if card["title"] in existing:
            skipped += 1
            print(f"  skip (already on board): {card['title']}")
            continue
        if dry:
            print(f"  would post [{card['priority']}/{card['assignee']}]: {card['title']}")
            created += 1
            continue
        try:
            res = post("/kanban/cards", {**card, "target_status": "TODO"})
        except (urllib.error.URLError, urllib.error.HTTPError, OSError, ValueError) as exc:
            print(f"  POST FAILED for '{card['title']}': {type(exc).__name__}: {exc}")
            continue
        if res.get("persisted"):
            created += 1
            cid = (res.get("card") or {}).get("card_id")
            print(f"  posted {cid} [{card['priority']}/{card['assignee']}]: {card['title']}")
        else:
            print(f"  NOT PERSISTED: {res}")

    for note in unmeasured:
        print(f"  UNMEASURED: {note}")
    for note in history:
        print(f"  skipped as history: {note}")

    print(
        f"  posted {created}, already present {skipped}, "
        f"skipped as history {len(history)}, sources unreadable {len(unmeasured)}"
    )
    if created == 0 and skipped == 0 and not unmeasured and not history:
        print("  nothing to post: no current defect was reported by any source")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
