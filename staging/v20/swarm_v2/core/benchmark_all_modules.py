#!/usr/bin/env python3
"""TRM Swarm OS v19 -- All-Module Benchmark (derived truth only).

Every number this harness reports is MEASURED at run time from a live source:
the derived module registry, the compiled kernels, the strip files' own bytes,
the fabric store, the ledger chain, and the running services. Nothing is taken
from a banner, a docstring, or a dashboard endpoint that answers with a literal.

The distinction this harness enforces: a probe is only evidence if the probe can
FAIL. The kernel answers 200 with {"status":"ok"} for every path, including a
path that does not exist -- so "the dashboard returned 200" is not evidence of
anything. The service probe therefore reads the catch-all body from a nonsense
path FIRST, and marks any route answering with that exact body as NOT
IMPLEMENTED rather than OK.

Usage:
    python swarm_v2/core/benchmark_all_modules.py [--fast] [--json OUT]

    --fast   skip the cargo builds (kernels are reported as NOT RUN, not OK)
"""
from __future__ import annotations

import argparse
import ctypes
import glob
import hashlib
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request

_Path = __import__("pathlib").Path
ROOT = _Path(__file__).resolve().parents[2]
STRIPS = ROOT / "swarm_v2_artifacts" / "native_voltage_strips"
LEDGER = ROOT / "swarm_v2_artifacts" / "monad_ledger.jsonl"
STORE = ROOT / "swarm_v2_nand_fabric.k3"
K3_LEGAL = (0b00, 0b01, 0b11)  # the platform's own rule: 0b10 is never legal

RESULTS: dict = {"root": str(ROOT), "started": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}


def _sec(title):
    print("\n" + "-" * 78)
    print(title)
    print("-" * 78)


# ── 1. module inventory (derived registry) ───────────────────────────────────
def bench_modules() -> dict:
    _sec("1. MODULE INVENTORY  (source: module_registry.json, generated)")
    p = ROOT / "module_registry.json"
    if not p.exists():
        print("   registry ABSENT -- cannot verify modules")
        return {"present": False}
    reg = json.loads(p.read_text(encoding="utf-8"))
    km = reg.get("kernel_modules") or []
    by = reg.get("by_status") or {}
    out = {"present": True, "generated": reg.get("generated"),
           "kernel_modules": len(km), "platform_modules": reg.get("summary", {}).get("platform_modules"),
           "by_status": by,
           "source_missing": [], "stub_modules": [], "fn_total": 0}

    # Modules live in TWO crates. Resolving `source` only under one of them reported
    # 168 of 182 modules as "missing source" when their files were present in the
    # other -- a false finding produced by the ruler, not by the modules.
    srcroots = [ROOT / "swarm_v2_rust_kernel" / "src", ROOT / "trm_uefi_kernel" / "src"]
    for m in km:
        src = (m.get("source") or "")
        name = m.get("name")
        fn = m.get("functions", 0)
        out["fn_total"] += fn
        if src.startswith("kernel:"):
            rel = src.split(":", 1)[1]
            f = next((r / rel for r in srcroots if (r / rel).exists()), None)
            if f is None:
                out["source_missing"].append(name)
                continue
            body = f.read_text(encoding="utf-8", errors="replace")
            real = len(re.findall(r"^\s*pub\s+fn\s+\w+", body, re.M))
            if real == 0:
                out["stub_modules"].append(name)
            out.setdefault("declared_fn", 0)
            out["declared_fn"] += real
    print("   kernel modules     : %d" % out["kernel_modules"])
    print("   by status          : %s" % json.dumps(by))
    print("   registry 'functions' sum : %d" % out["fn_total"])
    print("   pub fn found in source   : %d" % out.get("declared_fn", 0))
    print("   source files missing     : %d %s" % (len(out["source_missing"]), out["source_missing"][:6]))
    print("   modules with 0 pub fn    : %d %s" % (len(out["stub_modules"]), out["stub_modules"][:6]))
    return out


# ── 2. voltage strips (measure the bytes, then try to LOAD them) ─────────────
def bench_strips() -> dict:
    _sec("2. VOLTAGE STRIPS  (measured: file bytes + real LoadLibrary)")
    files = sorted(glob.glob(str(STRIPS / "*.dll")))
    pe, marker, other, loaded, load_err = [], [], [], [], {}
    for f in files:
        head = open(f, "rb").read(90)
        if head[:2] == b"MZ":
            pe.append(f)
        elif head[:8].startswith(b"TRMCDYLI"):
            marker.append(f)
        else:
            other.append(f)
    for f in pe:
        try:
            ctypes.WinDLL(f)
            loaded.append(os.path.basename(f))
        except OSError as e:
            load_err[os.path.basename(f)] = str(e)
    sizes = sorted(os.path.getsize(f) for f in pe) or [0]
    out = {"total": len(files), "pe": len(pe), "marker": len(marker), "other": len(other),
           "loaded_ok": len(loaded), "load_failed": len(load_err),
           "pe_bytes_min": sizes[0], "pe_bytes_max": sizes[-1],
           "load_errors": dict(list(load_err.items())[:5])}
    print("   total .dll        : %d" % out["total"])
    print("   real PE (MZ)      : %d" % out["pe"])
    print("   marker (TRMCDYLI) : %d   <- source containers, not loadable images" % out["marker"])
    print("   other             : %d" % out["other"])
    print("   PE that ACTUALLY LOADED : %d / %d" % (out["loaded_ok"], out["pe"]))
    if load_err:
        print("   load failures     : %d  e.g. %s" % (len(load_err), list(load_err.items())[:2]))
    return out


# ── 3. kernels (cargo) ───────────────────────────────────────────────────────
def _cargo(cwd, args, timeout=1500):
    try:
        r = subprocess.run(["cargo"] + args, cwd=str(cwd), capture_output=True,
                           text=True, timeout=timeout, errors="replace")
        return r.returncode, (r.stdout or "") + (r.stderr or "")
    except Exception as e:
        return None, "RUN ERROR: %s" % e


def bench_kernels(fast: bool) -> dict:
    _sec("3. KERNELS  (cargo -- real compile + real test run)")
    out = {}
    for name, sub in (("swarm_v2_rust_kernel", "swarm_v2_rust_kernel"), ("trm_uefi_kernel", "trm_uefi_kernel")):
        d = ROOT / sub
        if not d.exists():
            out[name] = {"present": False}
            print("   %-22s ABSENT" % name)
            continue
        if fast:
            out[name] = {"present": True, "skipped": True}
            print("   %-22s NOT RUN (--fast)" % name)
            continue
        t0 = time.time()
        rc, log = _cargo(d, ["test", "--release", "--quiet"], timeout=1800)
        passed = sum(int(x) for x in re.findall(r"(\d+) passed", log))
        failed = sum(int(x) for x in re.findall(r"(\d+) failed", log))
        out[name] = {"present": True, "rc": rc, "seconds": round(time.time() - t0, 1),
                     "passed": passed, "failed": failed}
        print("   %-22s rc=%s  passed=%d failed=%d  (%.1fs)" % (name, rc, passed, failed, time.time() - t0))
        if rc not in (0, None):
            # A no_std bin cannot host a test harness: the harness injects its own
            # panic_impl, so `cargo test` stops at E0152 having run nothing. That is
            # not a kernel failure. Verify it the way a no_std kernel CAN be verified
            # (a real compile in the target profile) and say which check actually ran.
            if "duplicate lang item `panic_impl`" in log:
                t1 = time.time()
                rc2, log2 = _cargo(d, ["check", "--release", "--quiet"], timeout=1800)
                out[name].update({"testable": False, "check_rc": rc2,
                                  "check_seconds": round(time.time() - t1, 1)})
                print("        NOT TESTABLE  (no_std bin: harness injects a 2nd panic_impl)")
                print("        cargo check --release : rc=%s  (%.1fs)   <- the check that CAN run"
                      % (rc2, time.time() - t1))
                if rc2 not in (0, None):
                    for ln in [l for l in log2.splitlines() if re.match(r"\s*error(\[|:)", l)][:4]:
                        print("        %s" % ln.strip()[:110])
                continue
            # Match real diagnostics only: "Error" as a substring also matches
            # warnings like `unused import: error::ParserError`, which previously
            # filled this report with warnings while the real error went unshown.
            for ln in [l for l in log.splitlines() if re.match(r"\s*error(\[|:)", l)][:4]:
                print("        %s" % ln.strip()[:110])
    return out


# ── 4. python modules compile ────────────────────────────────────────────────
def bench_python() -> dict:
    _sec("4. PYTHON MODULES  (real py_compile, per file)")
    skip = {".git", "target", "node_modules", "__pycache__", ".venv", "venv", "dist", "build"}
    ok, bad = 0, []
    for dp, dn, fn in os.walk(ROOT):
        dn[:] = [d for d in dn if d not in skip]
        for f in fn:
            if not f.endswith(".py"):
                continue
            p = os.path.join(dp, f)
            r = subprocess.run([sys.executable, "-m", "py_compile", p], capture_output=True)
            if r.returncode:
                bad.append(os.path.relpath(p, ROOT).replace("\\", "/"))
            else:
                ok += 1
    print("   compiled OK       : %d" % ok)
    print("   FAILED            : %d" % len(bad))
    print("   (pre-existing py2/snapshot fragments are expected here; they are")
    print("    reported, never silently assumed clean)")
    if bad:
        print("   first failures    : %s" % bad[:5])
    # Record EVERY failure, not the first 40: a list capped silently leaves the
    # reader unable to reconcile `failed` against the names, and the tree is LIVE
    # (files under runtime/ can be mid-write), so the count is not stable run to run.
    return {"ok": ok, "failed": len(bad), "failed_files": sorted(bad)}


# ── 5. fabric + ledger ───────────────────────────────────────────────────────
def bench_fabric() -> dict:
    _sec("5. NAND FABRIC + LEDGER  (measured on the store itself)")
    out = {}
    if STORE.exists():
        raw = STORE.read_bytes()
        illegal = 0
        for b in raw:
            for nib in ((b >> 4) & 3, b & 3):
                if nib not in K3_LEGAL:
                    illegal += 1
        out.update({"store_bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(),
                    "illegal_nibbles": illegal,
                    "illegal_pct": round(100.0 * illegal / (2 * len(raw)), 6),
                    "has_turbovec_header": raw[:10] == b"TURBOVEC\0\0"})
        print("   store             : %d bytes" % out["store_bytes"])
        print("   sha256            : %s" % out["sha256"][:32])
        print("   illegal nibbles   : %d (%.6f%%) under the platform's own K3 rule" %
              (out["illegal_nibbles"], out["illegal_pct"]))
        print("   TURBOVEC header   : %s" % out["has_turbovec_header"])
    else:
        print("   store ABSENT: %s" % STORE)

    if LEDGER.exists():
        rows = [json.loads(l) for l in LEDGER.read_text(encoding="utf-8").splitlines() if l.strip()]
        def canon(b):
            return json.dumps({k: v for k, v in b.items() if k != "hash"},
                              separators=(",", ":"), sort_keys=True).encode("utf-8")
        bad = [r.get("index") for i, r in enumerate(rows)
               if hashlib.sha256(canon(r)).hexdigest() != r.get("hash")
               or (i and r.get("prev_hash") != rows[i - 1].get("hash"))]
        out.update({"ledger_blocks": len(rows), "ledger_chain_ok": not bad, "ledger_bad": bad[:5],
                    "ledger_tip": rows[-1].get("hash", "")[:16] if rows else ""})
        print("   ledger blocks     : %d" % len(rows))
        print("   chain verifies    : %s %s" % (not bad, bad[:5]))
        print("   tip               : %s" % out["ledger_tip"])
        print("   last anchor       : %s" % (rows[-1].get("anchor") if rows else "?"))
    else:
        print("   ledger ABSENT")
    return out


# ── 6. live services ─────────────────────────────────────────────────────────
def _get(url, timeout=6):
    try:
        r = urllib.request.urlopen(url, timeout=timeout)
        return r.status, r.read()
    except urllib.error.HTTPError as e:
        return e.code, e.read()
    except Exception as e:
        return None, str(e).encode()


ROUTES = ["/health", "/system", "/council/status", "/council/proposals", "/swarm/experts",
          "/swarm/telemetry", "/modules", "/mesh", "/memory", "/federation", "/kanban",
          "/brain/status", "/research/stats", "/infrastructure", "/state", "/evolution/proposals"]


def bench_services() -> dict:
    _sec("6. LIVE SERVICES  (probe must be able to FAIL to count)")
    out = {}
    base = "http://127.0.0.1:8022"
    _, catchall = _get(base + "/zzz-not-a-route")
    out["catchall_body"] = catchall.decode("utf-8", "replace")[:120]
    print("   kernel :8022 catch-all body : %s" % out["catchall_body"].strip())

    real, fake, dead = [], [], []
    for p in ROUTES:
        code, body = _get(base + p)
        if code is None:
            dead.append(p)
        elif body == catchall:
            fake.append(p)
        else:
            real.append(p)
    out.update({"kernel_addr": base, "routes_real": real, "routes_catchall": fake, "routes_dead": dead})
    print("   routes NOT implemented (answered by catch-all) : %d %s" % (len(fake), fake))
    print("   routes with own body                           : %d %s" % (len(real), real))
    if dead:
        print("   dead                                           : %d %s" % (len(dead), dead))

    code, body = _get("http://127.0.0.1:5183/", timeout=8)
    out["lcars"] = {"up": code == 200, "bytes": len(body)}
    print("   LCARS  :5183 : up=%s (%d bytes)" % (code == 200, len(body)))

    models = []
    code, body = _get("http://127.0.0.1:11434/api/tags", timeout=8)
    if code == 200:
        models = json.loads(body).get("models", [])
    out["ollama_models"] = len(models)
    print("   Ollama :11434 : %d models available" % len(models))
    return out


def bench_inference() -> dict:
    """Real inference timing on the smallest local model -- a derived number."""
    _sec("7. INFERENCE (real request, real timing)")
    code, body = _get("http://127.0.0.1:11434/api/tags", timeout=8)
    if code != 200 or not body:
        print("   ollama unreachable -- inference NOT RUN")
        return {"ran": False}
    allm = json.loads(body).get("models", [])
    # A cloud-tagged entry (`x:cloud`, `gemma3:27b-cloud`) is a REMOTE model with no
    # local weights. It reports a tiny size, so sorting by size picks it first -- and
    # a generate request to it fails (410 Gone), which meant local inference was
    # never actually tested. Match the tag anywhere, not just a ":cloud" suffix.
    local = [m for m in allm
             if "cloud" not in m.get("name", "").lower() and (m.get("size") or 0) > 0]
    ms = sorted(local, key=lambda m: m.get("size", 1 << 62))
    if not ms:
        print("   no LOCAL model on this host (%d entries, all remote) -- NOT RUN" % len(allm))
        return {"ran": False, "remote_models": len(allm)}
    # The smallest local weights are often EMBEDDING models (`all-minilm`), which have
    # no /api/generate endpoint and answer 400 Bad Request. Walk candidates
    # smallest-first and use the first that actually generates, reporting which one
    # did: a benchmark that stops at the first failure measures the failure.
    r, name, tried, chosen = None, None, [], None
    for cand in ms[:6]:
        nm = cand["name"]
        tried.append(nm)
        payload = json.dumps({"model": nm, "prompt": "Reply with exactly: OK",
                              "stream": False, "options": {"num_predict": 16}}).encode()
        req = urllib.request.Request("http://127.0.0.1:11434/api/generate", data=payload,
                                     headers={"Content-Type": "application/json"})
        t0 = time.time()
        try:
            r = json.loads(urllib.request.urlopen(req, timeout=120).read())
            name, chosen = nm, cand
            break
        except Exception as e:
            print("   %-22s skipped: %s" % (nm, e))
    if name is None:
        print("   no local model generated (tried %d)" % len(tried))
        return {"ran": False, "tried": tried}
    wall = time.time() - t0
    ev = r.get("eval_count") or 0
    evd = (r.get("eval_duration") or 0) / 1e9
    # A rate over 3 tokens is NOISE, not a measurement: this same check returned
    # 52.73 and 162.61 tok/s on two runs of the same host. The short "OK" reply stays
    # as the liveness proof; issue a SECOND request forced to generate a real sample
    # and report n beside the rate so a small sample cannot masquerade as throughput.
    rate, n, secs = (ev / evd if evd > 0 else None), ev, evd
    payload2 = json.dumps({"model": name,
                           "prompt": "Count from 1 to 60 in digits, space separated, nothing else.",
                           "stream": False, "options": {"num_predict": 128}}).encode()
    try:
        r2 = json.loads(urllib.request.urlopen(urllib.request.Request(
            "http://127.0.0.1:11434/api/generate", data=payload2,
            headers={"Content-Type": "application/json"}), timeout=180).read())
        ev2 = r2.get("eval_count") or 0
        evd2 = (r2.get("eval_duration") or 0) / 1e9
        if ev2 >= 16 and evd2 > 0:
            rate, n, secs = ev2 / evd2, ev2, evd2
    except Exception as e:
        print("   throughput probe failed (%s) -- reporting the short sample" % e)
    out = {"ran": True, "model": name, "size": chosen.get("size"),
           "wall_s": round(wall, 3), "eval_count": n, "eval_s": round(secs, 3),
           "tok_per_s": round(rate, 2) if rate else None,
           "sample_ok": n >= 16,
           "reply": (r.get("response") or "").strip()[:40]}
    print("   model             : %s (%.1f MB)" % (name, (out["size"] or 0) / 1e6))
    print("   wall (liveness)   : %.3fs" % out["wall_s"])
    print("   reply             : %r" % out["reply"])
    print("   throughput sample : %d tokens in %.3fs eval%s"
          % (n, secs, "" if out["sample_ok"] else "   <- TOO SMALL to be a rate"))
    print("   throughput        : %s tok/s  (DERIVED: eval_count / eval_duration)"
          % out["tok_per_s"])
    return out


# ── main ─────────────────────────────────────────────────────────────────────
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--fast", action="store_true", help="skip cargo builds")
    ap.add_argument("--json", default=str(ROOT / "swarm_v2_artifacts" / "bench_all_modules.json"))
    a = ap.parse_args()

    print("=" * 78)
    print("TRM SWARM OS v19 -- ALL-MODULE BENCHMARK")
    print("root: %s" % ROOT)
    print("=" * 78)
    RESULTS["modules"] = bench_modules()
    RESULTS["strips"] = bench_strips()
    RESULTS["kernels"] = bench_kernels(a.fast)
    RESULTS["python"] = bench_python()
    RESULTS["fabric"] = bench_fabric()
    RESULTS["services"] = bench_services()
    RESULTS["inference"] = bench_inference()
    RESULTS["finished"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())

    _sec("SUMMARY")
    m, s, f, py, sv = (RESULTS["modules"], RESULTS["strips"], RESULTS["fabric"],
                       RESULTS["python"], RESULTS["services"])
    print("   modules declared        : %s" % m.get("kernel_modules"))
    print("   modules with 0 pub fn   : %s" % len(m.get("stub_modules", [])))
    print("   strips loadable         : %s / %s PE" % (s.get("loaded_ok"), s.get("pe")))
    print("   python compiles         : %s ok / %s failed" % (py.get("ok"), py.get("failed")))
    print("   store illegal nibbles   : %s" % f.get("illegal_nibbles"))
    print("   ledger chain            : %s (%s blocks)" % (f.get("ledger_chain_ok"), f.get("ledger_blocks")))
    print("   kernel routes real      : %s / %s probed" % (len(sv.get("routes_real", [])), len(ROUTES)))
    print("   inference tok/s         : %s" % RESULTS["inference"].get("tok_per_s"))
    os.makedirs(os.path.dirname(a.json), exist_ok=True)
    with open(a.json, "w", encoding="utf-8") as fh:
        json.dump(RESULTS, fh, indent=2)
    print("\n   written: %s" % a.json)
    return 0


if __name__ == "__main__":
    sys.exit(main())
