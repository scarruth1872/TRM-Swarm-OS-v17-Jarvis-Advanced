# Penetration test report — TRM Swarm OS kernel

Target: `host_council_server.exe` v19.001, bound to `127.0.0.1:8022`. Date 2026-11-25. All testing on the operator's own machine; no packets left the host and no external system was touched.

## Method

Ten tools under `tools/pt_*.py`, each writing machine-readable evidence to `evidence/pt_*.json`. Every number in this report is read back from those files by `tools/pt_report.py` rather than recalled.

## Attack surface (measured, task 321)

- **186 request-handling routes** in `main.rs`
- **99 `unsafe` blocks** total, of which **2** are in the request-path crate
- **57 JSON parse sites** on the request path
- listen address **['127.0.0.1']** (loopback only — no network exposure)

## Findings

| id | severity | finding | status |
|---|---|---|---|
| PT-326-1 | **CRITICAL (local)** | API is **unauthenticated**: `/kv/suite` answers 200 with no token, a wrong token, and a token in the query string; `/council/propose` accepts unauthenticated writes | OPEN — needs a decision, see remediation |
| PT-324-1 | **HIGH** | Concurrent proposals do not resolve: 1 storm(s) with 4 proposals produced 0 verdicts in 240 s | OPEN — submit in batches of ≤3 |
| PT-325-1 | **HIGH→FIXED** | Council token readable by `BUILTIN\Users` | **FIXED** (icacls: owner read, SYSTEM/admin full) |
| PT-322-1 | **MEDIUM→FIXED** | Unbounded `k` on `/reading/summarize` hangs the request (k = 2^63-1 → timeout) | **FIXED** (k/pool/max_pairs clamped) |
| PT-325-4 | MEDIUM | One `Command::new(cmd)` helper takes the command as a parameter (`voltage.rs:41`); every call site passes a literal | OPEN (informational) |
| PT-325-5 | MEDIUM (accepted) | `data/` is writable by the account serving it | accepted design — integrity comes from sha256 seals, not permissions |
| PT-329-1 | MEDIUM | Disk encryption not confirmed (BitLocker status not `FullyEncrypted`) | OPEN — operator action |

### PT-326-1 — no authentication (detail)

Measured:

```
GET  /kv/suite   no token      -> 200
GET  /kv/suite   wrong token   -> 200
GET  /kv/suite   token in query-> 200
POST /council/propose  no token -> 200  (proposal accepted into the ledger)
```

The design comment in `main.rs` states it plainly: writes require `X-Council-Token` only when `COUNCIL_AUTH_TOKEN` is set, and a loopback-bound server trusts local clients. That is a deliberate choice and it is not wrong for a single-user desktop — but it means **any local process, including one running as another user, can read the corpus and write council proposals**. Tightening the token file ACL (done) removes the cheapest path to the token, not the path itself.

Remediation, one variable: `TRM_LOOPBACK_TRUST=token`. Every tool in `tools/` already sends the header, so the test suite keeps working; the LCARS dashboard must be given the token via its config or the proxy. This was **not** applied unilaterally because it would break the running UI.

### PT-324-1 — concurrency liveness

```
health_x8            n=8  max 0.0s  timeouts 0
structured_x8        n=8  max 0.0s  timeouts 0
summarize_x6         n=6  max 0.1s  timeouts 0
async_proposals      n=4  max 240.3s  timeouts 4
```

Proposals return immediately and deliberate on background threads, so POST latency proves nothing. Four proposals submitted together produced **zero** terminal phases in 240 s, while three at a time complete in roughly eight minutes. The stall is in deliberation, not the HTTP layer — the same signature seen when ten were submitted at once earlier (CPU flat across 65 threads).

## What did not break

- **689 hostile requests** across 26 endpoints: **0 panics** (5xx). Malformed JSON, 2 MB strings, 200-deep nesting, NUL bytes, invalid UTF-8, type confusion on every field, NaN/Infinity — all rejected 4xx or handled.
- **Path traversal refused** on every file-reading endpoint: `../../../../Windows/win.ini` and `C:/Windows/win.ini` both 400. **57** apparent acceptances are endpoints that take `text` and ignore `path` — a harness artefact, corrected in the report rather than counted as a finding.
- **Timing side channel**: valid vs wrong-token median **0.04 ms** apart with **6.71 ms** noise — within noise, no early-exit leak at this resolution.
- **Outbound connections**: ['none'] — the process talks to nothing off-box.
- **Memory unsafety**: 99 unsafe blocks, 7 memory-unsafety constructs (`from_raw_parts`, `copy_nonoverlapping`). All are on FFI or UEFI paths; **none** takes a request field. The two on the request path are `from_raw_parts` over a `#[repr(C)]` struct's own bytes and a Windows shared-memory read (HWiNFO) — neither reads attacker-controlled lengths.

## What was not tested, and why

- **Physical security (329)**: there is no facility — no server room, no badge reader, no site. Reported as absent rather than simulated; the physical-adjacent controls that do exist (disk encryption, screen lock, file ACLs) were measured instead.
- **Social engineering (328)**: there are no employees. The human-attack analogue that does exist is prompt injection into the council's LLM agents, tested by submitting a control and a treated proposal.
- **Network attacks (326)**: confined to loopback, because the server binds only there. Port scanning, MITM and ARP spoofing would test the host's network, not this application.

## Task-by-task result

| task | tool | result |
|---|---|---|
| 321 vulnerability assessment | `pt_surface.py` | 186 routes, 99 unsafe blocks, 57 parse sites, loopback-only |
| 322 fuzzing suite | `pt_fuzz.py` | 689 requests, 0 panics, 1 DoS (fixed) |
| 323 buffer overflow on unsafe | `pt_unsafe_audit.py` | no unchecked construct on the input path |
| 324 race conditions | `pt_race.py` | liveness stall at ≥4 concurrent proposals |
| 325 privilege escalation | `pt_privesc.py` | token ACL fixed; traversal refused; spawns literal |
| 326 network attack simulation | `pt_network.py` | unauthenticated API (critical, local scope) |
| 327 side-channel simulation | `pt_sidechannel.py` | no timing leak above noise |
| 328 social engineering | `pt_social.py` | prompt-injection test (result in evidence/pt_social.json) |
| 329 physical security | `pt_physical.py` | no facility; physical-adjacent controls measured |
| 330 report | `pt_report.py` | this document |

