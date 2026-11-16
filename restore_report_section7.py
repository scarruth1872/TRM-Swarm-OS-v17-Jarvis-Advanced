"""Re-append the hand-written section 7 to the module-regenerated ABX-220 report.

The module owns sections 1-6; section 7 records findings about the benchmark and the
audit trail that are not derivable from the ledger, so it is carried across regeneration.
"""
import os

REPORT = r"F:\Development sites\TRM-Swarm-OS-v2\wetlab_dispatch\ABX-220_RECONCILIATION_REPORT.md"
BACKUP = os.path.join(os.environ["LOCALAPPDATA"], "Temp", "abx220_report_backup.md")
MARK = "## 7. Structure provenance audit"

with open(BACKUP, encoding="utf-8") as fh:
    backup = fh.read()
with open(REPORT, encoding="utf-8") as fh:
    current = fh.read()

if MARK not in backup:
    raise SystemExit("section 7 not found in backup - aborting")
section7 = MARK + backup.split(MARK, 1)[1]

if MARK in current:
    print("section 7 already present; nothing to do")
else:
    with open(REPORT, "w", encoding="utf-8") as fh:
        fh.write(current.rstrip() + "\n\n" + section7.rstrip() + "\n")
    print("section 7 re-appended")

with open(REPORT, encoding="utf-8") as fh:
    txt = fh.read()
print("report sections:", [l.strip() for l in txt.splitlines() if l.startswith("## ")])
