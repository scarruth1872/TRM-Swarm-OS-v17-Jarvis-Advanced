#!/usr/bin/env python3
"""Telegram transport health — quiet when the channel is fine, loud when it is not.

WHY (measured 2026-11-24)
------------------------
The gateway's Telegram path failed and recovered **222 times in one day** (542 warnings: 222 sticky-path
failures, 222 IPv4 re-walks, 122 IPv4-IP failures, 103 dual-stack failures) and lost one send at 08:31 to a
timeout. Every failure recovered, which is exactly why nobody noticed: a flapping transport that always
recovers is invisible until the day it stops recovering, and then the channel simply goes silent with no
signal that anything is wrong.

This prints NOTHING when the channel is healthy (cron delivers empty stdout as nothing at all), and one
short line when it is not, so the silence itself keeps its meaning.

Exit: 0 always unless the log cannot be read (2).
"""
from __future__ import annotations

import os
import re
import sys
from datetime import datetime, timedelta
from pathlib import Path

LOG = Path(os.environ.get("HERMES_GATEWAY_LOG",
                          str(Path.home() / "AppData/Local/hermes/logs/gateway.log")))
WINDOW_MIN = int(os.environ.get("TRM_TG_WINDOW_MIN", "60"))
MAX_SILENT_FAILURES_WITHOUT_RECOVERY = 0  # any failure with zero recoveries is an alert


def main() -> int:
    if not LOG.exists():
        print(f"TELEGRAM TRANSPORT: gateway log missing at {LOG} — the channel is unmeasured, not healthy")
        return 2

    cutoff = datetime.now() - timedelta(minutes=WINDOW_MIN)
    warnings = recoveries = failed_sends = responses = notices = 0
    worst_send_failure = ""

    with LOG.open("r", encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if "telegram" not in line.lower():
                continue
            try:
                ts = datetime.strptime(line[:19], "%Y-%m-%d %H:%M:%S")
            except ValueError:
                continue
            if ts < cutoff:
                continue
            low = line.lower()
            if "warning" in low and ("failed" in low or "re-walking" in low):
                warnings += 1
            if "recovered via" in low:
                recoveries += 1
            if "failed to send telegram message" in low:
                failed_sends += 1
                worst_send_failure = line.strip()[:120]
            if "response ready: platform=telegram" in low:
                responses += 1
            # Every injected notice is a turn: measured 2026-11-24, eight background-process notices each
            # cost an agent turn, three of them inside four minutes with no user message. The gateway has
            # no batching setting (only per-notice dedupe), so the count is what we can measure.
            if "watch pattern notification" in low or "injecting for telegram" in low:
                notices += 1

    problems = []
    if failed_sends > 0:
        problems.append(f"{failed_sends} send failure(s): {worst_send_failure}")
    if warnings > MAX_SILENT_FAILURES_WITHOUT_RECOVERY and recoveries == 0:
        problems.append(f"{warnings} transport failures and ZERO recoveries in {WINDOW_MIN} min — the channel may be down")
    if warnings > 0 and recoveries > 0 and warnings > 20 * max(recoveries, 1):
        problems.append(f"{warnings} failures vs {recoveries} recoveries in {WINDOW_MIN} min — flapping far above baseline")
    if notices >= 4:
        problems.append(f"{notices} background notices injected in {WINDOW_MIN} min — each one costs a turn; "
                        f"the gateway cannot batch them, so stop them at the source")

    if not problems:
        return 0  # healthy: silence is the signal

    print(f"TELEGRAM TRANSPORT ({WINDOW_MIN} min): {warnings} failures, {recoveries} recoveries, "
          f"{responses} responses, {failed_sends} failed sends, {notices} notices")
    for p in problems:
        print(f"  ! {p}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
