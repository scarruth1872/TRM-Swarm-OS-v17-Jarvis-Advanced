#!/usr/bin/env python3
"""329 — physical security: what exists, and what does not.

There is no server room, no badge reader and no site to walk into. Reporting "physical assessment complete" would be
a fabrication, so this measures the physical-adjacent controls that DO exist on a single desktop: disk encryption
state, screen-lock policy, and the permissions on the files that hold secrets and evidence.
"""
import json, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "evidence" / "pt_physical.json"


def sh(cmd):
    try:
        return subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=60).stdout.strip()
    except Exception as e:
        return f"<error {e}>"


def main():
    bitlocker = sh("powershell.exe -NoProfile -Command \"Get-BitLockerVolume -MountPoint C: | "
                   "Select-Object VolumeStatus,ProtectionStatus | ConvertTo-Json\"")
    screen = sh("powershell.exe -NoProfile -Command \"powercfg /q SCHEME_CURRENT SUB_VIDEO VIDEOIDLE\" | "
                "Select-String -Pattern 'Current AC Power Setting' | Select-Object -First 1")
    acl_token = sh("powershell.exe -NoProfile -Command \"(Get-Acl 'C:\\ProgramData\\trm\\council_token.txt').Access | "
                   "Select-Object -ExpandProperty IdentityReference | ForEach-Object {$_.Value}\"")
    acl_evidence = sh("powershell.exe -NoProfile -Command \"(Get-Acl '" + str(ROOT/'evidence').replace('\\','/') +
                      "').Access | Select-Object -ExpandProperty IdentityReference | ForEach-Object {$_.Value}\"")

    data = {
        "site": "NONE - single desktop, no server room, no badge or visitor controls to assess",
        "bitlocker_raw": bitlocker[:300],
        "screen_lock_raw": screen[:200],
        "token_readers": [x for x in acl_token.splitlines() if x.strip()],
        "evidence_dir_readers": [x for x in acl_evidence.splitlines() if x.strip()],
    }
    OUT.write_text(json.dumps(data, indent=1), encoding="utf-8")

    encrypted = "FullyEncrypted" in bitlocker
    lock_configured = bool(screen.strip())
    world = any("Users" in x for x in data["token_readers"])

    print("site: NONE — no physical facility exists to assess (stated, not faked)")
    print(f"disk encryption (BitLocker): {'FullyEncrypted' if encrypted else 'NOT CONFIRMED'}")
    print(f"screen-lock policy present : {lock_configured}")
    print(f"token readable by Users    : {world}")
    print(f"evidence dir readers       : {data['evidence_dir_readers'][:4]}")
    print(f"wrote {OUT.relative_to(ROOT)}")

    findings = []
    if not encrypted:
        findings.append("disk encryption not confirmed: an offline attacker with the machine reads every secret")
    if world:
        findings.append("token readable by BUILTIN\\Users on disk")
    print("findings:", findings or "none")
    print("RESULT:", "PARTIAL" if findings else "PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
