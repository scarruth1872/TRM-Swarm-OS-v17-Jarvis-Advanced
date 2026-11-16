#!/usr/bin/env python3
"""hw_verify.py — what changed after a BIOS change, measured instead of assumed.

Run this AFTER enabling DOCP and Core Performance Boost/PBO, and compare to the recorded baseline:

    BEFORE (2026-11-13, BIOS 1804, DOCP off, CPB off):
        Memory Clock      1,200 MHz   (DDR4-2400)      FCLK 1,200   UCLK 1,200
        Core clock under 12-proc load: pinned 4,000.0 MHz, 108 W package, Tctl 74.9 C
        Frequency Limit - Global:      4,440-4,625 MHz (unreachable)
        CPU 2B ternary: prefill 48.9 tok/s (pool t6/par6/ub512), decode 23.2 tok/s

    EXPECTED AFTER (not claimed until measured):
        Memory Clock      1,600 MHz   (DDR4-3200)      FCLK 1,600   UCLK 1,600
        Core clock under load: above 4,000 MHz (up to the 4,440-4,625 limit)
        +11..16 % single-thread ceiling; memory bandwidth headroom +33 % (not 1:1)

CHECKS
  1. rails and clocks from the HWiNFO CSV        (no model needed)
  2. 12-process load: does the core clock exceed 4,000 MHz?
  3. CPU tier prefill/decode on :8620 if it is running
  4. WHEA / machine-check events since boot       (instability after an OC)
"""
from __future__ import annotations

import argparse
import json
import subprocess
import time
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
DEFAULT_CSV = Path("C:/Users/shawn/AppData/Local/hermes/cache/scratch/hwi/hwinfo_live.csv")
PY = r"C:\Users\shawn\AppData\Local\Programs\Python\Python311\python.exe"
BURN = ("import time\nx=0.0\nt=time.time()\nwhile time.time()-t<20:\n"
        "    for i in range(500000): x+=(i*1.000001)**0.5\n")
LONG = "Audit this module. One line.\n" + (
    "pub fn handler(ctx: &Ctx) -> Result<u8> { let mut acc = 0u32; for slot in ctx.slots { "
    "acc = acc.wrapping_add(slot.trit as u32); } Ok(acc as u8) }\n" * 11)


def latest(csv: Path, max_age_s: float = 120.0) -> dict:
    """Read the newest data row of the newest table.

    TWO MEASURED DEFECTS, both of which produced a confident FAIL from nothing:
      1. HWiNFO can append a SECOND header block to the same CSV (it did here at 14:54). Reading lines[1:] from
         the first header then picks up the newer header row as if it were data - every field came back as a
         group label ("Memory Timings", "CPU [#0]: ...") and nothing parsed.
      2. A frozen CSV is not a measurement. If nothing has written to the file recently, the numbers are
         history, so they are reported as UNMEASURED rather than scored.
    """
    lines = csv.read_text(encoding="utf-8", errors="replace").splitlines()
    sentinel = "Memory Clock [MHz]"
    header_idx = [i for i, l in enumerate(lines) if sentinel in l]
    if not header_idx:
        return {k: None for k in ("Memory Clock [MHz]", "Infinity Fabric Clock (FCLK) [MHz]",
                                  "Memory Controller Clock (UCLK) [MHz]", "Core Clocks (avg) [MHz]",
                                  "Core Effective Clocks (avg) [MHz]", "CPU Package Power [W]",
                                  "CPU (Tctl/Tdie)", "CPU Core Voltage (SVI2 TFN) [V]",
                                  "Frequency Limit - Global [MHz]", "Bus Clock [MHz]")} | {"_status": "no header"}
    hi = header_idx[-1]
    hdr = [h.strip().strip('"') for h in lines[hi].split(",")]
    rows = [l for l in lines[hi + 1:] if l.strip() and sentinel not in l]
    if not rows:
        return {k: None for k in ("Memory Clock [MHz]", "Infinity Fabric Clock (FCLK) [MHz]",
                                  "Memory Controller Clock (UCLK) [MHz]", "Core Clocks (avg) [MHz]",
                                  "Core Effective Clocks (avg) [MHz]", "CPU Package Power [W]",
                                  "CPU (Tctl/Tdie)", "CPU Core Voltage (SVI2 TFN) [V]",
                                  "Frequency Limit - Global [MHz]", "Bus Clock [MHz]")} | {"_status": "no data"}
    age = time.time() - csv.stat().st_mtime
    vals = [v.strip().strip('"') for v in rows[-1].split(",")]

    def g(name):
        i = next((k for k, h in enumerate(hdr) if name in h), None)
        try:
            return float(vals[i]) if i is not None else None
        except (ValueError, IndexError):
            return None
    rec = {k: g(k) for k in ("Memory Clock [MHz]", "Infinity Fabric Clock (FCLK) [MHz]",
                              "Memory Controller Clock (UCLK) [MHz]", "Core Clocks (avg) [MHz]",
                              "Core Effective Clocks (avg) [MHz]", "CPU Package Power [W]",
                              "CPU (Tctl/Tdie)", "CPU Core Voltage (SVI2 TFN) [V]",
                              "Frequency Limit - Global [MHz]", "Bus Clock [MHz]")}
    rec["_status"] = "live" if age <= max_age_s else f"stale ({int(age)}s since last write - not a measurement)"
    return rec


def load_test(csv: Path, seconds: int = 20) -> dict:
    procs = [subprocess.Popen([PY, "-c", BURN], creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
             for _ in range(12)]
    time.sleep(5)
    samples = []
    t0 = time.time()
    while time.time() - t0 < seconds - 8:
        samples.append(latest(csv))
        time.sleep(1)
    for p in procs:
        try:
            p.kill()
        except Exception:
            pass
    clk = [s["Core Clocks (avg) [MHz]"] for s in samples if s["Core Clocks (avg) [MHz]"]]
    return {"clk_min": min(clk) if clk else None, "clk_max": max(clk) if clk else None,
            "pow_max": max((s["CPU Package Power [W]"] or 0) for s in samples),
            "tctl_max": max((s["CPU (Tctl/Tdie)"] or 0) for s in samples),
            "boosted_above_base": bool(clk and max(clk) > 4001)}


def cpu_tier(port: int = 8620) -> dict | None:
    try:
        urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=4).read()
    except Exception:
        return None
    body = {"prompt": LONG, "n_predict": 32, "temperature": 0, "stream": False, "cache_prompt": False}
    try:
        r = json.loads(urllib.request.urlopen(urllib.request.Request(
            f"http://127.0.0.1:{port}/completion", data=json.dumps(body).encode(),
            headers={"Content-Type": "application/json"}), timeout=600).read())
        tm = r["timings"]
        return {"prefill_tok_s": round(1000 / tm["prompt_per_token_ms"], 1),
                "decode_tok_s": round(1000 / tm["predicted_per_token_ms"], 1)}
    except Exception as e:
        return {"error": type(e).__name__}


# ── vendor-free probes ────────────────────────────────────────────────────────
# HWiNFO is a convenience, not a requirement. Two things can be measured from Windows itself, and they were what
# settled the post-reboot question when the sensor feed was dead:
#   * memory:  Win32_PhysicalMemory ConfiguredClockSpeed -> the speed the modules are ACTUALLY trained at
#   * clock:   \Processor Information(*)\% Processor Performance x nominal base -> the effective core clock
# MEASURED 2026-11-13: 106.7 % of 3,800 MHz = ~4,056 MHz. A pinned 40x ratio reads ~105 %, so a single-threaded
# value near 113 % (4,300 MHz) is what "boost is working" actually looks like. Without this, a stale HWiNFO CSV
# meant the boost question could not be answered at all.
def memory_wmi() -> dict:
    cmd = ("Get-CimInstance Win32_PhysicalMemory | ForEach-Object { "
           "'{0}|{1}|{2}|{3}' -f $_.DeviceLocator, $_.PartNumber, $_.Speed, $_.ConfiguredClockSpeed }")
    r = subprocess.run(["powershell", "-NoProfile", "-Command", cmd], capture_output=True, text=True, timeout=120)
    dimms = []
    for line in (r.stdout or "").strip().splitlines():
        parts = [x.strip() for x in line.split("|")]
        if len(parts) == 4:
            try:
                dimms.append({"slot": parts[0], "part": parts[1],
                              "rated_mts": int(parts[2]), "configured_mts": int(parts[3])})
            except ValueError:
                pass
    cfg = [d["configured_mts"] for d in dimms if d["configured_mts"]]
    return {"dimms": dimms, "configured_mts": max(cfg) if cfg else None,
            "rated_mts": max((d["rated_mts"] for d in dimms), default=None),
            "count": len(dimms), "part": dimms[0]["part"] if dimms else None}


def busy_pct() -> float | None:
    """Host CPU utilisation, so a boost verdict is never taken on a machine that is already working."""
    cmd = ("(Get-Counter '\\Processor Information(_Total)\\% Processor Utility').CounterSamples | "
           "Measure-Object -Property CookedValue -Average | ForEach-Object { $_.Average }")
    r = subprocess.run(["powershell", "-NoProfile", "-Command", cmd], capture_output=True, text=True, timeout=60)
    try:
        return float((r.stdout or "").strip())
    except ValueError:
        return None


def clock_perf(base_mhz: float = 3800.0) -> dict:
    def one() -> float | None:
        cmd = ("(Get-Counter '\\Processor Information(*)\\% Processor Performance' -ErrorAction SilentlyContinue)"
               ".CounterSamples | Measure-Object -Property CookedValue -Maximum | ForEach-Object { $_.Maximum }")
        r = subprocess.run(["powershell", "-NoProfile", "-Command", cmd], capture_output=True, text=True, timeout=60)
        try:
            return float((r.stdout or "").strip())
        except ValueError:
            return None
    idle = one()
    busy = busy_pct()
    proc = subprocess.Popen([PY, "-c", BURN], creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    time.sleep(4)
    loaded = []
    for _ in range(5):
        v = one()
        if v:
            loaded.append(v)
        time.sleep(2)
    try:
        proc.kill()
    except Exception:
        pass
    peak = max(loaded) if loaded else None
    # MEASURED 2026-11-13: two "boost FAIL" verdicts were taken while the platform was running and the package sat
    # at 93-130 W with Tctl 73-85 C. On a 3900X that is thermal/power territory, not a CPB setting, so a low peak
    # measured on a busy box proves nothing. The verdict is withheld unless the host was quiet to begin with.
    quiescent = bool(busy is not None and busy < 8.0)
    return {"busy_pct": busy, "quiescent": quiescent, "idle_pct": idle, "loaded_pct_max": peak,
            "loaded_mhz": round(peak / 100 * base_mhz) if peak else None,
            "nominal_mhz": base_mhz,
            "pinned_4000_pct": round(4000 / base_mhz * 100, 1)}


def whea() -> dict:
    """Count only ERRORS since the last boot.

    MEASURED DEFECT: the first version counted every WHEA-Logger record, which flagged this box as unstable on the
    strength of one Id=3 /*Information*/ event dated eight months earlier. An informational record is not a fault, and
    an old one is not evidence about today. Errors are what matter, and only since the current boot.
    """
    cmd = (
        "$boot=(Get-CimInstance Win32_OperatingSystem).LastBootUpTime; "
        "$e=Get-WinEvent -FilterHashtable @{LogName='System'; ProviderName='Microsoft-Windows-WHEA-Logger'; "
        "StartTime=$boot} -ErrorAction SilentlyContinue; "
        "if(-not $e){ '0|none|none' } else { "
        "$err=@($e | Where-Object {$_.LevelDisplayName -in @('Error','Critical')}); "
        "'{0}|{1}|{2}' -f $err.Count, ($e[0].TimeCreated), ($e[0].LevelDisplayName) }")
    r = subprocess.run(["powershell", "-NoProfile", "-Command", cmd], capture_output=True, text=True, timeout=120)
    parts = (r.stdout.strip() or "0|none|none").split("|")
    try:
        errs = int(parts[0])
    except ValueError:
        errs = -1
    return {"whea_errors_since_boot": errs,
            "latest_whea_ts": parts[1] if len(parts) > 1 else None,
            "latest_whea_level": parts[2] if len(parts) > 2 else None}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--csv", default=str(DEFAULT_CSV))
    ap.add_argument("--seconds", type=int, default=20)
    a = ap.parse_args()
    csv = Path(a.csv)
    rec = {"ts": time.strftime("%Y-%m-%dT%H:%M:%S"), "rails_idle": latest(csv)}
    print("RAILS (idle):", json.dumps(rec["rails_idle"], indent=1))
    rec["load"] = load_test(csv, a.seconds)
    print("LOAD:", json.dumps(rec["load"]))
    rec["memory"] = memory_wmi()
    print("MEMORY (WMI):", json.dumps(rec["memory"]))
    rec["clock"] = clock_perf()
    print("CLOCK (OS counter):", json.dumps(rec["clock"]))
    rec["cpu_tier"] = cpu_tier()
    print("CPU TIER:", rec["cpu_tier"])
    rec["stability"] = whea()
    print("STABILITY:", rec["stability"])

    mem = rec["rails_idle"]["Memory Clock [MHz]"]
    fclk = rec["rails_idle"]["Infinity Fabric Clock (FCLK) [MHz]"]
    status = rec["rails_idle"].get("_status", "")
    measured = status == "live"
    def tri(value: bool) -> str:
        return ("PASS" if value else "FAIL") if measured else "UNMEASURED"
    cfg = rec["memory"].get("configured_mts")
    peak_pct = rec["clock"].get("loaded_pct_max")
    # memory speed is measurable without any vendor tool; boost is judged from the OS counter when the rails are
    # unmeasured, and only falls back to UNMEASURED when neither instrument produced a number.
    docp = ("PASS" if cfg >= 3000 else "FAIL") if cfg else "UNMEASURED"
    if peak_pct and peak_pct >= 109.0:
        boost = "PASS"
    elif not rec["clock"].get("quiescent", True):
        boost = "UNMEASURED"
    elif peak_pct:
        boost = "FAIL"
    else:
        boost = tri(bool(rec["load"]["boosted_above_base"]))
    verdict = {
        "docp_3200": docp,
        "fclk_1to1": tri(bool(mem and fclk and abs(fclk - mem * 1.0) < 5)),
        "boost_active": boost,
        "stable": "PASS" if rec["stability"]["whea_errors_since_boot"] == 0 else "FAIL",
    }
    if not measured:
        print(f"\n!! HWiNFO feed is {status} - rail verdicts are UNMEASURED, not failures. "
              f"Start HWiNFO's sensor logging before trusting any of this.")
    rec["verdict"] = verdict
    print("\n=== VERDICT ===")
    for k, v in verdict.items():
        print(f"  {k:<14} {v}")
    if cfg:
        print(f"  memory trained at {cfg} MT/s (part {rec['memory']['part']})")
    if peak_pct:
        print(f"  clock {rec['clock']['loaded_mhz']} MHz = {peak_pct:.1f} % of {rec['clock']['nominal_mhz']} MHz nominal "
              f"(pinned-4000 reads {rec['clock']['pinned_4000_pct']} %)")
        if not rec["clock"].get("quiescent", True):
            print(f"  host was NOT quiescent before the test (CPU utility {rec['clock'].get('busy_pct'):.1f} %) - "
                  f"a boost verdict is withheld; stop the fleet and re-run on a quiet machine")
    out = REPO / "scratch" / "hw_verify" / f"{time.strftime('%Y%m%d_%H%M%S')}_verify.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(rec, indent=1), encoding="utf-8")
    print("record:", out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
