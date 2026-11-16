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

---

# REVISED after the laptop's reconnaissance (v19 host, not v20)

The first version of Prompt 1 assumed the laptop ran the same tree as the desktop. It does not:
the laptop's kernel is `C:\Users\shawn\Downloads\TRM-Swarm-OS-v19-...\host_council_server.exe`,
PID listening on **0.0.0.0:8022**, with `Server::http("0.0.0.0:{port}")` **hardcoded at v19
main.rs:3984** and **zero** occurrences of `COUNCIL_AUTH_TOKEN` or `TRM_BIND_HOST`. So the guard was
never there to verify - it has to be built. The desktop's v20 tree does have it (`auth_gate`,
`main.rs:2468`; fail-closed bind at `main.rs:6196-6220`), which is why the desktop printed the FATAL
line and the laptop could not.

Consequence, stated plainly: **the laptop's mutation surface is open to anyone on the tailnet right
now** (`POST /council/propose {}` returns 400 `topic_required`, i.e. it reached the handler).

## Step 0 - mitigate in the next 2 minutes, before touching code

```
On the laptop, restrict inbound 8022 to the desktop only (no rebuild, no restart):
  Remove-NetFirewallRule -DisplayName "TRM Kernel 8022" -ErrorAction SilentlyContinue
  New-NetFirewallRule -DisplayName "TRM Kernel 8022 (desktop only)" -Direction Inbound -Protocol TCP `
    -LocalPort 8022 -Action Allow -RemoteAddress 100.93.191.43
  New-NetFirewallRule -DisplayName "TRM Kernel 8022 (block rest)" -Direction Inbound -Protocol TCP `
    -LocalPort 8022 -Action Block -Profile Any
  Get-NetFirewallRule -DisplayName "TRM Kernel 8022*" | Format-Table DisplayName,Enabled,Direction,Action
Then prove it from a non-desktop address if you have one, or state that only the desktop answered.
The code fix below is the real closure; this buys the time.
```

## Prompt 1-REV - build the guard into v19, peer-scoped

```
You are on the field laptop running TRM Swarm OS v19 from C:\Users\shawn\Downloads\TRM-Swarm-OS-v19-...
The kernel binds 0.0.0.0 with no authentication on writes. Build the guard now, matching the desktop's
v20 design exactly. Paste raw output for every step; stop and report on any failure.

1. Back up before editing (the repo is not a git checkout on this host if it says so):
   Copy-Item main.rs main.rs.bak-<yyyyMMdd-HHmmss>

2. Add the bind guard. Replace the hardcoded listen with:
   let bind_host = std::env::var("TRM_BIND_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
   let auth_configured = std::env::var("COUNCIL_AUTH_TOKEN").ok()
       .map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).is_some();
   let is_loopback = matches!(bind_host.as_str(), "127.0.0.1" | "localhost" | "::1");
   if !is_loopback && !auth_configured {
       eprintln!("FATAL: refusing to bind {} without COUNCIL_AUTH_TOKEN", bind_host);
       std::process::exit(1);
   }
   let server = Arc::new(Server::http(format!("{}:{}", bind_host, port)).unwrap());

3. Add the write gate, called before dispatch for every request. PEER-SCOPED, deliberately:
   - is_write = POST | PUT | PATCH | DELETE
   - if !is_write -> allow (GETs stay readable for health checks and federation probes)
   - if request.remote_addr() is loopback -> allow WITHOUT a token. Loopback is the local trust
     boundary the dashboard lives behind (same-origin :5183 -> Vite proxy -> 127.0.0.1:8022). Without
     this exemption, enabling federation would 401 your own dashboard's POSTs.
   - otherwise: require header X-Council-Token == COUNCIL_AUTH_TOKEN, else 401
     {"error":"unauthorized","status":401,"path":...,"hint":"send header X-Council-Token ..."}
   - if COUNCIL_AUTH_TOKEN is unset and the peer is not loopback -> 401 as well (fail closed).

4. Build and restart with the token read from the file, never echoed:
   $env:COUNCIL_AUTH_TOKEN = (Get-Content C:\ProgramData\trm\council_token.txt -Raw).Trim()
   $env:TRM_BIND_HOST = "0.0.0.0"
   cargo build --release
   Start-Process -FilePath .\target\release\host_council_server.exe -WorkingDirectory <repo root> -WindowStyle Hidden
   Start-Sleep -Seconds 6

5. Test matrix - paste every HTTP code and body, 4 cases:
   a) POST 127.0.0.1:8022/council/propose -d "{}"                  -> NOT 401 (dashboard path)
   b) POST 100.123.215.119:8022/council/propose -d "{}"            -> 401 unauthorized
   c) same as (b) with -H "X-Council-Token: <token from file>"     -> NOT 401
   d) GET  100.123.215.119:8022/system/health_matrix               -> 200
   (b) and (d) must differ: writes locked, reads open.

6. Prove the dashboard still works from this host:
   curl.exe -s -o NUL -w "%{http_code}" -X POST http://127.0.0.1:5183/council/propose -H "Content-Type: application/json" -d "{}"
   Expected 200 (the Vite proxy connects from loopback, so the exemption applies).

7. Prove the fail-closed path fires:
   restart once with COUNCIL_AUTH_TOKEN unset and TRM_BIND_HOST=0.0.0.0; paste the FATAL line and the
   exit code. It must exit non-zero. Then restart correctly.

Report: the 4 test codes, the dashboard code, the FATAL line + exit code, and the bind line from netstat.
```

## Prompt 2-REV - close the loop from the laptop outward

```
After the guard is live:
  curl.exe -s -m 8 -X POST http://100.93.191.43:8022/federation/sync ^
    -H "X-Council-Token: <token from file>" -H "Content-Type: application/json" ^
    -d "{\"monad_anchor\":\"0x4D4F4E4144000001\",\"from\":\"nspiredfitness\"}"
Paste the body and quote the "status" field verbatim - "recorded_unauthenticated" or
"anchor_mismatch" is the truthful answer, not something to retry until it looks green.
Then for /system/health_matrix /memory/stats /kanban/stats on the desktop: save each to %TEMP%,
report byte size + Get-FileHash SHA256, and state the desktop's corpus documents, dim and kanban total
next to this laptop's own numbers.
```

## Codebase convergence (do this after the guard, not before)

The laptop runs v19 from `Downloads`, the desktop runs v20 from `F:\Development sites\TRM-Swarm-OS-v2`.
Two divergent kernels federating is a trap: every future fix has to be written twice, and the guards
will drift. The right end state is the same tree on both hosts - clone the v20 repo on the laptop,
build `host_council_server`, and compare `/system/health_matrix` `kernel_version` across the two.

---

# Prompt 3-REV - SSH file transport (key line included)

Desktop public key (public only; the private key stays on `C:\Users\shawn\.ssh\id_ed25519` and is
never printed or transmitted):

```
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOCCPWltHQmlZmWimTmXLRFfWorCFaEmAqh0uSm3IZJw trm-federation-desktop
```

```
Enable Windows OpenSSH server on the laptop so shards can move, then prove it.

1. What is the local SAM account (NOT the tailnet identity)?
   $env:USERNAME
   whoami

2. Install/start sshd:
   Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0
   Set-Service sshd -StartupType Automatic
   Start-Service sshd -Force
   netstat -ano | findstr ":22 " | findstr LISTENING

3. This account is almost certainly in Administrators. sshd_config ends with a
   "Match Group administrators" block that overrides AuthorizedKeysFile, so ~/.ssh/authorized_keys is
   IGNORED for admins. Check it, then append the desktop's key to the right file (append, never overwrite):
   Select-String -Path C:\ProgramData\ssh\sshd_config -Pattern "^AuthorizedKeysFile|^Match"
   ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
   Add-Content -Path C:\ProgramData\ssh\administrators_authorized_keys -Value "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOCCPWltHQmlZmWimTmXLRFfWorCFaEmAqh0uSm3IZJw trm-federation-desktop"
   icacls.exe "C:\ProgramData\ssh\administrators_authorized_keys" /inheritance:r /grant "Administrators:F" /grant "SYSTEM:F"

4. Validate the blob before blaming auth - a truncated paste fails exactly like a bad ACL:
   $line = (Get-Content C:\ProgramData\ssh\administrators_authorized_keys | Where-Object { $_ -like "ssh-ed25519*" } | Select-Object -Last 1)
   $blob = [Convert]::FromBase64String(($line -split " ")[1])
   "type=0 len=1" -f ([Text.Encoding]::ASCII.GetString($blob[0..10])), $blob.Length
   Expected: type=ssh-ed25519, len=51

5. Scope :22 to the same two addresses as the kernel (do not leave it open to everything):
   Get-NetFirewallRule -DisplayName "OpenSSH-Server-In-TCP" | Get-NetFirewallAddressFilter
   Set-NetFirewallRule -DisplayName "OpenSSH-Server-In-TCP" -RemoteAddress 100.93.191.43,192.168.1.0/24

6. Prove inbound SSH with a timeout so a hang cannot look like success:
   ssh -o BatchMode=yes -o ConnectTimeout=10 $env:USERNAME@127.0.0.1 "echo OK"
   (keys only - no password prompt should appear; if it asks for a password, the ACL or the key file is wrong)

7. Report: $env:USERNAME, sshd status, listen address, which authorized_keys file is in force, the ACL
   line, the blob check output, the firewall scope, and the exact ssh "OK" output.
```

## Transfer test the desktop will run after that

```
# 1. reachability + auth
ssh -o BatchMode=yes -o ConnectTimeout=10 <user>@100.123.215.119 "echo OK"

# 2. move a real shard and its provenance sidecar
scp -o BatchMode=yes build/chembl_lpxc_real.k3 build/chembl_lpxc_real.k3.json <user>@100.123.215.119:C:/Users/<user>/trm_incoming/

# 3. verify on the far side by hash, not by exit code
ssh <user>@100.123.215.119 "certutil -hashfile C:\Users\<user>\trm_incoming\chembl_lpxc_real.k3 SHA256"
sha256sum build/chembl_lpxc_real.k3

# 4. the far side must accept it as a shard, not just as bytes
ssh <user>@100.123.215.119 "python tools/audit_ingest_ledger.py C:\Users\<user>\trm_incoming\chembl_lpxc_real.k3.json"
```

Only when steps 3 and 4 agree does `files_synced` become a number instead of 0.

