# Crate wiring and reachability (2026-09-24)

Follows the docs rather than my assumptions: `research/MOTION_A_RUST_COUNCIL_ABSORPTION.md` (native >
bridged), `docs/IPC_TRANSPORT_AND_K3_NAMESPACE_20260924.md` (the internal transport is the ring bus, not
HTTP), and README §"Single-Target Proxy Routing Architecture" (the proxy table IS the reachable surface).

## What "wired" means here

The verified crates are **linked into `host_council_server`** as Cargo path dependencies, so the kernel
calls them in-process. That is MOTION A's doctrine — *native > bridged* — taken to its end: no `dlopen`
of `.so`/`.dll`, no IPC hop, no HTTP hop between the kernel and its own engines.

The internal transport remains the shared-memory ring bus (`swarm_mailbox.ring`); `/crate/*` is the
operator/dashboard control plane, like every other kernel route — not an internal bus.

## The routes

`GET /crate/ops` is the census, built from the same `OPS` table the dispatcher uses, so it cannot drift
from what is served.

| route | crate | gate behind it |
|---|---|---|
| `POST /crate/k3/collapse` | swarm-k3 | byte-identical to `libswarm_kernel_ops.so` over 1,000,000 values (P2d) |
| `POST /crate/trit/pack` | swarm-turbovec | byte-identical to the Python reference, 250,000 bytes (P2f) |
| `POST /crate/moe/gate` | swarm-instella | ids identical vs `c_instella_gate_topk`, weights 1.96e-6 (P2d) |
| `POST /crate/ssm/step` | swarm-ssm | max \|rust − `c_mamba3_step`\| = 1.19e-7 over 100 steps (P2d/P2e) |

Every route's answer is re-checked against a **different implementation** (`scratch/verify_crate_ops.py`,
23/23): the strict eps rule in Python for the collapse, the published 2-bit layout for the pack, and the
native C DLL through ctypes on its header's real ABI for the recurrence and the gate. Measured on this
host: ssm `9.686e-08`, instella ids identical with weights `4.889e-08`, pack byte-identical, collapse
identical. The SSM route also asserts the *defect class* — `distinct outputs > 50` — because
"memoryless" is exactly the bug the port had before P2e.

Refusals are refusals, never fall-throughs: unknown op → `404 unknown_op` naming the census; `GET` on a
`POST` op → `405` with `allowed: ["POST"]`; empty field, wrong expert count and out-of-alphabet trit →
`400` naming the field.

## Reachability — the gap this documentation check found

README documents the namespaces the dashboard proxies to `:8022`, and `dashboard/vite.config.js` is that
table (its 49 keys are where the README's "49" came from). **`/crate` was not in it**, so the new routes
answered curl on `:8022` and nothing else: a route the documented surface cannot reach is not wired as
far as the platform is concerned.

- Added `'/crate': 'http://127.0.0.1:8022'` to the proxy table; the table is now **50** entries and the
  README's count follows the table rather than asserting a number.
- Proven **through the documented path**, not just the direct port: `:5183/crate/ops` → `200
  application/json` carrying the kernel's own payload, and `:5183/crate/<unknown>` → `404
  application/json` carrying the kernel's `unknown_op`. A SPA fallback cannot produce either — which is
  why a status code alone was not accepted as evidence here; the body was read.

## P2j — the second batch of four, and the drift the oracle caught

The same rule, four more crates (each already exercised, each with a passing gate):

| route | crate | gate behind it |
|---|---|---|
| `POST /crate/council/vote` | swarm-council | `verify_council`: byte-identical to `libswarm_kernel_ops.so` on all vote vectors + the 1M sweep |
| `POST /crate/nand/roundtrip` | swarm-nand | `verify_nand`: reads the real fabric coherently (P2) |
| `POST /crate/gguf/inspect` | swarm-gguf | `verify_gguf`: parsed the real model, census and offsets ready |
| `POST /crate/git/verify` | swarm-git | `verify_git`: PASSED |

`POST /crate/gguf/inspect` translates its `path` through `swarm-paths` first, so a **recorded** WSL or
`/mnt/c` literal opens on this host. The two facts are reported separately — `input_looks_posix` and
`resolver_changed_path` — because `resolve` also normalizes separators, and an earlier field named
`path_translated` claimed a boundary crossing the value could not prove.

**What the independent ruler found (33/33, `scratch/verify_crate_ops_p2j.py`).** `swarm-council`'s
`AGENT_MODELS` claims to mirror `swarm_v2/core/agent_intelligence_bridge.py`. Re-deriving both motions
from that Python table (parsed with `ast`, never imported) showed the crate carried **14 of the platform's
15 agents — `LogicMaster` was absent — and every domain list truncated** (Archi 10 vs 20; 128 vs 244
domains in total). The truncated table votes *identically* on the two documented motions, which is why
nothing caught it. The table is now **generated** from the oracle (`scratch/regenerate_council_table.py`),
the kernel serves 15 agents, and the crate's own test asserts the platform roster by name rather than a
count. Two real defects, one class: a mirror nobody compared to its source of truth.

## Honest scope

- **8 engines are reachable** (the four above plus k3, turbovec, instella, ssm). The correction this
  batch produced: `swarm-packs` and `swarm-tensor-bytes` are **not crates** — they are verify bins inside
  `swarm-git` and `swarm-gguf`, so the earlier "six without a route" was four crates and two gates.
  `swarm-proof` was wired at P2 (`POST /verification/proof`); `swarm-paths` is a helper, not an engine.
- This wiring is **additive**. The kernel already serves `/council/*`, `/k3/*` and `/mailbox/ring` from
  its own implementations; pointing those at crates instead would change semantics a dashboard already
  depends on, so that stays a deliberate decision rather than a drive-by rewrite.
- **The 43 `ENGINE_PORT` refusals are reduced by 8, not closed.**
- Unchanged and still recorded: `/k3/verdict` is a 404 (`docs/DASHBOARD_DATA_PATH_AUDIT_20260924.md`),
  and `swarm_v2_artifacts/KERNEL_MODULE_MAP.md` still describes `host_council_server` as "1 crate,
  `main.rs` 403 lines" — measured now: **955 lines with 21 `.rs` modules** in that directory, so that
  figure is stale (recorded here rather than silently edited in a dated map).
