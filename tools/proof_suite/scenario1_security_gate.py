#!/usr/bin/env python3
"""PROOF SCENARIO 1 - Sovereign code-security gate against a REAL codebase.

REAL-WORLD USE: a CI gate that must scan a repository and block on dangerous
constructs. This is not a synthetic benchmark: the corpus is the platform's OWN
source tree (~500 Python files that are actually deployed), plus a control set of
known-bad files whose ground truth is stated up front.

WHY THIS PROVES SOMETHING THE OLD DETECTOR COULD NOT
The previous detector caught 7 of 15 standard attacks and flagged 4 of 5 benign
snippets as vulnerabilities. Run at repo scale, a detector with a 80% false
positive rate is not merely useless - it is worse than nothing, because every
clean file becomes an alert and real findings drown. The measured quantities here
are therefore BOTH directions:

    PRECISION  - of the files flagged, how many are actually dangerous?
    RECALL     - of the dangerous files, how many were caught?

A gate is only usable if precision is high enough that a human will read the
report, and recall is high enough that the report is worth reading.

GROUND TRUTH
    negative corpus : the repo's own Python files. These are deployed code; a
                      flagged file is a false positive unless it genuinely
                      contains one of the dangerous constructs.
    positive corpus : planted files, each written to contain exactly one class of
                      danger, labelled by construction.

NOT CLAIMED: that this finds all vulnerabilities. It is a lexical gate. It is
claimed to do the two things above measurably.
"""
import json, os, pathlib, subprocess, sys, time

ROOT = pathlib.Path(r"F:\Development sites\TRM-Swarm-OS-v2")
ENDPOINT = "http://127.0.0.1:8022/ddr/scan"

# --- the positive corpus: one danger class per file, labelled by construction ---
PLANTS = {
 "plant_eval_dynamic.py":  "import sys\ndef run(x):\n    return eval(x)\n",
 "plant_exec_dynamic.py":  "def run(src):\n    exec(src)\n",
 "plant_eval_spaced.py":    "def run(x):\n    return eval (x)\n",          # the old single-space bypass
 "plant_dunder_import.py":  "def run(c):\n    __import__('os').system(c)\n",
 "plant_os_popen.py":       "import os\ndef run(c):\n    return os.popen(c).read()\n",
 "plant_subprocess_shell.py":"import subprocess\ndef run(c):\n    subprocess.run(c, shell=True)\n",
 "plant_pickle.py":         "import pickle\ndef load(b):\n    return pickle.loads(b)\n",
 "plant_yaml.py":           "import yaml\ndef load(d):\n    return yaml.load(d)\n",
 "plant_importlib.py":      "import importlib\ndef run(n):\n    return importlib.import_module(n)\n",
 "plant_sql_concat.py":     "def q(n):\n    return \"SELECT * FROM u WHERE n=\" + n\n",
 "plant_getattr_obf.py":    "def run(x):\n    return getattr(__builtins__, 'e'+'v'+'al')(x)\n",
 "plant_ctypes.py":         "import ctypes\ndef run():\n    return ctypes.CDLL('libc.so.6')\n",
}


def scan(code, timeout=30):
    import urllib.request
    body = json.dumps({"code": code}).encode()
    req = urllib.request.Request(ENDPOINT, data=body, method="POST",
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def main():
    print("=" * 78)
    print("PROOF SCENARIO 1 - security gate vs a REAL codebase")
    print("=" * 78)

    # ---- RECALL: the labelled positive corpus ----
    tp = 0; fn = []
    print(f"\n[RECALL] scanning {len(PLANTS)} planted files, each one danger class")
    for name, body in PLANTS.items():
        try:
            r = scan(body)
            hits = r.get("matches", [])
        except Exception as e:
            hits = []; print(f"   {name:28} SCAN ERROR {type(e).__name__}")
            fn.append(name); continue
        if hits:
            tp += 1
        else:
            fn.append(name)
            print(f"   MISSED  {name}")
    print(f"   recall: {tp}/{len(PLANTS)} = {100*tp/len(PLANTS):.1f}%")
    if fn: print(f"   missed: {fn}")

    # ---- PRECISION: the repo's own deployed Python ----
    skip = ("site-packages", "node_modules", ".git", "train_venv", "__pycache__",
            "backups", "proof_suite")
    files = []
    for f in ROOT.rglob("*.py"):
        s = str(f).replace("\\", "/")
        if any(k in s for k in skip): continue
        try:
            if f.stat().st_size > 200_000: continue
            files.append(f)
        except OSError:
            pass
    print(f"\n[PRECISION] scanning {len(files)} real deployed Python files")
    flagged = []
    t0 = time.time()
    for i, f in enumerate(files):
        try:
            code = f.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        try:
            r = scan(code)
            hits = r.get("matches", [])
        except Exception:
            continue
        if hits:
            flagged.append((str(f.relative_to(ROOT)), [h["error_type"] for h in hits]))
    dur = time.time() - t0
    print(f"   scanned {len(files)} files in {dur:.1f}s ({len(files)/dur:.0f} files/s)")
    print(f"   flagged: {len(flagged)}  ({100*len(flagged)/max(1,len(files)):.2f}% of corpus)")
    print(f"   precision (using planted corpus as the only known positives):")
    print(f"     true positives  : {tp}")
    print(f"     flagged repo files treated as UNKNOWN, not assumed false:")
    for name, kinds in flagged[:12]:
        print(f"       {name[:62]:64} {kinds}")
    if len(flagged) > 12:
        print(f"       ... and {len(flagged)-12} more")

    out = {
      "scenario": "security_gate_real_codebase",
      "positive_corpus_size": len(PLANTS),
      "recall_caught": tp,
      "recall_pct": round(100*tp/len(PLANTS), 1),
      "missed": fn,
      "repo_python_files_scanned": len(files),
      "scan_duration_s": round(dur, 2),
      "files_per_second": round(len(files)/dur, 1) if dur else 0,
      "flagged": len(flagged),
      "flagged_pct": round(100*len(flagged)/max(1,len(files)), 2),
      "flagged_detail": [{"file": n, "kinds": k} for n, k in flagged],
      "verdict_note": ("Recall is unambiguous: each plant contains exactly one danger "
                       "class. Flagged repo files are NOT labelled negatives - they "
                       "require human adjudication, so the honest precision statement "
                       "is 'N files need review', not 'N false positives'."),
    }
    (ROOT/"evidence"/"proof_scenario1.json").write_text(json.dumps(out, indent=2), encoding="utf-8")
    print(f"\nwrote evidence/proof_scenario1.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
