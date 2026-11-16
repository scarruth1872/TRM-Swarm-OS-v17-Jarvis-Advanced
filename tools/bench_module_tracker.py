"""Benchmark the live module inventory that GET /system/modules exposes.

G13 measured 10.6 s for that route on an idle service. The route calls
module_tracker.snapshot() and then module_tracker.summary(), and summary() calls
snapshot() again, so every request walks sys.modules twice, calling Path.resolve()
and .is_file() (a stat) on every module not yet recorded.

This measures the function cost directly, so the fix can be shown as a number
rather than asserted. Run it before and after the change.

Usage: python tools/bench_module_tracker.py [--runs 5]
"""
import argparse
import statistics
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

import swarm_v2.app_v2  # noqa: E402,F401  (import the app so sys.modules is realistic)
from swarm_v2.core import module_tracker  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", type=int, default=5)
    args = ap.parse_args()

    module_tracker.install()
    n_mods = len(sys.modules)

    snap_t, sum_t, route_t = [], [], []
    for _ in range(args.runs):
        t0 = time.perf_counter()
        mods = module_tracker.snapshot()
        snap_t.append((time.perf_counter() - t0) * 1000)

        t0 = time.perf_counter()
        module_tracker.summary()
        sum_t.append((time.perf_counter() - t0) * 1000)

        # what the route actually does
        t0 = time.perf_counter()
        m = module_tracker.snapshot()
        _ = {"modules": m, "count": len(m),
             **{k: v for k, v in module_tracker.summary().items() if k != "count"}}
        route_t.append((time.perf_counter() - t0) * 1000)

    print(f"sys.modules entries: {n_mods}")
    print(f"tracked repo modules: {len(mods)}  (startup {sum(1 for m in mods if m['phase'] == 'startup')}, "
          f"lazy {sum(1 for m in mods if m['phase'] == 'lazy')})")
    for name, series in (("snapshot()", snap_t), ("summary()", sum_t), ("route work", route_t)):
        print(f"{name:12s} median {statistics.median(series):8.1f} ms   "
              f"min {min(series):8.1f}  max {max(series):8.1f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
