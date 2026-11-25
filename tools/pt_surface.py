#!/usr/bin/env python3
"""321 — vulnerability assessment: the attack surface, measured rather than assumed.

Enumerates route prefixes, `unsafe` blocks (and whether any sit on an input path), JSON parse sites, the bind
address, and the file permissions on the council token. Everything printed here is read from the live tree or the
live process.
"""
import json, re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "trm_uefi_kernel"
OUT = ROOT / "evidence" / "pt_surface.json"


def sh(cmd):
    try:
        return subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=60).stdout
    except Exception as e:
        return f"<error {e}>"


def routes():
    main = (KERNEL/"host_council_server/src/main.rs").read_text(errors="replace")
    return sorted(set(re.findall(r'url\.starts_with\("(/[a-z0-9_/\-]+)"', main)))


def unsafe_blocks():
    out = []
    for base in (KERNEL/"src", KERNEL/"host_council_server/src"):
        for f in base.rglob("*.rs"):
            if "target" in str(f):
                continue
            for i, line in enumerate(f.read_text(errors="replace").splitlines(), 1):
                if re.search(r"\bunsafe\s*\{", line) or re.search(r"\bunsafe\s+fn\b", line):
                    out.append({"file": str(f.relative_to(ROOT)), "line": i, "code": line.strip()[:120]})
    return out


def classify_unsafe(blocks):
    cats = {"ffi_or_syscall": 0, "raw_pointer": 0, "transmute": 0, "asm": 0, "unknown": 0}
    for b in blocks:
        c = b["code"]
        if "asm!" in c:
            cats["asm"] += 1
        elif "transmute" in c:
            cats["transmute"] += 1
        elif "from_raw" in c or "as *" in c or "ptr" in c:
            cats["raw_pointer"] += 1
        elif "extern" in c or "syscall" in c:
            cats["ffi_or_syscall"] += 1
        else:
            cats["unknown"] += 1
    return cats


def main():
    r = routes()
    blocks = unsafe_blocks()
    parse_sites = sum(len(re.findall(r"serde_json::from_str",
                                     f.read_text(errors="replace")))
                      for f in (KERNEL/"host_council_server/src").rglob("*.rs"))
    bind = sh('grep -n "bind_host\\|Server::http" "'
              + str(KERNEL/"host_council_server/src/main.rs") + '" | head -4')
    listen = sh('powershell.exe -NoProfile -Command "Get-NetTCPConnection -State Listen '
                '-OwningProcess (Get-Process host_council_server).Id | Select-Object -ExpandProperty LocalAddress"')

    acl = sh('powershell.exe -NoProfile -Command "(Get-Acl \'C:\\\\ProgramData\\\\trm\\\\council_token.txt\').Access '
             '| Select-Object IdentityReference,FileSystemRights | ConvertTo-Json"')

    data = {
        "routes": r,
        "route_count": len(r),
        "unsafe_blocks": len(blocks),
        "unsafe_categories": classify_unsafe(blocks),
        "unsafe_in_host_server": [b for b in blocks if "host_council_server" in b["file"]],
        "json_parse_sites": parse_sites,
        "bind_config": bind.strip().splitlines()[:2],
        "listen_addresses": [x.strip() for x in listen.splitlines() if x.strip()],
        "token_acl_raw": acl.strip()[:600],
    }
    OUT.parent.mkdir(exist_ok=True)
    OUT.write_text(json.dumps(data, indent=1), encoding="utf-8")

    print(f"routes {len(r)} | unsafe blocks {len(blocks)} | json parse sites {parse_sites}")
    print(f"unsafe categories {data['unsafe_categories']}")
    print(f"unsafe on the request path (host server): {len(data['unsafe_in_host_server'])}")
    for b in data["unsafe_in_host_server"][:5]:
        print(f"   {b['file']}:{b['line']}  {b['code']}")
    print(f"listen addresses: {data['listen_addresses'] or '(none reported)'}")
    world_readable = "BUILTIN" in acl and "Users" in acl
    print("token ACL:", "WORLD-READABLE" if world_readable else "restricted")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
