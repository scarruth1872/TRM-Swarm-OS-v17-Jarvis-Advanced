# Wiring the routes: Layer 1 served from the kernel's own source

**Block 143 companion.** What was wired this round, the evidence, and the two deployment facts Windows taught
this round that the previous procedure did not carry.

## 1. `/sram` — the SRAM layer was implemented and unreachable

`trm_uefi_kernel/src/sram_pattern_computer.rs` carries Layer 1 (the 15 topologies, the L3 geometry, and — as
of the relaxation round — `hopfield_relax`). Nothing referenced it: no route, no crate op, no caller. It is
now served by a new handler, `host_council_server/src/sram_handlers.rs`, with the module linked **by path**:

```rust
#[path = "../../src/sram_pattern_computer.rs"]
mod sram_pattern_computer;
```

One implementation, no copy to drift. The route table is a `const` array of `(name, factory, default_arg)`,
so a 16th topology is a row rather than a branch.

```
GET  /sram/architectures   the 15 topologies + L3 geometry + per-pattern FNV-1a
POST /sram/relax           settle a state under stored patterns or explicit weights
GET  /sram/relax           405 post_only
POST /sram/relax (bad len) 400 invalid_request, naming the expected length
GET  /sram/<unknown>       404 not_found
```

## 2. The evidence — three comparisons that cannot confirm themselves

`scratch/probe_new_routes.py` → `swarm_v2_artifacts/route_probe.json`, **9 checks, 0 failed**:

| check | result |
|---|---|
| route hashes == native strip hashes | sets equal; strip rc 0, **15 distinct** |
| `POST /sram/relax` vs the Python mirror | rust **2 sweeps, E=-32032** == python 2 sweeps, E=-32032, **states equal** |
| recall through the route | `recalled_exactly: true`, `energy_monotone_nonincreasing: true` |
| refusals | 405 post_only · 400 "a pattern has 2 entries, expected 64" · 404 not_found |
| `/system/modules` | `active_core` 150 names (summary 253), `native_rust_engines` 252 (summary 315) |

The first row is the one that matters: an endpoint agreeing with itself proves nothing, so the route's hashes
are checked against a **separate artifact** (the deployment strip's own output).

**A distinction worth naming:** `/sram/architectures` reports the kernel's **design** constant
(L3_SIZE = 32 MB), while `/system` reports the **host's measured** L3 (`l3_sram_kb: 16384` = 16 MiB). Both are
correct for what they are; they are not the same quantity, and neither should be read as the other.

## 3. `/system/modules` — a census that answered nothing

The counts were derived from the tree, but the four name lists beside them were hardcoded `[]` with a `basis`
admitting "no inventory artifact enumerates them". A census asserting zero names over a non-empty set.
`list_files()` now walks exactly as `count_files()` does, so list and count cannot disagree; each list is
capped (150/150+80+80/40+40/40) and the caps are **named in the `basis`**, while `summary` always carries the
true totals. `dormant_core_sample` stays empty and says why — it needs an import graph, which is not computed
here.

## 4. Two deployment facts this round added

1. **Windows locks a running image.** `os.replace` onto the deployed `.exe` fails with `WinError 5`. Renaming
   a running exe *is* permitted, so the swap is: `os.replace(deployed, deployed.old)` →
   `os.replace(staged, deployed)`. Two instants, old image preserved as `.old`.
2. **The supervisor relaunches on process exit, not on file change.** `swarm_supervisor.py:308` restarts a
   service when `spec.process.poll() is not None` (3s delay, `max_restarts=10`). Swapping the file alone left
   the old image mapped for 90s+; stopping the process is what the deploy needs, and the supervisor brings it
   back from the new path. Proof, unchanged: **process start 13:09:02 > exe mtime 13:06:08**, `/health` 200.

## 5. What the README (and the campaign log) say the skeletons owe

The README documents the capability — **Layer 1**: "15 Virtual SRAM Architectures … Dynamically reconfigures
CPU L3 cache lines into 15 domain-specific compute topologies"; **Layer 5**: the 14-agent council; and the
route contract (a `/crate/*` op answers from the crate that owns the algorithm). Per-module obligation lives
in `docs/EFI_PORT_CAMPAIGN_LOG.md`, whose table gives every module a **port ratio** and a target form:

| module | port ratio | remaining fns | target |
|---|---|---|---|
| `gpu_infinity_computer` | 18% → 60% | 10 | strip DLL bit-exact |

"Strip DLL bit-exact" is the obligation: each module owes a native form that is **bit-exact against its
Python oracle** and reachable — which is exactly the pattern this round proved (`#[path]`-link the module,
export it, gate it with a parity ruler). The four remaining `host_tranche` skeletons and what they carry:

| skeleton | what it owes the platform (from the kernel-tree version's API) |
|---|---|
| `chaos_engine` | fault injection for resilience testing: `enable`/`disable`, `set_latency`, `set_dropout_rate`, `should_drop_connection_with`, `get_profile`, `apply_network_delay` |
| `expert_registry` | the council roster itself: `get_expert_registry` (Layer 5's 14 agents) |
| `gpu_infinity_computer` | pattern compute + Infinity Cache line hashing: `sram_lut_fpga`, `pattern_neural`, `pattern_cellular`, `pattern_k3_ternary`, `infinity_cache_line_hash`, `infinity_cache_hashes` |
| `persona_matrix` | identity → compiled prompt: `IdentityKernel`, `GenerationContext`, `GenerationResult`, `build_compiled_prompt(persona, kernel)` |
| `sram_pattern_computer` | Layer 1 — **done**: 15 topologies + relaxation, now served by `/sram` |

## Honest bounds

- The four skeletons are still skeletons in `host_tranche`; this round wired **only** the SRAM family, and it
  wired it by linking the kernel implementation rather than completing the host port. That is one rung of the
  ladder, not the whole ladder.
- The `/sram` route is on `:8022` only; no dashboard tab was added, so the LCARS UI does not yet surface it.
- `/system/modules` lists are capped by design; they are not a full enumeration of 253 core files.
