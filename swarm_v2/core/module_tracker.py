"""Live module-load instrumentation.

The static trace (tools/trace_codebase.py) can only say whether a module is
*reachable* from an entrypoint. It cannot say whether a running process ever
imported it, when, or in what order. This module answers that by installing a
``sys.meta_path`` finder at app startup that stamps every module imported from
this repository with:

  * a monotonic ``import_index`` (0, 1, 2, ... in load order)
  * ``import_ms`` — wall-clock milliseconds since process creation
  * ``phase`` — ``startup`` (before the ASGI lifespan finished starting) or
    ``lazy`` (first imported later, e.g. on the first request that needs it)

Only modules whose ``__file__`` resolves inside the repository are stamped, so
the finder does not wrap loaders for the standard library or site-packages.

Usage (see swarm_v2/app_v2.py):

    from swarm_v2.core.module_tracker import install, mark_startup_complete
    install()                      # as early as possible
    ...
    mark_startup_complete()        # at the end of the lifespan startup phase
"""
from __future__ import annotations

import os
import sys
import threading
import time
from pathlib import Path
from typing import Any, Dict, List, Optional

# swarm_v2/core/module_tracker.py -> repo root
REPO_ROOT = Path(__file__).resolve().parents[2]
SWARM_ROOT = REPO_ROOT / "swarm_v2"

_lock = threading.RLock()
_records: Dict[str, Dict[str, Any]] = {}
_counter = 0
_installed = False
_startup_done = False
_process_start_epoch: Optional[float] = None

# G13 fix (2026-11-10): snapshot() walked all of sys.modules on EVERY request,
# calling Path.resolve() and .is_file() (a stat) per module, and summary() called
# snapshot() a second time — measured 758 ms + 757 ms = 1,516 ms of work per call
# in-process, and 10.6 s on the live route. Two caches fix it without changing the
# contract: memoise the repo-path decision per module key, and throttle the rescan
# to once per interval or when sys.modules actually changes size.
_MISSING = object()
_path_cache: Dict[str, Optional[str]] = {}
_last_scan_mono: float = 0.0
_scan_module_count: int = -1
SCAN_INTERVAL_S = 1.0


def _process_start_epoch_value() -> float:
    """Epoch seconds at which this process was created."""
    global _process_start_epoch
    if _process_start_epoch is None:
        try:
            import psutil

            _process_start_epoch = psutil.Process(os.getpid()).create_time()
        except Exception:
            # Fall back to "now minus uptime-ish": monotonic origin is unknown,
            # so treat instrumentation time as the process start.
            _process_start_epoch = time.time()
    return _process_start_epoch


def _offset_ms() -> float:
    return max(0.0, (time.time() - _process_start_epoch_value()) * 1000.0)


def _repo_path(module: Any) -> Optional[str]:
    """Absolute path of a module's source file if it lives in this repo."""
    f = getattr(module, "__file__", None)
    if not f:
        return None
    try:
        p = Path(f).resolve()
    except Exception:
        return None
    try:
        p.relative_to(REPO_ROOT)
    except ValueError:
        return None
    # Exclude pseudo-modules (__main__ from a REPL/stdin, frozen shims) whose
    # __file__ resolves into the cwd but is not a real source file.
    if not p.is_file():
        return None
    # __mp_main__ is the multiprocessing/uvicorn reload alias of the entry
    # module: it carries the same __file__ as __main__, so it appears as a
    # second "module" and inflates the count. It is not a distinct source file.
    if module.__dict__.get("__name__") == "__mp_main__":
        return None
    return str(p)


def _stamp(name: str, path: str, exact: bool) -> None:
    global _counter
    with _lock:
        if name in _records:
            return
        _records[name] = {
            "name": name,
            "file": path,
            "under_swarm_v2": _is_under_swarm(path),
            "import_index": _counter,
            "import_ms": round(_offset_ms(), 3),
            "phase": "lazy" if _startup_done else "startup",
            "import_ms_exact": exact,
        }
        _counter += 1


def _is_under_swarm(path: str) -> bool:
    try:
        Path(path).relative_to(SWARM_ROOT)
        return True
    except ValueError:
        return False


class _StampingLoader:
    """Delegating loader that stamps the module after its code has executed."""

    __slots__ = ("_loader", "_name")

    def __init__(self, loader: Any, name: str):
        self._loader = loader
        self._name = name

    def create_module(self, spec):
        return self._loader.create_module(spec)

    def exec_module(self, module):
        self._loader.exec_module(module)
        path = _repo_path(module)
        if path:
            _stamp(self._name, path, exact=True)

    def __getattr__(self, item):
        return getattr(self._loader, item)


class ModuleLoadFinder:
    """First meta_path finder: delegates, then wraps repo loaders to stamp."""

    def find_spec(self, fullname, path=None, target=None):
        for finder in sys.meta_path:
            if finder is self:
                continue
            find_spec = getattr(finder, "find_spec", None)
            if find_spec is None:
                continue
            try:
                spec = find_spec(fullname, path, target)
            except Exception:
                continue
            if spec is None:
                continue
            origin = getattr(spec, "origin", None)
            loader = getattr(spec, "loader", None)
            if origin and loader is not None and hasattr(loader, "exec_module"):
                try:
                    Path(origin).resolve().relative_to(REPO_ROOT)
                except (ValueError, OSError):
                    return spec
                spec.loader = _StampingLoader(loader, fullname)
            return spec
        return None

    def invalidate_caches(self):
        pass


def install() -> None:
    """Install the finder and backfill modules already in sys.modules."""
    global _installed
    with _lock:
        if _installed:
            return
        _process_start_epoch_value()
        # Backfill: everything already imported is, by definition, startup.
        for name, module in list(sys.modules.items()):
            if module is None:
                continue
            path = _repo_path(module)
            if path:
                _stamp(name, path, exact=False)
        sys.meta_path.insert(0, ModuleLoadFinder())
        _installed = True


def mark_startup_complete() -> None:
    """Everything imported from now on is recorded as phase='lazy'."""
    global _startup_done
    _startup_done = True


def is_startup_complete() -> bool:
    return _startup_done


def snapshot(force_scan: bool = False) -> List[Dict[str, Any]]:
    """Current live view, ordered by import index.

    Re-scans sys.modules so modules imported through a path the finder did not
    see (e.g. a loader without exec_module) still appear, stamped at the end.
    The rescan is throttled (see SCAN_INTERVAL_S) and per-module path decisions
    are memoised, because the full walk costs ~750 ms and this backs an HTTP
    route.
    """
    global _last_scan_mono, _scan_module_count
    with _lock:
        now = time.monotonic()
        changed = len(sys.modules) != _scan_module_count
        if force_scan or changed or (now - _last_scan_mono) >= SCAN_INTERVAL_S:
            _last_scan_mono = now
            _scan_module_count = len(sys.modules)
            for name, module in list(sys.modules.items()):
                if module is None or name in _records:
                    continue
                # __mp_main__ is multiprocessing's alias of the entry module. It has
                # its own sys.modules key but the SAME __file__ as __main__ and its
                # __dict__["__name__"] is "__main__", so filtering on the module
                # attribute silently misses it -- the key is the only reliable
                # signal. One source file must not be counted as two modules.
                if name == "__mp_main__":
                    continue
                path = _path_cache.get(name, _MISSING)
                if path is _MISSING:
                    path = _repo_path(module)
                    _path_cache[name] = path
                if path:
                    _stamp(name, path, exact=False)
        return [dict(_records[n]) for n in sorted(_records, key=lambda n: _records[n]["import_index"])]


def summary(mods: Optional[List[Dict[str, Any]]] = None) -> Dict[str, Any]:
    if mods is None:
        mods = snapshot()
    return {
        "count": len(mods),
        "startup_count": sum(1 for m in mods if m["phase"] == "startup"),
        "lazy_count": sum(1 for m in mods if m["phase"] == "lazy"),
        "swarm_v2_count": sum(1 for m in mods if m["under_swarm_v2"]),
        "repo_root": str(REPO_ROOT),
        "process_start_epoch": _process_start_epoch_value(),
        "startup_complete": _startup_done,
        "instrumented": _installed,
    }
