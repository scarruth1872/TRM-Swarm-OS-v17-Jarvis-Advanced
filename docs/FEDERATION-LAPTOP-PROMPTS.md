# Prompts for the laptop Hermes agent — federation setup

Hand these to the Hermes agent on `nspiredfitness` (RTX 3070 laptop, `100.123.215.119`).
Each prompt is self-contained and states its own pass criteria. The agent must paste **raw command
output**, never a summary, and must not write "SEALED", "VERIFIED" or "complete" unless the command
that proves it is in the transcript.

Shared secret handling: the user creates it once with `openssl rand -hex 32`, saves it to
`C:\ProgramData\trm\council_token.txt` on **both** machines, and neither agent prints its contents.
The token file is read at launch; it never appears in a prompt, a log or a chat message.

---

## Prompt 1 — make this kernel callable from the desktop (bind + auth)

```
You are on the RTX 3070 field laptop, a TRM Swarm OS node. Right now the native Rust kernel
(host_council_server.exe, v20.0.0-sovereign) is bound to 127.0.0.1:8022, so only this laptop can talk
to it. I need the desktop node (100.93.191.43) to be able to call it, WITHOUT exposing an
unauthenticated proposal endpoint.

Do exactly this, in order, and paste the raw output of every command:

1. Locate the running kernel and the repo:
   netstat -ano | findstr ":8022" | findstr LISTENING
   Get-CimInstance Win32_Process -Filter "Name='host_council_server.exe'" | Select-Object ProcessId,ExecutablePath
   Record the PID and the executable path.

2. Confirm the token file exists and is not empty, WITHOUT printing its contents:
   if (Test-Path C:\ProgramData\trm\council_token.txt) { "present, length=" + (Get-Content C:\ProgramData\trm\council_token.txt -Raw).Trim().Length } else { "MISSING - stop and report" }
   Expected: "present, length=64". If it is missing, STOP and tell me; do not invent a token.

3. Confirm the guard actually fires, so we know it is real:
   run the binary with TRM_BIND_HOST=0.0.0.0 and COUNCIL_AUTH_TOKEN *unset* and paste the exact
   stderr/stdout. Expected, verbatim: "FATAL: refusing to bind 0.0.0.0 without COUNCIL_AUTH_TOKEN".
   It must exit non-zero. If it starts instead, say so loudly - that is a security regression.

4. Restart the kernel properly, with the token loaded from the file (never echo it):
   $env:COUNCIL_AUTH_TOKEN = (Get-Content C:\ProgramData\trm\council_token.txt -Raw).Trim()
   $env:TRM_BIND_HOST = "0.0.0.0"
   Start-Process -FilePath <executable path from step 1> -WorkingDirectory <repo root> -WindowStyle Hidden
   Start-Sleep -Seconds 6

5. Prove it is listening on all interfaces and answering:
   netstat -ano | findstr ":8022" | findstr LISTENING
   curl.exe -s -m 6 http://127.0.0.1:8022/system/health_matrix
   curl.exe -s -m 6 http://100.123.215.119:8022/system/health_matrix
   Expected: "0.0.0.0:8022 ... LISTENING" and BOTH curl commands return JSON containing
   "kernel_version":"v20.0.0-sovereign".

6. Prove authentication is enforced, both ways:
   curl.exe -s -m 6 -X POST http://127.0.0.1:8022/council/propose -d "{}"
   curl.exe -s -m 6 -X POST http://127.0.0.1:8022/council/propose -H "X-Council-Token: <the token value read from the file>" -d "{}"
   Expected: the first is rejected (401/403, unauthenticated), the second is accepted or rejected on
   content - never on auth. Paste both bodies. THIS IS THE POINT: binding off-loopback is only safe
   if the token is genuinely checked.

7. Report exactly: PID, bind address, both health_matrix results, both propose results, and whether
   a Windows Firewall rule is blocking 8022 from the tailnet/LAN:
   Get-NetFirewallRule -Enabled True -Direction Inbound | Get-NetFirewallPortFilter | Where-Object { $_.LocalPort -eq 8022 }
   If nothing is listed, add one scoped to the tailnet and LAN subnets only:
   New-NetFirewallRule -DisplayName "TRM Kernel 8022" -Direction Inbound -Protocol TCP -LocalPort 8022 -Action Allow -Profile Private,Domain

Do not claim success from a start-up banner. Success is: 0.0.0.0:8022 LISTENING, the tailnet IP answers
health_matrix, and /council/propose rejects a request that carries no token.
```

---

## Prompt 2 — peer-verify from the laptop side (proves the link is bidirectional)

```
Continue on the field laptop. The desktop node must now be reachable BOTH ways. Prove the outbound
half and record the evidence with hashes.

1. Confirm the desktop is up and which kernel answers:
   curl.exe -s -m 8 http://100.93.191.43:8022/system/health_matrix
   Expected: JSON with "kernel_version" and "k3_invariant".

2. Post the federation sync handshake to the desktop, WITH the token header:
   curl.exe -s -m 8 -X POST http://100.93.191.43:8022/federation/sync -H "X-Council-Token: <token from file>" -d "{\"monad_anchor\":\"0x4D4F4E4144000001\",\"from\":\"nspiredfitness\"}"
   Paste the full body. Report the "status" field verbatim. If it says
   "recorded_unauthenticated" or "anchor_mismatch", that is the truthful result - report it, do not
   retry until it looks green.

3. Pull and hash the desktop's state so we have a cross-node record:
   For each of /system/health_matrix /memory/stats /kanban/stats:
     curl.exe -s -m 8 http://100.93.191.43:8022<path> -o %TEMP%\peer_<name>.json
     then compute SHA-256 (Get-FileHash -Algorithm SHA256) and report path, byte size, hash.
   Report the desktop's numbers: corpus documents, dim, kanban total.

4. Compare with this laptop's own numbers from the same endpoints and state the difference plainly.

Pass criteria: a JSON body from the desktop's health_matrix, a full body from the sync POST with its
"status" field quoted, and three hashes. If any endpoint 404s, say so - a 404 is a real answer.
```

---

## Prompt 3 — file transport (SSH server, so shards can move)

```
Continue on the field laptop. We need a real file path between the two machines. There is no upload
route on :8022, so use Windows OpenSSH server. Configure it correctly and prove it.

1. Is sshd installed, running, and listening?
   Get-Service sshd | Select-Object Status,StartType
   netstat -ano | findstr ":22 " | findstr LISTENING

2. This account is almost certainly in the Administrators group. On Windows, sshd_config ends with a
   "Match Group administrators" block that overrides AuthorizedKeysFile, so ~/.ssh/authorized_keys is
   IGNORED for admins - the key must go in C:\ProgramData\ssh\administrators_authorized_keys.
   Check and report:
   Select-String -Path C:\ProgramData\ssh\sshd_config -Pattern "^AuthorizedKeysFile|^Match"
   ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

3. Install the desktop's PUBLIC key (never a private key) into the correct file for this account type.
   Append, do not overwrite:
   Add-Content -Path C:\ProgramData\ssh\administrators_authorized_keys -Value "<desktop public key line>"
   Then lock the ACL or sshd refuses the file:
   icacls.exe "C:\ProgramData\ssh\administrators_authorized_keys" /inheritance:r /grant "Administrators:F" /grant "SYSTEM:F"

4. Validate the key blob before blaming auth (a truncated paste fails exactly like a wrong ACL):
   base64-decode the second field and confirm it is ssh-ed25519, 32-byte key, 51 bytes total.

5. Prove inbound SSH actually works, with a timeout so a hang cannot look like success:
   ssh -i <key> -o BatchMode=yes -o ConnectTimeout=10 <local-account-name>@100.123.215.119 "echo OK"
   Note: the tailnet identity (shawncarruth@) is NOT a local account; Windows OpenSSH authenticates
   against the local SAM account. Test with the real local username.

6. Report: sshd status, listen address, which authorized_keys file is in force, the ACL line you set,
   and the exact output of the ssh test.

Do not report success from a service being "Running". Success is the ssh command returning "OK".
```

---

## Prompt 4 — evidence block (what to send back)

```
Send back one block containing only raw artifacts, no commentary:

- PID and executable path of the running kernel, and its bind address from netstat
- The verbatim FATAL line from the unauthenticated 0.0.0.0 attempt
- health_matrix JSON from 127.0.0.1, from the laptop's tailnet IP, and from the desktop's tailnet IP
- The /council/propose response WITHOUT a token and the response WITH one
- The /federation/sync response from the desktop, with its "status" field
- Three (endpoint, bytes, sha256) triples for the desktop's state, and the same for this laptop
- sshd status, listen address, and the ssh "OK" output

Do not include the token value, any private key, or any paraphrase of these results. If something
failed, send the failure verbatim - a blocker reported honestly is worth more than a green summary
that cannot be re-checked.
```

---

## What the desktop will check when you report back

| check | command run on the desktop | expected |
|---|---|---|
| laptop callable | `curl http://100.123.215.119:8022/system/health_matrix` | 200, `v20.0.0-sovereign` |
| auth enforced | `curl -X POST .../council/propose -d '{}'` | 401/403 without token |
| bidirectional sync | `python tools/federation_sync.py --pull` | NODE-004 `reachable`, POST returns a status string |
| file transport | `ssh -o BatchMode=yes -o ConnectTimeout=10 shawn@100.123.215.119 echo OK` | `OK` |
| ledger | `swarm_v2_federation/sync_ledger.json` | new run with per-endpoint hashes |

`tools/federation_sync.py` (desktop) writes that ledger; `tools/audit_ingest_ledger.py` will reject any
claimed ingest that lacks source + records + bytes + SHA-256.
