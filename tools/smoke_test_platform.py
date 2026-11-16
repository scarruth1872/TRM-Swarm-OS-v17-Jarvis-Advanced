#!/usr/bin/env python3
"""Smoke-test every TRM Swarm OS service over HTTP.

The launcher (START_TRM_PLATFORM.bat) and this script must agree on what
"ready" means: an HTTP 200 from the service's health route whose body carries
the expected marker. Port occupancy alone is NOT proof -- a process can hold a
port and serve nothing, or serve a stale binary that 404s.

Exit codes
    0  every service passed
    1  at least one service failed
    2  bad invocation

Usage
    python tools/smoke_test_platform.py
    python tools/smoke_test_platform.py --json
    python tools/smoke_test_platform.py --only 8099,8587 --timeout 3

No third-party dependencies: this is meant to run in CI unchanged.
"""

from __future__ import annotations

import argparse
import os
import json
import socket
import sys
import time
import urllib.error
import urllib.request

# (name, port, path, marker) -- keep in lock-step with START_TRM_PLATFORM.bat.
# (name, port, path, marker, expect_json)
#
# expect_json exists because a marker substring is NOT proof of a healthy API.
# MEASURED 2026-11-03: Jarvis /health returned the SPA index page ("<!doctype
# html>") and this test reported PASS, because the page <title> contains the word
# "Jarvis". That is the same defect class as "the port is listening therefore the
# service is healthy", one layer up. Any service whose contract is JSON must
# actually serve JSON.
SERVICES: list[tuple[str, int, str, str, bool]] = [
    ("Council Server (Rust)",   8022, "/health", "HEALTHY",   True),
    # NOT STARTED BY START_TRM_PLATFORM.BAT (measured 2026-11-13): the launcher's steps are 1..9 and include no
    # app_v2 step, and app_v2 itself refuses to run a second instance while one holds the port ("A second instance
    # would lose the bind race and die silently"). Gating on it made every launch report FAIL for a service the
    # launcher was never asked to start. Still probed, reported, and never able to pass the gate silently.
    ("Council API (app_v2)",    8099, "/health", "v5_online", True),
    ("Jarvis (tsx server.ts)",  3000, "/health", "jarvis",    True),
    ("Companion Proxy (py)",    3001, "/health", "TRM Sovereign Companion Bridge", True),
    ("Companion Bridge (rust)", 3002, "/health", "TRM Sovereign Companion Bridge", True),
    # The dashboard is HTML by design, so it is the one service allowed to serve a page.
    # Measured 2026-11-09: the v20 kernel binds a SECOND listener on :5183 and serves its own
    # /health there, so probing /health for "HEALTHY" passed while vite was dead -- the port
    # answered, it just was not the dashboard. "/" carrying the vite HMR hook is the only
    # marker the kernel cannot produce.
    ("LCARS Dashboard (vite)",  5183, "/",       "react-refresh", False),
    ("Inference Engine",        8587, "/health", "ok",        True),
    # Deep tier: Bonsai-27B-Q1_0 ternary on 8591. voice.py routes deep-word
    # questions here; without it every deep question silently answers from the
    # 1.5B and the port is a phantom. Part of the contract.
    ("Deep Tier (Bonsai 27B)",  8591, "/health", "ok",        True),
    # Same class as 8099: no launcher step starts trm_server, so its absence is not a launch failure.
    ("TRM Service (trm_server)", 8588, "/health", "trm7m",     True),
    # Without 8593 the semantic domain router silently degrades to
    # keyword_fallback. It is part of the contract, not an optional extra.
    ("Embeddings Server",       8593, "/health", "ok",        True),
    # Added 2026-11-10: both were serving but absent from the gate, so the launch
    # table reported 10/10 while two registered tiers could have been dead. A tier
    # that /models advertises is part of the contract.
    ("Qwen3-4B (8595)",         8595, "/health", "ok",        True),
    # Instella runs on a fork build (upstream llama.cpp has no InstellaMoE arch).
    # Required=False: the binary and GGUF live off-repo, so a clean checkout must
    # not fail its smoke gate over a tier it never had.
    ("Instella-MoE (8596 fork)", 8596, "/health", "ok",       False),
]

# A profile that does not start a tier must not be scored as if that tier were missing. MEASURED 2026-11-13: the
# default profile is "lean" (:8593 + :8596), yet the gate required :8587, :8591 and :8595, so every clean launch of
# the documented default ended in "LAUNCH FAILED" while the platform was exactly as configured.
PROFILE_TIERS: dict[int, tuple[str, ...]] = {
    8587: ("standard", "full"),
    8591: ("standard", "full"),
    8595: ("full",),
}
# services this launcher does not manage at all (see the notes beside their entries)
UNMANAGED_PORTS: dict[int, str] = {
    8099: "not started by the launcher (app_v2 refuses a second instance)",
    8588: "not started by the launcher (no trm_server step)",
}

DEFAULT_TIMEOUT = 5.0


class Result:
    __slots__ = ("name", "port", "path", "marker", "ok", "detail", "ms")

    def __init__(self, name, port, path, marker):
        self.name = name
        self.port = port
        self.path = path
        self.marker = marker
        self.ok = False
        self.detail = ""
        self.ms = 0.0


def port_open(port: int, host: str, timeout: float) -> bool:
    try:
        with socket.create_connection((host, port), timeout=timeout):
            return True
    except OSError:
        return False


def probe(name, port, path, marker, expect_json, host, timeout) -> Result:
    r = Result(name, port, path, marker)
    if not port_open(port, host, timeout):
        r.detail = "connection refused"
        return r

    url = f"http://{host}:{port}{path}"
    started = time.perf_counter()
    try:
        with urllib.request.urlopen(url, timeout=timeout) as resp:
            code = resp.status
            body = resp.read(65536).decode("utf-8", "replace")
    except urllib.error.HTTPError as exc:
        r.ms = (time.perf_counter() - started) * 1000
        r.detail = f"HTTP {exc.code}"
        return r
    except (urllib.error.URLError, OSError, TimeoutError) as exc:
        r.ms = (time.perf_counter() - started) * 1000
        r.detail = f"request failed: {getattr(exc, 'reason', exc)}"
        return r

    r.ms = (time.perf_counter() - started) * 1000
    if code != 200:
        r.detail = f"HTTP {code}"
        return r
    if not body.strip():
        r.detail = "empty body"
        return r
    stripped = body.lstrip()
    if expect_json and not stripped.startswith("{"):
        # MEASURED 2026-11-03: Jarvis /health served <!doctype html> (the SPA
        # fallback) and this gate still passed, because the page title contains
        # the word "Jarvis". A marker substring is not proof of an API.
        first = stripped.split("\n", 1)[0][:48]
        r.detail = f"expected JSON, got {first!r}"
        return r
    if expect_json:
        try:
            json.loads(body)
        except ValueError:
            r.detail = "body is not valid JSON"
            return r
    if marker.lower() not in body.lower():
        r.detail = f"marker {marker!r} missing"
        return r

    r.ok = True
    r.detail = "ok"
    return r


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="HTTP readiness gate for the TRM platform.")
    ap.add_argument("--host", default="127.0.0.1", help="target host (default 127.0.0.1)")
    ap.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT, help="per-request timeout seconds")
    ap.add_argument("--only", default="", help="comma separated ports to test")
    ap.add_argument("--json", action="store_true", help="machine readable output")
    ap.add_argument("--profile", default=None,
                    help="model profile (default: $TRM_MODEL_PROFILE or lean); tiers the profile never starts are SKIPPED")
    args = ap.parse_args(argv)
    profile = (args.profile or os.environ.get("TRM_MODEL_PROFILE") or "lean").lower()

    services = SERVICES
    if args.only:
        wanted = {p.strip() for p in args.only.split(",") if p.strip()}
        services = [s for s in SERVICES if str(s[1]) in wanted]
        if not services:
            print(f"error: --only {args.only!r} matched no known port", file=sys.stderr)
            return 2

    results, skipped = [], []
    for name, port, path, marker, expect_json in services:
        if port in PROFILE_TIERS and profile not in PROFILE_TIERS[port]:
            skipped.append({"name": name, "port": port,
                            "why": f"not in profile '{profile}'"})
            continue
        if port in UNMANAGED_PORTS:
            skipped.append({"name": name, "port": port, "why": UNMANAGED_PORTS[port]})
            continue
        results.append(probe(name, port, path, marker, expect_json, args.host, args.timeout))

    failed = [r for r in results if not r.ok]

    if args.json:
        print(json.dumps(
            {
                "ok": not failed,
                "passed": len(results) - len(failed),
                "failed": len(failed),
                "skipped": skipped,
                "profile": profile,
                "services": [
                    {
                        "name": r.name, "port": r.port, "path": r.path,
                        "marker": r.marker, "ok": r.ok,
                        "detail": r.detail, "ms": round(r.ms, 1),
                    }
                    for r in results
                ],
            },
            indent=2,
        ))
        return 1 if failed else 0

    width = max(len(r.name) for r in results) + 2
    print()
    print("TRM Swarm OS - platform smoke test")
    print("=" * (width + 40))
    print(f"{'SERVICE':<{width}}{'PORT':<7}{'RESULT':<8}{'MS':>7}  DETAIL")
    print("-" * (width + 40))
    for r in results:
        status = "PASS" if r.ok else "FAIL"
        print(f"{r.name:<{width}}{r.port:<7}{status:<8}{r.ms:>7.0f}  {r.detail}")
    print("-" * (width + 40))
    print(f"{len(results) - len(failed)}/{len(results)} services healthy (profile '{profile}')")
    for s in skipped:
        print(f"  SKIP  {s['name']:<{width - 8}}:{s['port']:<7} {s['why']}")
    print()

    if failed:
        print("RESULT: FAIL")
        for r in failed:
            print(f"  - {r.name} (port {r.port}): {r.detail}")
        return 1

    print("RESULT: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
