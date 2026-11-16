#!/usr/bin/env python3
"""Run the platform test suite and record the REAL result.

WHY THIS EXISTS
/testing/stats served a hardcoded fixture: 60 tests, 60 passed, 0 failed,
"accuracy_percent": 100.0, "coverage": "100.0%", with six run records all
timestamped 2026-09-03T13:36:06Z. The real suite is 31 tests. The fixture had no
origin anywhere - not in the repo, not in the WSL home, not in the antigravity
brain - and no run artefact existed on disk to support any number in it.

This writes what a test run actually produced, so /testing/stats has something
true to serve and says so plainly when there is nothing on record.

USAGE
    python tools/run_platform_tests.py [--out evidence/platform_test_results.json]
                                    [--path tests/]
Exit code is the pytest exit code: 0 all passed, non-zero otherwise. A non-zero
exit is DATA, not a failure of this script - it is recorded and reported.
"""
import argparse, json, os, subprocess, sys, time, datetime, shutil


def find_python():
    """Prefer the interpreter that actually has the platform's test deps."""
    for cand in [
        r"C:\Users\shawn\AppData\Local\Programs\Python\Python311\python.exe",
        shutil.which("python"), shutil.which("python3"),
    ]:
        if cand and os.path.exists(cand):
            return cand
    return sys.executable


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=r"evidence/platform_test_results.json")
    ap.add_argument("--path", default=".",
                    help="default . so pytest.ini testpaths + addopts apply; pass tests/ for the unit subset")
    ap.add_argument("--root", default=".")
    a = ap.parse_args()
    py = find_python()
    # When no path is given pytest honours testpaths from pytest.ini. Passing "."
    # OVERRIDES testpaths and collects the whole tree - including standalone scripts
    # that dial services at import (they raised 154 collection errors and aborted the
    # run). So "." means "no path argument".
    base = [py, "-m", "pytest"]
    if a.path and a.path != ".":
        base.append(a.path)
    cmd = base + ["-q", "--no-header", "-p", "no:cacheprovider",
                  "--timeout=25", "--timeout-method=thread"]
    started = datetime.datetime.now(datetime.timezone.utc)
    t0 = time.time()
    r = subprocess.run(cmd, cwd=a.root, capture_output=True, text=True, timeout=1800)
    dur = time.time() - t0
    out = (r.stdout or "") + (r.stderr or "")

    passed = failed = errors = skipped = collected = None
    import re
    m = re.search(r"(\d+) passed", out)
    if m: passed = int(m.group(1))
    m = re.search(r"(\d+) failed", out)
    if m: failed = int(m.group(1))
    m = re.search(r"(\d+) error", out)
    if m: errors = int(m.group(1))
    m = re.search(r"(\d+) skipped", out)
    if m: skipped = int(m.group(1))
    if passed is not None:
        collected = (passed or 0) + (failed or 0) + (errors or 0) + (skipped or 0)

    rec = {
        "run_at_utc": started.isoformat(timespec="seconds"),
        "duration_seconds": round(dur, 3),
        "command": " ".join(cmd),
        "working_dir": os.path.abspath(a.root),
        "path": a.path,
        "python": py,
        "exit_code": r.returncode,
        "collected": collected,
        "passed": passed if passed is not None else 0,
        "failed": failed if failed is not None else 0,
        "errors": errors if errors is not None else 0,
        "skipped": skipped if skipped is not None else 0,
        "status": ("ALL_PASSED" if r.returncode == 0 and not (failed or errors)
                   else ("FAILURES" if (failed or errors) else "UNKNOWN")),
        # the last lines of real output, so the number is auditable
        "tail": "\n".join(out.strip().splitlines()[-6:]),
    }
    dst = os.path.join(a.root, a.out) if not os.path.isabs(a.out) else a.out
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    with open(dst, "w", encoding="utf-8") as f:
        json.dump(rec, f, indent=2)
    print(json.dumps(rec, indent=2))
    return r.returncode


if __name__ == "__main__":
    sys.exit(main())
