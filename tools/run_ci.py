#!/usr/bin/env python
"""Single CI entrypoint for TRM Swarm OS.

Runs, in order:
  1. unit tests        pytest -m "not integration and not asgi and not integration_live"
  2. integration tests pytest -m integration   (requires the live stack)
  3. integration_live  pytest -m integration_live  (real HTTP to the nine ports)
  4. ledger concurrency tools/test_ledger_concurrency.py 8 125

Stage 3 is the one that proves the RUNNING stack works: it dials
8022/8099/3000/3001/3002/5183/8587/8588/8593 over TCP and asserts JSON shape.
Stage 2's `asgi`-marked tests are in-process TestClient(app) and are excluded
from stage 1 so they never masquerade as live coverage.

Prints one PASS/FAIL summary and exits non-zero if any stage fails.

Usage:
    python tools/run_ci.py                 # all four stages
    python tools/run_ci.py --skip-integration
    python tools/run_ci.py --only integration_live
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PY = sys.executable

STAGES = ("unit", "integration", "integration_live", "ledger")


def _run(name: str, cmd: list[str], timeout: int) -> tuple[bool, float, str]:
    print("\n" + "=" * 78)
    print(f"[CI] {name}: {' '.join(cmd)}")
    print("=" * 78, flush=True)
    started = time.time()
    try:
        proc = subprocess.run(
            cmd, cwd=ROOT, timeout=timeout, text=True,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        )
        out = proc.stdout or ""
        ok = proc.returncode == 0
        rc = proc.returncode
    except subprocess.TimeoutExpired as exc:
        out = (exc.stdout or "") if isinstance(exc.stdout, str) else ""
        out += f"\n[CI] TIMEOUT after {timeout}s"
        ok, rc = False, "timeout"
    except FileNotFoundError as exc:
        out, ok, rc = f"[CI] could not start: {exc}", False, "missing"

    # Keep the console readable but never hide the tail, which is where pytest
    # puts the failure summary.
    lines = out.rstrip().splitlines()
    if len(lines) > 120:
        print("\n".join(lines[:20]))
        print(f"... [{len(lines) - 100} lines omitted] ...")
        print("\n".join(lines[-80:]))
    else:
        print(out.rstrip())
    print(f"[CI] {name} -> {'PASS' if ok else 'FAIL'} (rc={rc}, {time.time() - started:.1f}s)",
          flush=True)
    return ok, time.time() - started, out


def main() -> int:
    ap = argparse.ArgumentParser(description="TRM Swarm OS CI entrypoint")
    ap.add_argument("--skip-integration", action="store_true",
                    help="skip the live-service integration stage")
    ap.add_argument("--skip-integration-live", action="store_true",
                    help="skip the real-HTTP integration_live stage")
    ap.add_argument("--only", choices=STAGES, help="run a single stage")
    ap.add_argument("--unit-timeout", type=int, default=1800)
    ap.add_argument("--integration-timeout", type=int, default=1800)
    ap.add_argument("--integration-live-timeout", type=int, default=900)
    ap.add_argument("--ledger-timeout", type=int, default=600)
    args = ap.parse_args()

    selected = [args.only] if args.only else list(STAGES)
    if args.skip_integration and "integration" in selected:
        selected.remove("integration")
    if args.skip_integration_live and "integration_live" in selected:
        selected.remove("integration_live")

    commands = {
        # asgi tests are in-process TestClient(app); they must not run as "unit"
        # and must not be counted as live coverage.
        "unit": ([PY, "-m", "pytest", "-q", "-m",
                  "not integration and not asgi and not integration_live"],
                 args.unit_timeout),
        "integration": ([PY, "-m", "pytest", "-q", "-m", "integration", "--timeout=120"],
                        args.integration_timeout),
        # Real HTTP against the nine live ports. --timeout is per-test; the Rust
        # council deliberation alone measured ~16s, so allow 180s.
        "integration_live": ([PY, "-m", "pytest", "-q", "-m", "integration_live",
                              "--timeout=180"],
                             args.integration_live_timeout),
        "ledger": ([PY, os.path.join("tools", "test_ledger_concurrency.py"), "8", "125"],
                   args.ledger_timeout),
    }

    results: list[tuple[str, bool, float]] = []
    for stage in selected:
        cmd, timeout = commands[stage]
        ok, elapsed, _ = _run(stage, cmd, timeout)
        results.append((stage, ok, elapsed))

    print("\n" + "=" * 78)
    print("CI SUMMARY")
    print("=" * 78)
    for stage, ok, elapsed in results:
        print(f"  {stage:<12} {'PASS' if ok else 'FAIL':<5} {elapsed:6.1f}s")
    failed = [s for s, ok, _ in results if not ok]
    overall = not failed
    print("-" * 78)
    print(f"  OVERALL: {'PASS' if overall else 'FAIL'}"
          + ("" if overall else f"  (failed: {', '.join(failed)})"))
    print("=" * 78)
    return 0 if overall else 1


if __name__ == "__main__":
    sys.exit(main())
