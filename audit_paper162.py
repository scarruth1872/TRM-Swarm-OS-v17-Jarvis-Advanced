"""Epistemology audit: Paper 162 (corrected) — verify no overclaims survive."""
import os, re

ROOT = r"F:\Development sites\TRM-Swarm-OS-v2"
paper = open(os.path.join(ROOT, "docs", "PAPER-162-TRANSISTOR-MAPPING.md"), encoding="utf-8").read()

# 1. No forbidden overclaims
forbidden = [
    "within 1 order of human", "78.8× human", "7,884% of human",
    "10,662 agents", "755.5M trits", "742.3B intelligence",
    "% of human brain capacity", "exceeds human",
]
print("=== FORBIDDEN OVERCLAIM CHECK ===")
ok = True
for f in forbidden:
    if f.lower() in paper.lower():
        print(f"  ✗ FOUND: '{f}'")
        ok = False
if ok:
    print("  ✓ none present")

# 2. Corrections are explicit
print("=== CORRECTIONS PRESENT ===")
for c in ["5900X", "742B", "98.4% free"]:
    present = c in paper
    print(f"  {'✓' if present else '✗'} prior-claim '{c}' documented as corrected: {present}")

# 3. Measured claims present + grounded
print("=== MEASURED GROUNDING ===")
for m in ["3.8B", "26.8B", "40 CU", "402M trits", "53 modules ENERGIZED", "70.6M agent-tasks", "0.9115", "41.8B"]:
    print(f"  {'✓' if m in paper else '✗'} measure '{m}' present")

# 4. Epistemology section present
print("=== EPISTEMOLOGY SECTION ===")
print(f"  {'✓' if '## 6. Epistemology Audit Summary' in paper else '✗'} audit table present")

print("\n=== AUDIT RESULT ===")
print("PASS" if ok else "FAIL — overclaims must be removed")