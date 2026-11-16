"""Correct unsupported TRM-ABX-220 claims in the booted platform's Rust sources.

The council server is what the booted system actually serves, so these are the highest-
visibility instances: a fabricated Phase II clinical trial, an SPR/drug-susceptibility
verdict, a potency figure, and a "VALIDATED_LOCK_SEALED" pre-registration lock for a
compound that has no molecular structure on record.

Substring replacements only, so surrounding code and formatting are untouched.
"""
import sys

FILES = [
    "trm_uefi_kernel/host_council_server/src/main.rs",
    "trm_uefi_kernel/src/autonomous_scenario_setup.rs",
    "trm_uefi_kernel/trm_companion_bridge/src/main.rs",
]

REPL = [
    # clinical verdict / susceptibility
    ("TRM-ABX-220 is FULLY SUSCEPTIBLE against the A215V mutant with clean cardiac safety.",
     "TRM-ABX-220 has no molecular structure on record, so no susceptibility or cardiac-safety claim is made for it."),
    ("TRM-ABX-220 remains fully susceptible.",
     "TRM-ABX-220: no molecular structure on record, so no susceptibility claim is supported."),
    # fabricated clinical trial
    ("12-Hospital Phase II Clinical Trial Monitoring for TRM-ABX-220 (230 patients, 148.2k FHIR events, zero data leakage, differential privacy e=0.45) with FDA eCTD safety reporting",
     "12-Hospital Phase II Clinical Trial Monitoring for TRM-ABX-220 [RETRACTED: no such trial exists - the compound has no molecular structure on record]"),
    ("Scenario C: Autonomous Phase II Clinical Trial Monitoring (TRM-ABX-220)",
     "Scenario C: [RETRACTED] Autonomous Phase II Clinical Trial Monitoring (TRM-ABX-220 - no structure on record, no trial exists)"),
    # potency
    ("Enhanced pipeline validated TRM-ABX-220 LpxC potency (3.6-4.8 nM across Gram-negatives)",
     "RETRACTED: TRM-ABX-220 has no molecular structure on record, so no potency value is supported. (Formerly asserted: 3.6-4.8 nM across Gram-negatives)"),
    # pre-registration lock that does not exist
    ('"preview": "LpxC inhibitor Ki=4.2nM"',
     '"preview": "WITHHELD - no molecular structure on record"'),
    ("TRM-ABX-220 (LpxC Ki=4.2nM)",
     "TRM-ABX-220 (withheld - no molecular structure on record)"),
    ('"potency": "Ki=4.2 nM", "status": "VALIDATED_LOCK_SEALED"',
     '"potency": null, "status": "WITHHELD_NO_STRUCTURE_ON_RECORD"'),
    ("TRM-ABX-220 (LpxC A215V resistant)",
     "TRM-ABX-220 (withheld - no molecular structure on record)"),
]

total = 0
for path in FILES:
    try:
        src = open(path, encoding="utf-8", errors="surrogateescape").read()
    except FileNotFoundError:
        print(f"  SKIP (absent): {path}")
        continue
    n = 0
    for old, new in REPL:
        c = src.count(old)
        if c:
            src = src.replace(old, new)
            n += c
    if n:
        open(path, "w", encoding="utf-8", errors="surrogateescape").write(src)
    print(f"  {n:>2} replacement(s)  {path}")
    total += n

print(f"\ntotal: {total}")
# report anything still asserting a bare figure
import re
for path in FILES:
    try:
        s = open(path, encoding="utf-8", errors="surrogateescape").read()
    except FileNotFoundError:
        continue
    left = [m for m in re.findall(r".{40}(?:Ki=4\.2|VALIDATED_LOCK_SEALED|Phase II Clinical Trial Monitoring for TRM-ABX-220).{40}", s)]
    if left:
        print(f"  REMAINING in {path}:")
        for l in left[:5]:
            print("   ", l.replace("\n", " ")[:120])
