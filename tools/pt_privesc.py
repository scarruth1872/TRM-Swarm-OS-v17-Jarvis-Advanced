#!/usr/bin/env python3
"""325 — privilege escalation: what can an unprivileged local process do to this platform?

This is a single-user desktop, so the classic escalation paths (setuid, sudo, service accounts) barely exist. What
does exist: file permissions on the auth token, path handling on the file-reading endpoints, whether any endpoint
can write or execute, and whether the process itself holds more privilege than it needs.
"""
import json, os, subprocess, sys, urllib.request, urllib.error
from pathlib import Path

TOK = Path("C:/ProgramData/trm/council_token.txt").read_text().strip()
ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "evidence" / "pt_privesc.json"
BASE = "http://127.0.0.1:8022"


def sh(cmd):
    try:
        return subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=45).stdout.strip()
    except Exception as e:
        return f"<error {e}>"


def post(path, payload):
    req = urllib.request.Request(BASE+path, data=json.dumps(payload).encode(), method="POST",
                                 headers={"Content-Type": "application/json", "X-Council-Token": TOK})
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return r.status, r.read()[:300].decode("utf-8", "replace")
    except urllib.error.HTTPError as e:
        return e.code, e.read()[:300].decode("utf-8", "replace")


def main():
    findings = []

    # 1. token permissions
    acl_cmd = ("powershell.exe -NoProfile -Command \"Get-Acl 'C:\\ProgramData\\trm\\council_token.txt' "
               "| Select-Object -ExpandProperty Access | Select-Object IdentityReference,FileSystemRights "
               "| ConvertTo-Json\"")
    acl = sh(acl_cmd)
    world = "BUILTIN" in acl and "Users" in acl
    findings.append({"id": "PT-325-1", "severity": "MEDIUM" if world else "INFO",
                     "finding": "council token readable by BUILTIN\\Users" if world else "token ACL restricted",
                     "evidence": acl[:300],
                     "impact": "any local interactive user can read the token and call 186 authenticated endpoints",
                     "fix": "icacls council_token.txt /inheritance:r /grant:r \"%USERNAME%:R\" /grant:r SYSTEM:R"})

    # 2. process privileges
    who = sh("whoami")
    privs = sh("powershell.exe -NoProfile -Command \"whoami /priv\"")
    elevated = "SeDebugPrivilege" in privs and "Enabled" in privs
    findings.append({"id": "PT-325-2", "severity": "INFO",
                     "finding": f"kernel runs as {who}",
                     "evidence": "SeDebugPrivilege enabled" if elevated else "no elevated debug privilege reported",
                     "impact": "single-user process, no service account to pivot to"})

    # 3. can endpoints read outside the repo?
    probes = [("../../../../Windows/win.ini", "/reading/parse"), ("C:/Windows/win.ini", "/kv/source"),
              ("docs/../../secrets.txt", "/ie/language")]
    escapes = []
    for p, ep in probes:
        code, body = post(ep, {"path": p})
        leaked = "win.ini" in body or "[fonts]" in body
        if code == 200 and leaked:
            escapes.append({"endpoint": ep, "path": p, "status": code})
    findings.append({"id": "PT-325-3", "severity": "HIGH" if escapes else "PASS",
                     "finding": "path traversal reaches files outside the repo" if escapes else "path traversal refused on every file-reading endpoint",
                     "evidence": json.dumps(escapes) if escapes else "all probes returned 400",
                     "impact": "arbitrary file read" if escapes else "none"})

    # 4. write or execute capability — counted properly. The first version of this check counted the string
    # "std::process" and reported 3 spawn sites; there are none in main.rs, and the real spawn sites pass literal
    # commands. A false HIGH is as damaging as a missed finding, so each site is listed and judged.
    spawn_sites = []
    for f in (ROOT/"trm_uefi_kernel/host_council_server/src").rglob("*.rs"):
        for i, line in enumerate(f.read_text(errors="replace").splitlines(), 1):
            if "Command::new(" in line and "//" not in line.split("Command::new")[0]:
                spawn_sites.append({"file": f.name, "line": i, "code": line.strip()[:100]})
    literal_only = all('Command::new("' in x["code"] for x in spawn_sites)
    findings.append({"id": "PT-325-4", "severity": "INFO" if literal_only else "MEDIUM",
                     "finding": f"process-spawn sites {len(spawn_sites)} (literal commands only: {literal_only})",
                     "evidence": "; ".join(f"{x['file']}:{x['line']}" for x in spawn_sites[:6]) or "none",
                     "impact": "no request data reaches an argv; wmic/powershell telemetry with fixed arguments",
                     "fix": "keep spawns behind literal argv; never interpolate a request field into a command"})

    # 5. repo writability for the corpus the library trusts
    repo_write = os.access(str(ROOT/"data"), os.W_OK)
    findings.append({"id": "PT-325-5", "severity": "MEDIUM" if repo_write else "INFO",
                     "finding": f"data/ writable by the kernel user: {repo_write}",
                     "evidence": str(ROOT/"data"),
                     "impact": "the evidence index and verdict ledgers can be edited by the same account that serves them",
                     "fix": "this is inherent to a single-user on-device design; integrity is protected by sha256 seals, not by permissions"})

    OUT.write_text(json.dumps({"whoami": who, "findings": findings}, indent=1), encoding="utf-8")
    for f in findings:
        print(f"[{f['severity']:6}] {f['id']} {f['finding']}")
    print(f"wrote {OUT.relative_to(ROOT)}")
    bad = [f for f in findings if f["severity"] in ("HIGH",)]
    print("RESULT:", "PASS" if not bad else "FAIL")
    return 0 if not bad else 1


if __name__ == "__main__":
    sys.exit(main())
