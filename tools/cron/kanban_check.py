#!/usr/bin/env python3
"""Deterministic kanban board check for the TRM Swarm platform.

WHY THIS EXISTS
---------------
`/kanban/board`, `/kanban/stats` and `/kanban/verify` have been served since G161, but nothing ever
called them on a schedule: the board was seeded once (2026-08-27), every card aged past the 7-day stale
threshold, and no job existed to notice. A board nobody reads is indistinguishable from a board that
does not work, so this reads it and says what it finds.

WHAT IT REFUSES TO DO
---------------------
It does not move cards, invent progress, or claim verification. `/kanban/verify` reports
`UNVERIFIED / not instrumented`, and this prints that verbatim rather than turning it into a tick.
Cards still flagged `seeded` are counted separately from any that changed organically, because the seed
is fiction (an FDA eCTD dossier, a muon-flux scrubber) and mixing the two would report a backlog that
never existed.

Fields the kernel does not instrument are reported as `unmeasured`, never guessed.

Exit codes: 0 = board read (whatever it says), 3 = kernel unreachable (nothing measured).
"""
from __future__ import annotations

import json
import os
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

KERNEL = os.environ.get("TRM_KERNEL", "http://127.0.0.1:8022")
STORE = Path(os.environ.get(
    "TRM_KANBAN_STORE",
    r"F:\Development sites\TRM-Swarm-OS-v2\data\kanban_board.json",
))


def get(path: str, timeout: float = 10.0):
    with urllib.request.urlopen(f"{KERNEL}{path}", timeout=timeout) as r:  # noqa: S310 - localhost only
        return json.loads(r.read().decode("utf-8"))


def age_days(ts: str | None) -> float | None:
    if not ts:
        return None
    try:
        when = datetime.fromisoformat(ts.replace("Z", "+00:00"))
    except ValueError:
        return None
    if when.tzinfo is None:
        when = when.replace(tzinfo=timezone.utc)
    return round((datetime.now(timezone.utc) - when).total_seconds() / 86400.0, 1)


def main() -> int:
    try:
        stats = get("/kanban/stats")
        board = get("/kanban/board")
        verify = get("/kanban/verify")
    except (urllib.error.URLError, urllib.error.HTTPError, OSError, ValueError) as exc:
        print(f"KANBAN: kernel unreachable at {KERNEL} ({type(exc).__name__}: {exc})")
        print("KANBAN: no values reported — the board was not measured, not found empty.")
        return 3

    lines: list[str] = []
    add = lines.append

    total = stats.get("total_cards")
    stale = stats.get("stale_count")
    stale_after = stats.get("stale_after_days")
    store = stats.get("store")
    add(f"KANBAN BOARD — {KERNEL} — {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}")
    add(f"  store: {store}  status: {stats.get('status')}")

    if total is None:
        add("  cards: unmeasured (the kernel did not return total_cards)")
    else:
        add(
            "  cards: {} total — todo {} / in_progress {} / review {} / done {} / retired {}".format(
                total,
                stats.get("todo_count", "unmeasured"),
                stats.get("in_progress_count", "unmeasured"),
                stats.get("review_count", "unmeasured"),
                stats.get("done_count", "unmeasured"),
                stats.get("retired_count", 0),
            )
        )

    # Staleness: this is the number that explains "it hasn't been running".
    if stale is not None and total:
        pct = round(100.0 * stale / total, 1)
        add(f"  stale: {stale}/{total} ({pct}%) — nothing moved for more than {stale_after} days")
    else:
        add("  stale: unmeasured")

    # Every card in the response, flattened, so seeded vs organic and real ages are visible.
    cards: list[dict] = []
    if isinstance(board, dict):
        for column, items in board.items():
            if isinstance(items, list):
                for c in items:
                    if isinstance(c, dict):
                        c = dict(c)
                        c.setdefault("status", column)
                        cards.append(c)

    seeded = [c for c in cards if c.get("seeded") is True]
    organic = [c for c in cards if c.get("seeded") is not True]
    if cards:
        add(f"  provenance: {len(seeded)} seeded card(s), {len(organic)} changed organically")
        aged = [age_days(c.get("updated_at")) for c in cards]
        aged = [a for a in aged if a is not None]
        if aged:
            add(f"  age: newest update {min(aged)} days ago, oldest {max(aged)} days ago")
        else:
            add("  age: unmeasured (no updated_at on any card)")

        # Seed origin vs last movement, so a frozen board is stated as frozen.
        origin = stats.get("seed_origin")
        if origin:
            d = age_days(origin)
            if d is not None:
                add(f"  last seed: {origin} ({d} days ago)")

    # The verify endpoint is honest about not verifying; repeat it rather than implying a pass.
    vstatus = verify.get("status") if isinstance(verify, dict) else None
    vmode = verify.get("verify_mode") if isinstance(verify, dict) else None
    if vstatus:
        add(f"  verification: {vstatus} ({vmode}) — no per-card evidence is instrumented")
        unverified = verify.get("cards") or []
        if isinstance(unverified, list) and unverified:
            with_evidence = [c for c in unverified if c.get("evidence")]
            add(f"  cards with evidence: {len(with_evidence)} of {len(unverified)}")

    # Unassigned / missing fields are a real defect class, so count them when present.
    missing_assignee = [c.get("card_id") for c in cards if not c.get("assignee")]
    if cards:
        add(f"  unassigned cards: {len(missing_assignee)}"
            + (f" ({', '.join(str(x) for x in missing_assignee[:5])})" if missing_assignee else ""))

    # Store file mtime: independent evidence of whether anything wrote to the board.
    try:
        mtime = datetime.fromtimestamp(STORE.stat().st_mtime, tz=timezone.utc)
        add(f"  store file: last written {mtime.strftime('%Y-%m-%d %H:%M UTC')} "
            f"({round((datetime.now(timezone.utc) - mtime).total_seconds() / 86400.0, 1)} days ago)")
    except OSError:
        add(f"  store file: unreadable at {STORE}")

    # A verdict that only repeats the numbers, and says what would change it.
    if stale is not None and total and stale == total:
        add("  verdict: FROZEN — every card is past the stale threshold; no card has moved since the seed.")
        add("  next: post real work through POST /kanban/cards/{id}/move, or retire the seeded cards.")
    elif stale:
        add("  verdict: STALE — some cards have not moved within the threshold.")
    elif total:
        add("  verdict: ACTIVE — at least one card moved within the stale threshold.")
    else:
        add("  verdict: EMPTY — the board holds no cards.")

    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main())
