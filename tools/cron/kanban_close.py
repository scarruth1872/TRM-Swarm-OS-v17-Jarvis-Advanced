#!/usr/bin/env python3
"""Close kanban cards whose defect has measurably gone away.

The partner to kanban_post.py. The poster creates a card because a defect exists *now*; this closes it
only when the same measurement that opened it says the defect is gone, and it prints the before/after
evidence for every close.

RULES (one per source the poster uses). Each is a predicate over a live endpoint, never over elapsed time
and never over a card's own text:

    "DDR scanner armed but has performed N scans"   closes when /ddr/stats scans_performed > 0
    "DDR rowhammer mitigation is ..."               closes when rowhammer_mitigation is no longer NOT_IMPLEMENTED
    "Federation: N of M configured peers offline"    closes when offline_nodes == 0
    "Anomaly: <service> - ..."                      closes when health_matrix reports no anomaly for <service>
    "Cron job failing: <name> - ..."                closes when that job's last_status == ok

Everything else is left open and named as unverifiable, because a closer with no rule is not evidence of
anything. Seeded cards are never touched: they are not products of measurement, so no measurement can
retire them.

Usage: kanban_close.py [--dry-run|--self-test]
Exit: 0 normally, 3 when the kernel is unreachable.
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


def open_cards(board) -> list[dict]:
    out: list[dict] = []
    if isinstance(board, dict):
        for column, items in board.items():
            if isinstance(items, list):
                for c in items:
                    if isinstance(c, dict) and str(c.get("status", "")).upper() != "DONE":
                        out.append(c)
    return out


def rule_for(title: str):
    """Map a card title to (rule_name, resolver). resolver(state) -> (resolved: bool, evidence: str)."""
    t = title.strip()

    if t.startswith("DDR scanner armed but has performed"):
        def check(state):
            scans = state.get("ddr", {}).get("scans_performed")
            if scans is None:
                return False, "unmeasured: /ddr/stats returned no scans_performed"
            return scans > 0, f"/ddr/stats scans_performed = {scans} (> 0)"
        return "ddr_scans", check

    if t.startswith("DDR rowhammer mitigation is"):
        def check(state):
            mit = state.get("ddr", {}).get("rowhammer_mitigation")
            if mit is None:
                return False, "unmeasured: /ddr/stats returned no rowhammer_mitigation"
            return not str(mit).upper().startswith("NOT_IMPLEMENTED"), f"/ddr/stats rowhammer_mitigation = {mit}"
        return "ddr_mitigation", check

    if t.startswith("Federation:"):
        def check(state):
            off = state.get("fed", {}).get("offline_nodes")
            if off is None:
                return False, "unmeasured: /federation/stats returned no offline_nodes"
            return off == 0, f"/federation/stats offline_nodes = {off}"
        return "federation_offline", check

    if t.startswith("Anomaly:"):
        service = t.split(":", 1)[1].split("—")[0].strip()

        def check(state):
            hm = state.get("hm") or {}
            anomalies = hm.get("anomalies")
            if anomalies is None:
                # Absent data is not absence of the defect. Before this guard, an unreadable
                # /system/health_matrix defaulted to an empty list and CLOSED the anomaly card — the
                # self-test case {"hm": {}} caught it. Unknown must leave the card open.
                return False, "unmeasured: /system/health_matrix returned no anomalies field"
            still = [a for a in anomalies
                     if str(a.get("service") or a.get("service_name") or "").strip() == service]
            return len(still) == 0, f"/system/health_matrix anomalies mentioning {service!r} = {len(still)}"
        return f"anomaly:{service}", check

    if t.startswith("Cron job failing:"):
        name = t.split(":", 1)[1].split("—")[0].strip()

        def check(state):
            jobs = state.get("jobs")
            if not isinstance(jobs, list):
                return False, "unmeasured: cron job list unreadable"
            job = next((j for j in jobs if str(j.get("name") or "").strip() == name), None)
            if job is None:
                return False, f"no job named {name!r} in the job list"
            status = job.get("last_status")
            return status == "ok", f"job {name!r} last_status = {status}"
        return f"cron:{name}", check

    return None, None


def gather() -> dict:
    state: dict = {}
    for key, path in (("hm", "/system/health_matrix"), ("ddr", "/ddr/stats"), ("fed", "/federation/stats")):
        try:
            state[key] = get(path)
        except (urllib.error.URLError, urllib.error.HTTPError, OSError, ValueError):
            state[key] = {}
    try:
        raw = json.loads(JOBS.read_text(encoding="utf-8"))
        jobs = raw.get("jobs", raw) if isinstance(raw, dict) else raw
        if isinstance(jobs, dict):
            jobs = list(jobs.values())
        state["jobs"] = jobs if isinstance(jobs, list) else None
    except (OSError, ValueError):
        state["jobs"] = None
    return state


def self_test() -> int:
    """Prove each rule fires BOTH ways on synthetic state before trusting it on the live board."""
    cases = [
        ("DDR scanner armed but has performed 0 scans (last_scan=on request)", {"ddr": {"scans_performed": 1}}, True),
        ("DDR scanner armed but has performed 0 scans (last_scan=on request)", {"ddr": {"scans_performed": 0}}, False),
        ("DDR rowhammer mitigation is NOT_IMPLEMENTED — 7 rules loaded", {"ddr": {"rowhammer_mitigation": "ENFORCING"}}, True),
        ("DDR rowhammer mitigation is NOT_IMPLEMENTED — 7 rules loaded", {"ddr": {"rowhammer_mitigation": "NOT_IMPLEMENTED"}}, False),
        ("Federation: 3 of 5 configured peers offline (2 active)", {"fed": {"offline_nodes": 0}}, True),
        ("Federation: 3 of 5 configured peers offline (2 active)", {"fed": {"offline_nodes": 3}}, False),
        ("Anomaly: lcars_dashboard — unreachable", {"hm": {"anomalies": []}}, True),
        ("Anomaly: lcars_dashboard — unreachable", {"hm": {"anomalies": [{"service": "lcars_dashboard"}]}}, False),
        ("Cron job failing: X — boom", {"jobs": [{"name": "X", "last_status": "ok"}]}, True),
        ("Cron job failing: X — boom", {"jobs": [{"name": "X", "last_status": "error"}]}, False),
        ("Anomaly: svc — x", {"hm": {}}, False),  # absent data must NOT close
        ("DDR scanner armed but has performed 0 scans", {"ddr": {}}, False),
    ]
    failures = 0
    for title, state, want in cases:
        _, check = rule_for(title)
        if check is None:
            print(f"  FAIL no rule for {title!r}")
            failures += 1
            continue
        got, evidence = check(state)
        ok = got == want
        failures += 0 if ok else 1
        print(f"  {'ok  ' if ok else 'FAIL'} {title[:52]!r} -> resolved={got} (want {want}) | {evidence}")
    print(f"self-test: {len(cases) - failures}/{len(cases)} cases behaved as specified")
    return 0 if failures == 0 else 1


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    dry = "--dry-run" in argv

    try:
        board = get("/kanban/board")
    except (urllib.error.URLError, urllib.error.HTTPError, OSError, ValueError) as exc:
        print(f"KANBAN CLOSE: kernel unreachable at {KERNEL} ({type(exc).__name__}: {exc})")
        print("KANBAN CLOSE: nothing closed and nothing measured.")
        return 3

    state = gather()
    cards = open_cards(board)
    print(f"KANBAN CLOSE — {KERNEL} — {'DRY RUN' if dry else 'live'} — {len(cards)} open card(s)")

    closed = unverifiable = still_open = 0
    for card in cards:
        title = str(card.get("title") or "")
        cid = card.get("card_id")
        if card.get("seeded") is True:
            unverifiable += 1
            print(f"  {cid}: SEEDED — not a product of measurement, left untouched")
            continue
        name, check = rule_for(title)
        if check is None:
            unverifiable += 1
            print(f"  {cid}: no rule for this card, left open (unverifiable): {title[:70]}")
            continue
        try:
            resolved, evidence = check(state)
        except (KeyError, TypeError, AttributeError) as exc:
            unverifiable += 1
            print(f"  {cid}: rule {name} could not evaluate ({exc}); left open")
            continue
        if not resolved:
            still_open += 1
            print(f"  {cid}: STILL OPEN — {evidence}")
            continue
        if dry:
            closed += 1
            print(f"  {cid}: would close -> DONE — {evidence}")
            continue
        try:
            res = post(f"/kanban/cards/{cid}/move", {"target_status": "DONE"})
        except (urllib.error.URLError, urllib.error.HTTPError, OSError, ValueError) as exc:
            print(f"  {cid}: move failed ({type(exc).__name__}: {exc})")
            continue
        if res.get("persisted"):
            closed += 1
            print(f"  {cid}: CLOSED -> DONE — {evidence}")
        else:
            print(f"  {cid}: NOT PERSISTED: {res}")

    print(
        f"  closed {closed}, still open {still_open}, "
        f"left alone {unverifiable} (seeded or no rule)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
